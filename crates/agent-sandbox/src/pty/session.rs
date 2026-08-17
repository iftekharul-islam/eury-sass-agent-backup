use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use uuid::Uuid;

use super::PtyError;
use super::coalesce::FrameCoalescer;
use super::env::SanitizedEnv;
use super::kill::PtyKiller;
use super::ring::ScrollbackRing;
use super::shell::ShellSpec;

pub struct PtySpawnSpec {
    pub cwd: PathBuf,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum PtyStatus {
    Running,
    Exited { code: Option<i32> },
    Killed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtySessionInfo {
    pub terminal_id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub cwd: PathBuf,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub pid: Option<u32>,
    pub status: PtyStatus,
    pub created_at: i64,
    pub degraded: Option<String>,
}

/// One live PTY-backed shell session.
///
/// Every `std::sync::Mutex` here guards a short, non-`.await`-holding
/// critical section (raw reads/writes, struct field updates); recovery from
/// poisoning uses `unwrap_or_else(PoisonError::into_inner)` throughout
/// rather than `.unwrap()`, since this crate denies `unwrap_used`.
pub(crate) struct PtySession {
    pub terminal_id: Uuid,
    /// `None` after `terminate()` drops it — closing the master fd delivers
    /// `SIGHUP` to the foreground process group on unix.
    master: StdMutex<Option<Box<dyn MasterPty + Send>>>,
    writer: StdMutex<Box<dyn Write + Send>>,
    child: StdMutex<Box<dyn portable_pty::Child + Send + Sync>>,
    killer: PtyKiller,
    pub ring: StdMutex<ScrollbackRing>,
    pub coalescer: StdMutex<FrameCoalescer>,
    info: StdMutex<PtySessionInfo>,
    status_tx: watch::Sender<PtyStatus>,
    pub status_rx: watch::Receiver<PtyStatus>,
}

fn now_millis() -> i64 {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

impl PtySession {
    /// # Errors
    ///
    /// Returns [`PtyError::Backend`] if the platform PTY backend fails to
    /// open a pty pair, spawn the shell, or acquire the master reader/writer.
    pub fn spawn(
        terminal_id: Uuid,
        workspace_id: Uuid,
        spec: &PtySpawnSpec,
        shell: &ShellSpec,
        env: &SanitizedEnv,
        ring_capacity: usize,
        max_frame_bytes: usize,
    ) -> Result<Arc<PtySession>, PtyError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: spec.rows, cols: spec.cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| PtyError::Backend(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&shell.program);
        cmd.args(&shell.args);
        cmd.cwd(&spec.cwd);
        cmd.env_clear();
        for (key, value) in env.iter() {
            cmd.env(key, value);
        }

        let child = pair.slave.spawn_command(cmd).map_err(|e| PtyError::Backend(e.to_string()))?;
        // The child owns the slave fd from here; the parent's copy must go.
        drop(pair.slave);

        let pid = child.process_id();
        let killer = PtyKiller::capture(pid);

        let reader =
            pair.master.try_clone_reader().map_err(|e| PtyError::Backend(e.to_string()))?;
        let writer = pair.master.take_writer().map_err(|e| PtyError::Backend(e.to_string()))?;

        let (status_tx, status_rx) = watch::channel(PtyStatus::Running);

        let title = shell
            .program
            .file_name()
            .map_or_else(|| "shell".to_string(), |n| n.to_string_lossy().into_owned());

        let info = PtySessionInfo {
            terminal_id,
            workspace_id,
            title,
            cwd: spec.cwd.clone(),
            shell: shell.program.to_string_lossy().into_owned(),
            cols: spec.cols,
            rows: spec.rows,
            pid,
            status: PtyStatus::Running,
            created_at: now_millis(),
            degraded: shell.degraded.map(|d| d.message().to_string()),
        };

        let session = Arc::new(PtySession {
            terminal_id,
            master: StdMutex::new(Some(pair.master)),
            writer: StdMutex::new(writer),
            child: StdMutex::new(child),
            killer,
            ring: StdMutex::new(ScrollbackRing::with_capacity(ring_capacity)),
            coalescer: StdMutex::new(FrameCoalescer::new(max_frame_bytes)),
            info: StdMutex::new(info),
            status_tx,
            status_rx,
        });

        spawn_reader_thread(session.clone(), reader)?;

        Ok(session)
    }

    /// # Errors
    ///
    /// Returns [`PtyError::Io`] if the write to the pty master fails.
    pub fn write(&self, data: &[u8]) -> Result<(), PtyError> {
        let mut writer = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
        writer.write_all(data)?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns [`PtyError::InvalidSize`] if `cols` or `rows` is zero, or
    /// [`PtyError::Backend`] if the platform resize call fails.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        if cols == 0 || rows == 0 {
            return Err(PtyError::InvalidSize);
        }
        let master = self.master.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(master) = master.as_ref() {
            master
                .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
                .map_err(|e| PtyError::Backend(e.to_string()))?;
        }
        drop(master);
        let mut info = self.info.lock().unwrap_or_else(PoisonError::into_inner);
        info.cols = cols;
        info.rows = rows;
        Ok(())
    }

    #[must_use]
    pub fn capture(&self, lines: Option<usize>) -> Vec<u8> {
        let ring = self.ring.lock().unwrap_or_else(PoisonError::into_inner);
        match lines {
            Some(n) => ring.tail_lines(n),
            None => ring.snapshot(),
        }
    }

    #[must_use]
    pub fn info(&self) -> PtySessionInfo {
        self.info.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// SIGHUP the process group, drop the master fd (catches foreground
    /// programs the shell doesn't forward a HUP to), wait up to `grace` for
    /// a clean exit, then SIGKILL. Always reaps, even if the process had
    /// already exited, so nothing is left a zombie.
    pub async fn terminate(&self, grace: Duration) {
        self.killer.hangup();
        {
            let mut master = self.master.lock().unwrap_or_else(PoisonError::into_inner);
            *master = None;
        }

        let deadline = tokio::time::Instant::now() + grace;
        while tokio::time::Instant::now() < deadline {
            let alive = {
                let mut child = self.child.lock().unwrap_or_else(PoisonError::into_inner);
                matches!(child.try_wait(), Ok(None))
            };
            if !alive {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        self.killer.kill_hard();

        {
            let mut child = self.child.lock().unwrap_or_else(PoisonError::into_inner);
            let _ = child.wait();
        }

        {
            let mut info = self.info.lock().unwrap_or_else(PoisonError::into_inner);
            info.status = PtyStatus::Killed;
        }
        let _ = self.status_tx.send(PtyStatus::Killed);
    }
}

fn spawn_reader_thread(
    session: Arc<PtySession>,
    mut reader: Box<dyn Read + Send>,
) -> Result<std::thread::JoinHandle<()>, PtyError> {
    std::thread::Builder::new()
        .name(format!("pty-read-{}", session.terminal_id))
        .spawn(move || {
            let mut buf = vec![0u8; 32 * 1024].into_boxed_slice();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        {
                            let mut ring =
                                session.ring.lock().unwrap_or_else(PoisonError::into_inner);
                            ring.push(&buf[..n]);
                        }
                        let mut coalescer =
                            session.coalescer.lock().unwrap_or_else(PoisonError::into_inner);
                        coalescer.append(&buf[..n]);
                    }
                }
            }

            let exit_code = {
                let mut child = session.child.lock().unwrap_or_else(PoisonError::into_inner);
                child.wait().ok().map(|status| i32::try_from(status.exit_code()).unwrap_or(-1))
            };
            {
                let mut info = session.info.lock().unwrap_or_else(PoisonError::into_inner);
                if info.status == PtyStatus::Running {
                    info.status = PtyStatus::Exited { code: exit_code };
                }
            }
            let _ = session.status_tx.send(PtyStatus::Exited { code: exit_code });
        })
        .map_err(PtyError::Io)
}
