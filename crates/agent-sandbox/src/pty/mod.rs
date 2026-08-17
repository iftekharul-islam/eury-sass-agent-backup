//! PTY-backed terminal sessions (Phase 12).
//!
//! Owns everything with direct process/PTY access for interactive terminal
//! sessions, mirroring the crate's existing boundary for `run_command`
//! (`crate::process`). Unlike `ProcessSupervisor`, sessions here are
//! long-lived, driven by external writes/resizes, and killed by id rather
//! than captured once via `wait_with_output()`.

pub mod ansi;
pub mod coalesce;
pub mod env;
pub mod kill;
pub mod manager;
pub mod ring;
pub mod session;
pub mod shell;

pub use env::{SanitizedEnv, sanitize};
pub use manager::{MAX_SESSIONS_PER_WORKSPACE, PtyEvent, PtyManager};
pub use session::{PtySessionInfo, PtyStatus};
pub use shell::{DegradeReason, ShellSpec, resolve_shell};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("no shell available: {0}")]
    NoShell(String),
    #[error("session not found")]
    NotFound,
    #[error("workspace already has the maximum of {max} terminal sessions")]
    SessionLimit { max: usize },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PTY backend error: {0}")]
    Backend(String),
    #[error("invalid size: cols and rows must both be non-zero")]
    InvalidSize,
}
