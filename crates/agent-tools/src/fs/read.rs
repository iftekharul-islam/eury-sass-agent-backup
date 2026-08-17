use crate::registry::{RiskLevel, Tool, ToolClass, ToolError};
use agent_sandbox::path::PathGuard;
use agent_sandbox::workspace::Workspace;
use agent_types::requests::RunMode;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::io::Read;
use std::path::Path;

/// Lines returned when the caller doesn't specify a `limit`. Large enough for
/// most source files, small enough that one call can't exhaust the context.
const DEFAULT_LINE_LIMIT: u64 = 2000;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "Read file"
    }
    fn description(&self) -> &'static str {
        "Reads a file from the workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "minimum": 1 },
                "limit": { "type": "integer", "minimum": 1, "maximum": 5000 }
            },
            "required": ["path"],
            "additionalProperties": false
        })
    }

    fn class(&self) -> ToolClass {
        ToolClass::Read
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn modes(&self) -> Vec<RunMode> {
        vec![RunMode::Ask, RunMode::Plan, RunMode::Agent, RunMode::Build]
    }

    fn idempotent(&self) -> bool {
        true
    }
    fn mutates(&self) -> bool {
        false
    }
    fn checkpointed(&self) -> bool {
        false
    }
    fn timeout_ms(&self) -> u32 {
        10_000
    }
    fn result_cap_tokens(&self) -> u32 {
        25_000
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

        // `offset` is 1-based to match how editors and the tool schema talk
        // about line numbers; `limit` caps how many lines come back so a large
        // file can't flood the model's context in a single call.
        let offset = args.get("offset").and_then(Value::as_u64).unwrap_or(1).max(1);
        let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(DEFAULT_LINE_LIMIT);

        let mut file = PathGuard::open_read(ws, rel_path)
            .map_err(|e| ToolError::Execution(format!("EURY_TOOL_NOT_FOUND: {e}")))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| ToolError::Execution(format!("EURY_TOOL_READ_ERROR: {e}")))?;

        let total_lines = content.lines().count() as u64;
        let start = offset.saturating_sub(1);
        let selected: Vec<&str> = content
            .lines()
            .skip(usize::try_from(start).unwrap_or(usize::MAX))
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .collect();
        let returned_lines = selected.len() as u64;

        Ok(json!({
            "content": selected.join("\n"),
            "startLine": offset,
            "endLine": start + returned_lines,
            "totalLines": total_lines,
            // True when lines were withheld, so the caller knows to page rather
            // than assume it has seen the whole file.
            "truncated": start + returned_lines < total_lines,
        }))
    }
}
