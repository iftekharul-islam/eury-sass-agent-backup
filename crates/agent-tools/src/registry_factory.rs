use crate::fs::edit::EditFileTool;
use crate::fs::read::ReadFileTool;
use crate::fs::search::{GlobTool, GrepTool, ListDirTool};
use crate::fs::write::WriteFileTool;
use crate::registry::ToolRegistry;
use crate::shell::run::RunCommandTool;

#[must_use]
pub fn default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ReadFileTool));
    registry.register(Box::new(ListDirTool));
    registry.register(Box::new(GlobTool));
    registry.register(Box::new(GrepTool));
    registry.register(Box::new(WriteFileTool));
    registry.register(Box::new(EditFileTool));
    registry.register(Box::new(RunCommandTool));
    registry
}
