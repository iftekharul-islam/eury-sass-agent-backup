//! Future filesystem and process-security boundary.

#![allow(clippy::disallowed_types)]
#![allow(clippy::disallowed_methods)]

pub mod command;
pub mod os;
pub mod path;
pub mod process;
pub mod pty;
pub mod rm_safety;
pub mod workspace;

/// The crate identity used by Phase 0 build and boundary checks.
pub const CRATE_NAME: &str = "agent-sandbox";
