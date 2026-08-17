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

pub struct EditFileTool;

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "Edit file"
    }
    fn description(&self) -> &'static str {
        "Exact string replacement in a file."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "oldString": { "type": "string" },
                "newString": { "type": "string" }
            },
            "required": ["path", "oldString", "newString"],
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

        let old_str = args["oldString"].as_str().unwrap_or("");
        let new_str = args["newString"].as_str().unwrap_or("");

        // A model reaching for edit_file on a file that does not exist yet is
        // common, and "IO Error: No such file or directory (os error 2)" tells
        // it nothing — it just retries the same call until the round budget is
        // gone. Name the file and the tool that would work.
        let mut file = PathGuard::open_read(ws, rel_path).map_err(|e| {
            let missing = matches!(
                e,
                agent_sandbox::path::PathGuardError::Io(ref io)
                    if io.kind() == std::io::ErrorKind::NotFound
            );
            if missing {
                ToolError::from(TaxonomyError::NotFound(format!(
                    "{rel_path_str} does not exist, so there is nothing to edit.                      Call write_file to create it instead — do not retry edit_file."
                )))
            } else {
                ToolError::from(TaxonomyError::NotFound(e.to_string()))
            }
        })?;

        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            ToolError::from(TaxonomyError::ExecutionFailed(format!("Read failed: {e}")))
        })?;

        let matches: Vec<_> = content.match_indices(old_str).collect();
        if matches.is_empty() {
            return Err(ToolError::from(TaxonomyError::EditAmbiguous(
                "oldString not found in file".into(),
            )));
        }
        if matches.len() > 1 {
            return Err(ToolError::from(TaxonomyError::EditAmbiguous(
                "oldString found multiple times, not unique".into(),
            )));
        }

        let new_content = content.replace(old_str, new_str);

        PathGuard::open_write(ws, rel_path, new_content.as_bytes()).map_err(|e| {
            ToolError::from(TaxonomyError::ExecutionFailed(format!("Write failed: {e}")))
        })?;

        let diff_str = generate_unified_diff(&content, &new_content, rel_path_str);

        Ok(json!({
            "success": true,
            "diff": diff_str
        }))
    }
}
