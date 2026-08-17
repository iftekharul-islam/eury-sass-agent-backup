//! Integration tests exercising a real PTY-backed shell through the public
//! `PtyManager` API — the same surface the Tauri IPC layer drives.

use agent_sandbox::pty::{PtyEvent, PtyManager};
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;

async fn collect_until(
    rx: &mut mpsc::Receiver<PtyEvent>,
    contains: &str,
    timeout: Duration,
) -> String {
    let mut collected = Vec::new();
    let deadline = tokio::time::Instant::now() + timeout;

    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining.min(Duration::from_millis(250)), rx.recv()).await {
            Ok(Some(PtyEvent::Data(frame))) => {
                collected.extend_from_slice(&frame.bytes);
                if String::from_utf8_lossy(&collected).contains(contains) {
                    break;
                }
            }
            Ok(Some(_)) | Err(_) => {}
            Ok(None) => break,
        }
    }

    String::from_utf8_lossy(&collected).into_owned()
}

#[tokio::test]
async fn spawn_write_echo_close() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PtyManager::new("test");
    let workspace_id = Uuid::now_v7();
    let workspace_root = std::env::temp_dir();
    let (tx, mut rx) = mpsc::channel(256);

    let info = manager.create(workspace_id, &workspace_root, None, None, 80, 24, tx).await?;
    assert!(info.pid.is_some(), "expected a spawned shell to report a pid");

    manager.write(info.terminal_id, b"echo pty-lifecycle-marker\n").await?;

    let output = collect_until(&mut rx, "pty-lifecycle-marker", Duration::from_secs(10)).await;
    assert!(
        output.contains("pty-lifecycle-marker"),
        "expected echoed marker in output, got: {output:?}"
    );

    manager.resize(info.terminal_id, 100, 40).await?;

    let sessions = manager.list(Some(workspace_id)).await;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].cols, 100);
    assert_eq!(sessions[0].rows, 40);

    manager.close(info.terminal_id).await?;
    let sessions_after_close = manager.list(Some(workspace_id)).await;
    assert!(sessions_after_close.is_empty());

    Ok(())
}

#[tokio::test]
async fn capture_returns_scrollback_after_write() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PtyManager::new("test");
    let workspace_id = Uuid::now_v7();
    let workspace_root = std::env::temp_dir();
    let (tx, mut rx) = mpsc::channel(256);

    let info = manager.create(workspace_id, &workspace_root, None, None, 80, 24, tx).await?;
    manager.write(info.terminal_id, b"echo capture-marker\n").await?;
    let _ = collect_until(&mut rx, "capture-marker", Duration::from_secs(10)).await;

    let captured = manager.capture(info.terminal_id, Some(50)).await?;
    let captured_text = String::from_utf8_lossy(&captured);
    assert!(
        captured_text.contains("capture-marker"),
        "expected capture to include the echoed marker, got: {captured_text:?}"
    );

    manager.close(info.terminal_id).await?;
    Ok(())
}

#[tokio::test]
async fn session_cap_is_enforced_per_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PtyManager::new("test");
    let workspace_id = Uuid::now_v7();
    let workspace_root = std::env::temp_dir();

    let mut created = Vec::new();
    for _ in 0..4 {
        let (tx, _rx) = mpsc::channel(16);
        let info = manager.create(workspace_id, &workspace_root, None, None, 80, 24, tx).await?;
        created.push(info.terminal_id);
    }

    let (tx, _rx) = mpsc::channel(16);
    let over_cap = manager.create(workspace_id, &workspace_root, None, None, 80, 24, tx).await;
    assert!(over_cap.is_err(), "a 5th session in the same workspace should be rejected");

    for id in created {
        manager.close(id).await?;
    }
    Ok(())
}

/// Confirms `close()` leaves no orphaned process behind — the property
/// D12.1 and the phase's "0 orphaned shells" metric depend on.
#[cfg(unix)]
#[tokio::test]
async fn close_leaves_no_orphaned_process() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PtyManager::new("test");
    let workspace_id = Uuid::now_v7();
    let workspace_root = std::env::temp_dir();
    let (tx, _rx) = mpsc::channel(256);

    let info = manager.create(workspace_id, &workspace_root, None, None, 80, 24, tx).await?;
    let Some(pid) = info.pid else {
        return Err("expected a pid".into());
    };

    manager.close(info.terminal_id).await?;

    // Signal 0 checks existence without sending a real signal.
    let target = nix::unistd::Pid::from_raw(i32::try_from(pid).unwrap_or(i32::MAX));
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut alive = true;
    while tokio::time::Instant::now() < deadline {
        alive = nix::sys::signal::kill(target, None::<nix::sys::signal::Signal>).is_ok();
        if !alive {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(!alive, "shell process {pid} is still alive after close()");

    Ok(())
}
