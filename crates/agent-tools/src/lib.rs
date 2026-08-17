pub mod errors;
pub mod fs;
pub mod registry;
pub mod registry_factory;
pub mod shell;

// Re-exports
pub use registry::{Tool, ToolError, ToolRegistry};
pub use registry_factory::default_registry;
