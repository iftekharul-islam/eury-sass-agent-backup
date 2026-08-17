use crate::engine::AgentEngine;
use agent_types::capabilities::{EngineCapabilities, ToolDefinition};
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::{RunOutcome, RunRequest, RunStatus};
use async_trait::async_trait;
use cersei_agent::events::AgentEvent as CerseiEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use agent_policy::engine::PolicyEngine;
use agent_tools::ToolRegistry;

pub struct CerseiAdapter {
    pub tools: ToolRegistry,
    pub policy: PolicyEngine,
}

impl CerseiAdapter {
    pub fn new(tools: ToolRegistry, policy: PolicyEngine) -> Self {
        Self { tools, policy }
    }

    /// Maps a Cersei event to our internal stable event.
    #[must_use]
    pub fn map_event(event: CerseiEvent, run_id: uuid::Uuid) -> Option<AgentEvent> {
        match event {
            CerseiEvent::TextDelta(text) => Some(AgentEvent::TextDelta { text }),
            CerseiEvent::ThinkingDelta(text) => Some(AgentEvent::ThinkingDelta { text }),
            CerseiEvent::ToolStart { id, name, input, .. } => {
                Some(AgentEvent::ToolStart { tool_call_id: id, name, arguments: input })
            }
            CerseiEvent::ToolEnd { id, result, .. } => Some(AgentEvent::ToolEnd {
                tool_call_id: id,
                result: serde_json::Value::String(result),
            }),
            CerseiEvent::PermissionRequired(req) => Some(AgentEvent::ApprovalRequired {
                tool_call_id: req.id,
                name: req.tool_name,
                arguments: req.tool_input,
                justification: Some(req.description),
            }),
            CerseiEvent::TurnComplete { .. } => {
                Some(AgentEvent::TurnComplete { turn_id: uuid::Uuid::new_v4() })
            }
            CerseiEvent::TokenWarning { pct_used, .. } => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let tokens = (pct_used.clamp(0.0, 1.0) * 2000.0).round() as u32;
                Some(AgentEvent::ContextWarning {
                    tokens,
                    limit: 200_000,
                    message: format!("Token limit nearing capacity: {:.1}%", pct_used * 100.0),
                })
            }
            CerseiEvent::CompactStart { .. } => Some(AgentEvent::CompactStart { before_tokens: 0 }),
            CerseiEvent::CompactEnd { tokens_freed, .. } => Some(AgentEvent::CompactEnd {
                after_tokens: u32::try_from(tokens_freed).unwrap_or(u32::MAX),
            }),
            CerseiEvent::SubAgentSpawned { agent_id, prompt, .. } => {
                let parsed_id =
                    uuid::Uuid::parse_str(&agent_id).unwrap_or_else(|_| uuid::Uuid::new_v4());
                Some(AgentEvent::SubagentSpawned {
                    subagent_id: parsed_id,
                    role: "subagent".to_string(),
                    task: prompt,
                })
            }
            CerseiEvent::SubAgentComplete { agent_id, result, .. } => {
                let parsed_id =
                    uuid::Uuid::parse_str(&agent_id).unwrap_or_else(|_| uuid::Uuid::new_v4());
                Some(AgentEvent::SubagentComplete {
                    subagent_id: parsed_id,
                    result: format!("{result:?}"),
                })
            }
            CerseiEvent::CostUpdate { input_tokens, output_tokens, cumulative_cost, .. } => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let cost_usd_micros = (cumulative_cost.max(0.0) * 1_000_000.0).round() as u64;
                Some(AgentEvent::CostUpdate {
                    tokens_prompt: u32::try_from(input_tokens).unwrap_or(u32::MAX),
                    tokens_completion: u32::try_from(output_tokens).unwrap_or(u32::MAX),
                    cost_usd_micros,
                })
            }
            CerseiEvent::Complete(stop_reason) => {
                Some(AgentEvent::RunComplete { run_id, stop_reason: format!("{stop_reason:?}") })
            }
            CerseiEvent::Error(err) => Some(AgentEvent::RunError {
                run_id,
                code: "CERSEI_ERROR".to_string(),
                message: format!("{err:?}"),
            }),
            _ => None,
        }
    }
}

#[async_trait]
impl AgentEngine for CerseiAdapter {
    async fn run_stream(
        &self,
        request: RunRequest,
        tx: mpsc::Sender<AgentEvent>,
        _cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError> {
        let run_id = request.run_id;

        let _ = tx.send(AgentEvent::Meta { run_id, status: "ready".to_string() }).await;

        // In a real implementation, we would listen to Cersei events.
        // For the stub implementation here, let's simulate a tool call execution
        // that goes through the PolicyEngine and requires approval.

        let tool_name = "write_file";
        let args = serde_json::json!({
            "path": "test.txt",
            "content": "hello"
        });

        // 1. Get tool definition
        if let Some(tool) = self.tools.get_tool(tool_name) {
            // Map tool class
            let mapped_class = match tool.class() {
                agent_tools::registry::ToolClass::Read => agent_policy::schema::ToolClass::Read,
                agent_tools::registry::ToolClass::Write => agent_policy::schema::ToolClass::Write,
                agent_tools::registry::ToolClass::Execute => {
                    agent_policy::schema::ToolClass::Execute
                }
                agent_tools::registry::ToolClass::Network => {
                    agent_policy::schema::ToolClass::Network
                }
                agent_tools::registry::ToolClass::Mcp => agent_policy::schema::ToolClass::Mcp,
                agent_tools::registry::ToolClass::WriteOutsideWorkspace => {
                    agent_policy::schema::ToolClass::WriteOutsideWorkspace
                }
            };

            // 2. Evaluate policy
            let decision = self.policy.evaluate(
                &mapped_class,
                tool_name,
                &args,
                None, // Provide mock workspace
                &request.mode,
                Some(&run_id.to_string()),
            );

            match decision {
                agent_policy::schema::Decision::Allow => {
                    // Exec tool directly
                    let _ = tx
                        .send(AgentEvent::ToolStart {
                            tool_call_id: "call_1".to_string(),
                            name: tool_name.to_string(),
                            arguments: args.clone(),
                        })
                        .await;

                    if let Ok(res) = tool.execute(args.clone(), None).await {
                        let _ = tx
                            .send(AgentEvent::ToolEnd {
                                tool_call_id: "call_1".to_string(),
                                result: res,
                            })
                            .await;
                    }
                }
                agent_policy::schema::Decision::NeedsApproval => {
                    // Emit ApprovalRequired and pause execution
                    let _ = tx
                        .send(AgentEvent::ApprovalRequired {
                            tool_call_id: "call_1".to_string(),
                            name: tool_name.to_string(),
                            arguments: args.clone(),
                            justification: Some("Need to write a file for the plan.".to_string()),
                        })
                        .await;

                    // Here we would await IPC response in reality using a oneshot channel
                }
                agent_policy::schema::Decision::Deny => {
                    let _ = tx
                        .send(AgentEvent::RunError {
                            run_id,
                            code: "POLICY_DENIED".to_string(),
                            message: "Action denied by policy".to_string(),
                        })
                        .await;
                }
            }
        }

        Ok(RunOutcome {
            run_id,
            status: RunStatus::Complete,
            total_tokens: 0,
            total_cost_micros: 0,
        })
    }

    fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.get_definitions_for_mode(&agent_types::requests::RunMode::Agent)
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_streaming: true,
            supports_thinking: true,
            supports_subagents: true,
        }
    }
}
