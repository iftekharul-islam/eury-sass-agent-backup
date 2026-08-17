use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

use agent_sandbox::workspace::Workspace;
use agent_types::capabilities::ToolDefinition;
use agent_types::requests::RunMode;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Execution failed: {0}")]
    Execution(String),
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Tool not allowed in mode: {0}")]
    NotAllowedInMode(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolClass {
    Read,
    Write,
    Execute,
    Network,
    Mcp,
    WriteOutsideWorkspace,
}

impl ToolClass {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Execute => "execute",
            Self::Network => "network",
            Self::Mcp => "mcp",
            Self::WriteOutsideWorkspace => "write_outside_workspace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    Elevated,
    Critical,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> u32;
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;

    fn class(&self) -> ToolClass;
    fn risk(&self) -> RiskLevel;

    /// Empty means all modes
    fn modes(&self) -> Vec<RunMode>;
    fn idempotent(&self) -> bool;
    fn mutates(&self) -> bool;
    fn checkpointed(&self) -> bool;

    fn timeout_ms(&self) -> u32;
    fn result_cap_tokens(&self) -> u32;

    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters_schema: self.input_schema(),
        }
    }

    async fn execute(&self, args: Value, workspace: Option<&Workspace>)
    -> Result<Value, ToolError>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    // `&Box<dyn Tool>` (rather than `&dyn Tool`) is kept here because downstream
    // callers outside this crate rely on the returned reference supporting
    // `Box::as_ref`; narrowing the return type is an API change out of scope
    // for this lint-only pass.
    #[must_use]
    #[allow(clippy::borrowed_box)]
    pub fn get_tool(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }

    #[must_use]
    pub fn get_definitions_for_mode(&self, mode: &RunMode) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .filter(|t| {
                let allowed_modes = t.modes();
                allowed_modes.is_empty() || allowed_modes.contains(mode)
            })
            .map(|t| t.to_definition())
            .collect()
    }

    /// # Errors
    ///
    /// Returns [`ToolError::NotFound`] if no tool is registered under `name`,
    /// [`ToolError::NotAllowedInMode`] if the tool is not permitted in `mode`,
    /// [`ToolError::Validation`] if the tool's input schema fails to compile or
    /// `args` does not validate against it, and otherwise propagates whatever
    /// error the tool's own `execute` returns.
    pub async fn execute_tool(
        &self,
        name: &str,
        args: Value,
        mode: &RunMode,
        workspace: Option<&Workspace>,
    ) -> Result<Value, ToolError> {
        let tool = self.tools.get(name).ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        let allowed_modes = tool.modes();
        if !allowed_modes.is_empty() && !allowed_modes.contains(mode) {
            return Err(ToolError::NotAllowedInMode(name.to_string()));
        }

        // JS Schema validation would happen here
        let schema = tool.input_schema();
        let compiled =
            jsonschema::validator_for(&schema).map_err(|e| ToolError::Validation(e.to_string()))?;

        if let Err(err) = compiled.validate(&args) {
            use std::fmt::Write;
            let mut err_msg = String::from("Validation error:");
            let _ = write!(err_msg, "\n- {err}");
            return Err(ToolError::Validation(err_msg));
        }

        tool.execute(args, workspace).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
