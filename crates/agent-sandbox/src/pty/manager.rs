use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use super::PtyError;
use super::coalesce::OutputFrame;
use super::env::sanitize;
use super::session::{PtySession, PtySessionInfo, PtySpawnSpec, PtyStatus};
use super::shell::resolve_shell;

/// Matches `phase-12.md` / the terminal UI spec ("up to 4 concurrent per
/// workspace"). The IPC command spec previously said "8 per window"; that
/// was a spec bug, corrected alongside this implementation.
pub const MAX_SESSIONS_PER_WORKSPACE: usize = 4;

/// 4 MiB scrollback per session, per the terminal UI spec.
const RING_CAPACITY_BYTES: usize = 4 * 1024 * 1024;

/// 512 KiB/frame at a 16 ms tick is a 30 MB/s ceiling — 3x headroom over
/// the 10 MB/s throughput target.
const MAX_FRAME_BYTES: usize = 512 * 1024;

const FLUSH_INTERVAL: Duration = Duration::from_millis(16);

/// Events forwarded out of `PtyManager` for one session, over the
/// `mpsc::Sender` passed to `create`. The Tauri layer (`commands.rs`)
/// translates these into the wire `TerminalFrame` shape sent over that
/// session's `Channel`.
#[derive(Debug)]
pub enum PtyEvent {
    Data(OutputFrame),
    Exited { code: Option<i32> },
    Killed,
}

struct ManagedSession {
    session: std::sync::Arc<PtySession>,
    flush_task: tokio::task::JoinHandle<()>,
}

pub struct PtyManager {
    sessions: RwLock<HashMap<Uuid, ManagedSession>>,
    app_version: String,
}

impl PtyManager {
    #[must_use]
    pub fn new(app_version: impl Into<String>) -> Self {
        Self { sessions: RwLock::new(HashMap::new()), app_version: app_version.into() }
    }

    /// Spawns a new PTY session for `workspace_id`, enforcing the
    /// per-workspace session cap. `sink` receives coalesced output and
    /// lifecycle events until the session closes or exits.
    ///
    /// # Errors
    ///
    /// Returns [`PtyError::InvalidSize`] if `cols`/`rows` is zero,
    /// [`PtyError::SessionLimit`] if the workspace is already at
    /// [`MAX_SESSIONS_PER_WORKSPACE`], or [`PtyError::NoShell`]/
    /// [`PtyError::Backend`] if the shell cannot be resolved or spawned.
    // Each parameter is a distinct, independently-supplied piece of spawn
    // configuration (workspace scoping, working directory, shell override,
    // terminal geometry, event sink) — bundling them into a struct would
    // just move the same eight fields one level down without reducing
    // caller complexity.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        workspace_id: Uuid,
        workspace_root: &Path,
        cwd: Option<PathBuf>,
        shell_override: Option<&str>,
        cols: u16,
        rows: u16,
        sink: mpsc::Sender<PtyEvent>,
    ) -> Result<PtySessionInfo, PtyError> {
        if cols == 0 || rows == 0 {
            return Err(PtyError::InvalidSize);
        }

        {
            let sessions = self.sessions.read().await;
            let count =
                sessions.values().filter(|m| m.session.info().workspace_id == workspace_id).count();
            if count >= MAX_SESSIONS_PER_WORKSPACE {
                return Err(PtyError::SessionLimit { max: MAX_SESSIONS_PER_WORKSPACE });
            }
        }

        let shell = resolve_shell(shell_override)?;
        let env = sanitize(workspace_root, &self.app_version, std::env::vars());
        let terminal_id = Uuid::now_v7();
        let spawn_spec =
            PtySpawnSpec { cwd: cwd.unwrap_or_else(|| workspace_root.to_path_buf()), cols, rows };

        let session = PtySession::spawn(
            terminal_id,
            workspace_id,
            &spawn_spec,
            &shell,
            &env,
            RING_CAPACITY_BYTES,
            MAX_FRAME_BYTES,
        )?;

        let flush_task = spawn_flush_task(session.clone(), sink);
        let info = session.info();

        let mut sessions = self.sessions.write().await;
        sessions.insert(terminal_id, ManagedSession { session, flush_task });

        Ok(info)
    }

    /// # Errors
    ///
    /// Returns [`PtyError::NotFound`] if `id` has no live session, or
    /// [`PtyError::Io`] if the write fails.
    pub async fn write(&self, id: Uuid, data: &[u8]) -> Result<(), PtyError> {
        let sessions = self.sessions.read().await;
        let managed = sessions.get(&id).ok_or(PtyError::NotFound)?;
        managed.session.write(data)
    }

    /// # Errors
    ///
    /// Returns [`PtyError::NotFound`] if `id` has no live session, or
    /// [`PtyError::InvalidSize`]/[`PtyError::Backend`] on an invalid or
    /// failed resize.
    pub async fn resize(&self, id: Uuid, cols: u16, rows: u16) -> Result<(), PtyError> {
        let sessions = self.sessions.read().await;
        let managed = sessions.get(&id).ok_or(PtyError::NotFound)?;
        managed.session.resize(cols, rows)
    }

    /// # Errors
    ///
    /// Returns [`PtyError::NotFound`] if `id` has no live session.
    pub async fn close(&self, id: Uuid) -> Result<(), PtyError> {
        let managed = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&id).ok_or(PtyError::NotFound)?
        };
        managed.session.terminate(Duration::from_secs(2)).await;
        managed.flush_task.abort();
        Ok(())
    }

    pub async fn list(&self, workspace_id: Option<Uuid>) -> Vec<PtySessionInfo> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .map(|m| m.session.info())
            .filter(|info| workspace_id.is_none_or(|w| w == info.workspace_id))
            .collect()
    }

    /// # Errors
    ///
    /// Returns [`PtyError::NotFound`] if `id` has no live session.
    pub async fn capture(&self, id: Uuid, lines: Option<usize>) -> Result<Vec<u8>, PtyError> {
        let sessions = self.sessions.read().await;
        let managed = sessions.get(&id).ok_or(PtyError::NotFound)?;
        Ok(managed.session.capture(lines))
    }

    /// Terminates every live session concurrently, bounded by `grace` each.
    /// Called from the Tauri app's quit handler — must not block exit on a
    /// hung shell.
    pub async fn shutdown_all(&self, grace: Duration) {
        let drained: Vec<_> = {
            let mut sessions = self.sessions.write().await;
            sessions.drain().collect()
        };

        let mut handles = Vec::with_capacity(drained.len());
        for (_, managed) in drained {
            handles.push(tokio::spawn(async move {
                managed.session.terminate(grace).await;
                managed.flush_task.abort();
            }));
        }
        for handle in handles {
            let _ = handle.await;
        }
    }
}

fn spawn_flush_task(
    session: std::sync::Arc<PtySession>,
    sink: mpsc::Sender<PtyEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(FLUSH_INTERVAL);
        let mut status_rx = session.status_rx.clone();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let frame = {
                        let mut coalescer = session
                            .coalescer
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        coalescer.flush()
                    };
                    if let Some(frame) = frame
                        && sink.send(PtyEvent::Data(frame)).await.is_err()
                    {
                        break;
                    }
                }
                changed = status_rx.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    let status = status_rx.borrow_and_update().clone();
                    match status {
                        PtyStatus::Exited { code } => {
                            let frame = {
                                let mut coalescer = session
                                    .coalescer
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                                coalescer.flush()
                            };
                            if let Some(frame) = frame {
                                let _ = sink.send(PtyEvent::Data(frame)).await;
                            }
                            let _ = sink.send(PtyEvent::Exited { code }).await;
                            break;
                        }
                        PtyStatus::Killed => {
                            let _ = sink.send(PtyEvent::Killed).await;
                            break;
                        }
                        PtyStatus::Running => {}
                    }
                }
            }
        }
    })
}
