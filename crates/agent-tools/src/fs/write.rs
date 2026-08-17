use crate::errors::TaxonomyError;
use crate::fs::diff::generate_unified_diff;
use crate::registry::{RiskLevel, Tool, ToolClass, ToolError};
use agent_sandbox::path::PathGuard;
use agent_sandbox::workspace::Workspace;
use agent_types::requests::RunMode;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::io::Read;
use std::path::Path;

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "Write file"
    }
    fn description(&self) -> &'static str {
        "Writes a file to the workspace, creating or overwriting it."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"],
            "additionalProperties": false
        })
    }

    fn class(&self) -> ToolClass {
        ToolClass::Write
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn modes(&self) -> Vec<RunMode> {
        vec![RunMode::Agent, RunMode::Build]
    }

    fn idempotent(&self) -> bool {
        true
    }
    fn mutates(&self) -> bool {
        true
    }
    fn checkpointed(&self) -> bool {
        true
    }
    fn timeout_ms(&self) -> u32 {
        10_000
    }
    fn result_cap_tokens(&self) -> u32 {
        10_000
    }

    async fn execute(
        &self,
        args: Value,
        workspace: Option<&Workspace>,
    ) -> Result<Value, ToolError> {
        let ws = workspace.ok_or_else(|| ToolError::Execution("Workspace required".into()))?;
        let rel_path_str = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Validation("path must be string".into()))?;
        let rel_path = Path::new(rel_path_str);
        let content = args["content"].as_str().unwrap_or("");

        let old_content = match PathGuard::open_read(ws, rel_path) {
            Ok(mut file) => {
                let mut s = String::new();
                file.read_to_string(&mut s).unwrap_or_default();
                s
            }
            Err(_) => String::new(),
        };

        PathGuard::open_write(ws, rel_path, content.as_bytes()).map_err(|e| {
            ToolError::from(TaxonomyError::ExecutionFailed(format!("Write failed: {e}")))
        })?;

        let diff_str = generate_unified_diff(&old_content, content, rel_path_str);

        Ok(json!({
            "success": true,
            "diff": diff_str
        }))
    }
}
