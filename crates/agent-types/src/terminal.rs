use serde::{Deserialize, Serialize};

/// Wire frames sent over a terminal session's dedicated
/// `tauri::ipc::Channel`, tagged the same way as `events::AgentEvent`.
///
/// `Data.bytes` is currently JSON-serialized (a number array over the
/// wire) rather than sent as a raw `ArrayBuffer` response body. That is a
/// correctness-first interim choice, not the final throughput-critical
/// design: hitting the 10 MB/s target needs a raw binary encoding (see
/// Phase 12's IPC plan) and is tracked as follow-up work, not silently
/// dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum TerminalFrame {
    Data { seq: u64, dropped_before: u64, bytes: Vec<u8> },
    Exited { code: Option<i32> },
    Killed,
    Degraded { reason: String, detail: String },
}
