use agent_types::capabilities::{EngineCapabilities, ToolDefinition};
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::{RunOutcome, RunRequest};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait AgentEngine: Send + Sync {
    /// Start a streaming run. Events delivered via mpsc.
    async fn run_stream(
        &self,
        request: RunRequest,
        events: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError>;

    /// List tools available for current mode + policy.
    fn tool_definitions(&self) -> Vec<ToolDefinition>;

    /// Engine capabilities for UI.
    fn capabilities(&self) -> EngineCapabilities;
}
