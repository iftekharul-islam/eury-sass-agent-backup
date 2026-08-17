use crate::engine::AgentEngine;
use agent_types::capabilities::{EngineCapabilities, ToolDefinition};
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::{RunOutcome, RunRequest, RunStatus};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct StubProvider;

impl StubProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for StubProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentEngine for StubProvider {
    async fn run_stream(
        &self,
        request: RunRequest,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError> {
        let run_id = request.run_id;

        let _ = tx.send(AgentEvent::Meta { run_id, status: "streaming".to_string() }).await;

        let words = vec!["Hello", " from", " the", " stub", " provider!"];

        for word in words {
            tokio::select! {
                () = cancel.cancelled() => {
                    let _ = tx.send(AgentEvent::Meta {
                        run_id,
                        status: "cancelled".to_string(),
                    }).await;
                    return Err(AgentError::RunCancelled);
                }
                () = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                    let _ = tx.send(AgentEvent::TextDelta {
                        text: word.to_string(),
                    }).await;
                }
            }
        }

        let _ =
            tx.send(AgentEvent::RunComplete { run_id, stop_reason: "end_turn".to_string() }).await;

        Ok(RunOutcome {
            run_id,
            status: RunStatus::Complete,
            total_tokens: 10,
            total_cost_micros: 30, // 10 tokens * 3 micros
        })
    }

    fn tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![]
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_streaming: true,
            supports_thinking: false,
            supports_subagents: false,
        }
    }
}
