//! Shared, I/O-free types for the Eury Agent workspace.

pub mod capabilities;
pub mod context;
pub mod errors;
pub mod events;
pub mod requests;
pub mod terminal;

/// The crate identity used by Phase 0 build and boundary checks.
pub const CRATE_NAME: &str = "agent-types";
