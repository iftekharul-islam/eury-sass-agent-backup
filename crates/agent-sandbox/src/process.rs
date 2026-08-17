use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command as TokioCommand};
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Execution timed out")]
    Timeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Process group kill failed")]
    KillFailed,
}

pub struct ProcessSupervisor {
    timeout_duration: Duration,
    output_cap_bytes: usize,
}

impl ProcessSupervisor {
    #[must_use]
    pub fn new(timeout_duration: Duration, output_cap_bytes: usize) -> Self {
        Self { timeout_duration, output_cap_bytes }
    }

    /// # Errors
    ///
    /// Returns [`ProcessError::Io`] if the child process fails to spawn, or
    /// [`ProcessError::Timeout`] if it does not exit within the configured
    /// timeout.
    pub async fn spawn_and_wait(
        &self,
        mut cmd: TokioCommand,
    ) -> Result<(ExitStatus, String, String), ProcessError> {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()?;

        let res = timeout(self.timeout_duration, child.wait_with_output()).await;

        match res {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

                let stdout = self.truncate_output(&stdout);
                let stderr = self.truncate_output(&stderr);

                Ok((output.status, stdout, stderr))
            }
            Ok(Err(e)) => Err(ProcessError::Io(e)),
            Err(_) => Err(ProcessError::Timeout),
        }
    }

    /// Like [`Self::spawn_and_wait`], but invokes `on_chunk(stream, text)` as
    /// stdout/stderr arrive so the UI can show live command logs.
    ///
    /// # Errors
    ///
    /// Same as [`Self::spawn_and_wait`].
    pub async fn spawn_and_wait_streaming<F>(
        &self,
        mut cmd: TokioCommand,
        on_chunk: F,
    ) -> Result<(ExitStatus, String, String), ProcessError>
    where
        F: FnMut(&str, &str) + Send + 'static,
    {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| {
            ProcessError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "stdout pipe missing",
            ))
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            ProcessError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "stderr pipe missing",
            ))
        })?;

        let cap = self.output_cap_bytes;
        let stdout_acc = Arc::new(Mutex::new(String::new()));
        let stderr_acc = Arc::new(Mutex::new(String::new()));
        let on_chunk = Arc::new(Mutex::new(on_chunk));

        let wait_result = timeout(self.timeout_duration, async {
            let (stdout_res, stderr_res, status) = tokio::join!(
                read_stream(
                    stdout,
                    "stdout",
                    Arc::clone(&stdout_acc),
                    cap,
                    Arc::clone(&on_chunk),
                ),
                read_stream(
                    stderr,
                    "stderr",
                    Arc::clone(&stderr_acc),
                    cap,
                    Arc::clone(&on_chunk),
                ),
                child.wait(),
            );
            stdout_res?;
            stderr_res?;
            status.map_err(ProcessError::Io)
        })
        .await;

        match wait_result {
            Ok(Ok(status)) => {
                let stdout = stdout_acc.lock().map(|g| g.clone()).unwrap_or_default();
                let stderr = stderr_acc.lock().map(|g| g.clone()).unwrap_or_default();
                Ok((status, stdout, stderr))
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                let _ = kill_child(&mut child).await;
                Err(ProcessError::Timeout)
            }
        }
    }

    fn truncate_output(&self, text: &str) -> String {
        if text.len() > self.output_cap_bytes {
            let half = self.output_cap_bytes / 2;
            let start = &text[..half];
            let end = &text[text.len() - half..];
            format!("{start}\n...[TRUNCATED]...\n{end}")
        } else {
            text.to_string()
        }
    }
}

async fn read_stream<F>(
    mut pipe: impl AsyncRead + Unpin,
    stream: &'static str,
    acc: Arc<Mutex<String>>,
    cap: usize,
    on_chunk: Arc<Mutex<F>>,
) -> Result<(), ProcessError>
where
    F: FnMut(&str, &str) + Send + 'static,
{
    let mut buf = [0u8; 8192];
    loop {
        let n = pipe.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        let text = String::from_utf8_lossy(&buf[..n]);
        {
            let mut guard = acc.lock().map_err(|_| ProcessError::KillFailed)?;
            append_capped(&mut guard, &text, cap);
        }
        if let Ok(mut guard) = on_chunk.lock() {
            guard(stream, &text);
        }
    }
    Ok(())
}

fn append_capped(acc: &mut String, chunk: &str, cap: usize) {
    acc.push_str(chunk);
    if acc.len() > cap {
        let keep = cap / 2;
        let tail = acc.split_off(acc.len().saturating_sub(keep));
        acc.clear();
        acc.push_str("...[TRUNCATED]...\n");
        acc.push_str(&tail);
    }
}

async fn kill_child(child: &mut Child) -> Result<(), ProcessError> {
    child.kill().await.map_err(ProcessError::Io)?;
    let _ = child.wait().await;
    Ok(())
}
