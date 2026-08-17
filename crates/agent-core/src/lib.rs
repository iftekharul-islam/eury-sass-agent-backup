//! Future orchestration and `AgentEngine` boundary.
//!
//! Cersei is intentionally not added in Phase 0.

pub mod accounting;
pub mod agent_loop;
pub mod approval;
pub mod assembly;
pub mod auth;
pub mod cersei_adapter;
pub mod engine;
pub mod providers;
pub mod run_manager;
pub mod tool_calls;

/// The crate identity used by Phase 0 build and boundary checks.
pub const CRATE_NAME: &str = "agent-core";
