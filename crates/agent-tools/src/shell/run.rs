use crate::errors::TaxonomyError;
use crate::registry::{RiskLevel, Tool, ToolClass, ToolError};
use agent_sandbox::command::CommandGuard;
use agent_sandbox::process::ProcessSupervisor;
use agent_sandbox::workspace::Workspace;
use agent_types::requests::RunMode;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::time::Duration;

pub struct RunCommandTool;

fn build_command(cmd_str: &str, ws: &Workspace) -> Result<tokio::process::Command, ToolError> {
    let shape = CommandGuard::parse_and_verify(cmd_str).map_err(|e| {
        ToolError::from(TaxonomyError::ExecutionFailed(format!(
            "Command validation failed: {e}"
        )))
    })?;

    #[allow(clippy::disallowed_types)]
    let cmd = if shape.is_shell_eval {
        let mut c = tokio::process::Command::new("sh");
        c.arg("-c").arg(cmd_str).current_dir(&ws.root_path);
        c
    } else {
        let mut c = tokio::process::Command::new(&shape.executable);
        c.args(&shape.args).current_dir(&ws.root_path);
        c
    };

    Ok(cmd)
}

/// Runs a shell command, streaming stdout/stderr chunks to `on_chunk` as they
/// arrive. Used by the agent loop so the desktop can show live command logs.
pub async fn execute_run_command_streaming<F>(
    args: Value,
    workspace: &Workspace,
    timeout: Duration,
    cap_bytes: usize,
    on_chunk: F,
) -> Result<Value, ToolError>
where
    F: FnMut(&str, &str) + Send + 'static,
{
    let cmd_str = args["command"]
        .as_str()
        .ok_or_else(|| ToolError::Validation("command must be string".into()))?;

    let cmd = build_command(cmd_str, workspace)?;
    let supervisor = ProcessSupervisor::new(timeout, cap_bytes);
    let (status, stdout, stderr) = supervisor
        .spawn_and_wait_streaming(cmd, on_chunk)
        .await
        .map_err(|e| {
            ToolError::from(TaxonomyError::ExecutionFailed(format!("Execution failed: {e}")))
        })?;

    let truncated = stdout.len() >= cap_bytes || stderr.len() >= cap_bytes;

    Ok(json!({
        "exit_code": status.code().unwrap_or(-1),
        "stdout": stdout,
        "stderr": stderr,
        "truncated": truncated
    }))
}
#[async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &'static str {
        "run_command"
    }
    fn version(&self) -> u32 {
        1
    }
    fn title(&self) -> &'static str {
        "Run Command"
    }
    fn description(&self) -> &'static str {
        "Runs a shell command in the workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" }
            },
            "required": ["command"],
            "additionalProperties": false
        })
    }

    fn class(&self) -> ToolClass {
        ToolClass::Execute
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Elevated
    }

    fn modes(&self) -> Vec<RunMode> {
        vec![RunMode::Ask, RunMode::Plan, RunMode::Agent, RunMode::Build]
    }

    fn idempotent(&self) -> bool {
        false
    }
    fn mutates(&self) -> bool {
        true
    }
    fn checkpointed(&self) -> bool {
        false
    }
    fn timeout_ms(&self) -> u32 {
        60_000
    }
    fn result_cap_tokens(&self) -> u32 {
        30_000
    }

    async fn execute(
        &self,
        args: Value,
        workspace: Option<&Workspace>,
    ) -> Result<Value, ToolError> {
        let ws = workspace.ok_or_else(|| ToolError::Execution("Workspace required".into()))?;
        let cmd_str = args["command"]
            .as_str()
            .ok_or_else(|| ToolError::Validation("command must be string".into()))?;

        let cmd = build_command(cmd_str, ws)?;
        let cap_bytes = 2 * 1024 * 1024;
        let supervisor =
            ProcessSupervisor::new(Duration::from_millis(u64::from(self.timeout_ms())), cap_bytes);
        let (status, stdout, stderr) = supervisor.spawn_and_wait(cmd).await.map_err(|e| {
            ToolError::from(TaxonomyError::ExecutionFailed(format!("Execution failed: {e}")))
        })?;

        let truncated = stdout.len() >= cap_bytes || stderr.len() >= cap_bytes;

        Ok(json!({
            "exit_code": status.code().unwrap_or(-1),
            "stdout": stdout,
            "stderr": stderr,
            "truncated": truncated
        }))
    }
}
