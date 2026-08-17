use crate::registry::{RiskLevel, Tool, ToolClass, ToolError};
use agent_sandbox::path::PathGuard;
use agent_sandbox::workspace::Workspace;
use agent_types::requests::RunMode;
use async_trait::async_trait;
use ignore::WalkBuilder;
use serde_json::{Value, json};
use std::path::Path;

pub struct ListDirTool;

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &'static str {
        "list_dir"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "List Directory"
    }
    fn description(&self) -> &'static str {
        "Lists files in a directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
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
        vec![]
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
        10_000
    }

    async fn execute(
        &self,
        args: Value,
        workspace: Option<&Workspace>,
    ) -> Result<Value, ToolError> {
        let ws = workspace.ok_or_else(|| ToolError::Execution("Workspace required".into()))?;
        let rel_path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let rel_path = Path::new(rel_path_str);

        // Route the traversal root through PathGuard rather than a plain
        // `starts_with` check, which `..` segments and symlinks defeat.
        let (dir, canon_root) = PathGuard::resolve_dir(ws, rel_path)
            .map_err(|e| ToolError::Execution(e.to_string()))?;

        let walker = WalkBuilder::new(&dir).max_depth(Some(1)).build();
        let mut entries = Vec::new();
        let mut count = 0;

        for res in walker {
            if let Ok(entry) = res
                && entry.path() != dir
                && let Ok(stripped) = entry.path().strip_prefix(&canon_root)
            {
                entries.push(stripped.to_string_lossy().to_string());
                count += 1;
                if count > 1000 {
                    break;
                }
            }
        }

        entries.sort();

        Ok(json!({
            "files": entries,
            "truncated": count > 1000
        }))
    }
}

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &'static str {
        "glob"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "Glob Search"
    }
    fn description(&self) -> &'static str {
        "Search for files using glob pattern"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "dir": { "type": "string" }
            },
            "required": ["pattern"],
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
        vec![]
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
        30_000
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
        let pattern = args["pattern"].as_str().unwrap_or("");
        let dir = args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");

        // Route the traversal root through PathGuard rather than a plain
        // `starts_with` check, which `..` segments and symlinks defeat.
        let (search_root, canon_root) = PathGuard::resolve_dir(ws, Path::new(dir))
            .map_err(|e| ToolError::Execution(e.to_string()))?;

        let builder = globset::GlobBuilder::new(pattern);
        let matcher =
            builder.build().map_err(|e| ToolError::Validation(e.to_string()))?.compile_matcher();

        let walker = WalkBuilder::new(&search_root).build();
        let mut entries = Vec::new();
        let mut count = 0;

        for res in walker {
            if let Ok(entry) = res
                && let Ok(stripped) = entry.path().strip_prefix(&canon_root)
                && matcher.is_match(stripped)
            {
                entries.push(stripped.to_string_lossy().to_string());
                count += 1;
                if count > 1000 {
                    break;
                }
            }
        }

        entries.sort();

        Ok(json!({
            "matches": entries,
            "truncated": count > 1000
        }))
    }
}

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "Grep Search"
    }
    fn description(&self) -> &'static str {
        "Search for regex pattern in files"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "dir": { "type": "string" }
            },
            "required": ["pattern"],
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
        vec![]
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
        30_000
    }
    fn result_cap_tokens(&self) -> u32 {
        15_000
    }

    async fn execute(
        &self,
        args: Value,
        workspace: Option<&Workspace>,
    ) -> Result<Value, ToolError> {
        let ws = workspace.ok_or_else(|| ToolError::Execution("Workspace required".into()))?;
        let pattern_str = args["pattern"].as_str().unwrap_or("");
        let dir = args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");

        let full_path = ws.root_path.join(dir);
        if !full_path.starts_with(&ws.root_path) {
            return Err(ToolError::Execution("Path traversal attempt".into()));
        }

        let re =
            regex::Regex::new(pattern_str).map_err(|e| ToolError::Validation(e.to_string()))?;

        let walker = WalkBuilder::new(full_path).build();
        let mut results = Vec::new();
        let mut match_count = 0;

        for res in walker {
            if let Ok(entry) = res
                && entry.file_type().is_some_and(|ft| ft.is_file())
                && let Ok(stripped) = entry.path().strip_prefix(&ws.root_path)
                && let Ok(mut file) = agent_sandbox::path::PathGuard::open_read(ws, stripped)
            {
                use std::io::Read;
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    for (i, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            results.push(json!({
                                "file": stripped.to_string_lossy().to_string(),
                                "line": i + 1,
                                "content": line.trim()
                            }));
                            match_count += 1;
                            if match_count >= 500 {
                                break;
                            }
                        }
                    }
                }
            }
            if match_count >= 500 {
                break;
            }
        }

        Ok(json!({
            "matches": results,
            "truncated": match_count >= 500
        }))
    }
}
