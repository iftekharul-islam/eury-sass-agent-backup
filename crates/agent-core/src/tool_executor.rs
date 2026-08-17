//! Local tool execution for the AI SDK desktop path.
//!
//! Policy, approvals, sandboxing, and streaming stdout all stay in Rust;
//! model inference runs in TypeScript via the Vercel AI SDK.

use crate::approval::{ApprovalOutcome, SharedApprovalWaiter};
use agent_policy::engine::PolicyEngine;
use agent_policy::presets::standard_preset;
use agent_policy::schema::{Decision, GrantScope, ToolClass};
use agent_policy::store::GrantStore;
use agent_sandbox::workspace::{TrustStore, Workspace};
use agent_tools::registry::{ToolClass as ToolsToolClass, ToolRegistry};
use agent_tools::registry_factory::default_registry;
use agent_tools::shell::run::execute_run_command_streaming;
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::RunMode;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const APPROVAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

struct ToolStreamContext<'a> {
    tx: &'a mpsc::Sender<AgentEvent>,
    tool_call_id: &'a str,
}

pub struct ToolExecuteRequest {
    pub run_id: Uuid,
    pub conversation_id: Uuid,
    pub tool_call_id: String,
    pub name: String,
    pub arguments: Value,
    pub mode: RunMode,
    pub workspace_root: Option<std::path::PathBuf>,
}

pub struct ToolExecutor {
    tools: ToolRegistry,
    policy: PolicyEngine,
    approvals: SharedApprovalWaiter,
    trust: Arc<TrustStore>,
}

impl ToolExecutor {
    /// # Errors
    ///
    /// Returns [`AgentError::Internal`] if policy stores cannot be opened.
    pub fn new(approvals: SharedApprovalWaiter, trust: Arc<TrustStore>) -> Result<Self, AgentError> {
        let grant_store = GrantStore::new_in_memory()
            .map_err(|e| AgentError::Internal(format!("failed to open grant store: {e}")))?;
        let policy = PolicyEngine::new(standard_preset(), grant_store)
            .map_err(|e| AgentError::Internal(format!("failed to open audit log: {e}")))?;
        Ok(Self {
            tools: default_registry(),
            policy,
            approvals,
            trust,
        })
    }

    pub fn workspace_from_root(&self, root: &std::path::Path) -> Workspace {
        Workspace {
            id: Uuid::new_v4(),
            root_path: root.to_path_buf(),
            trust_state: self.trust.trust_state(root),
        }
    }

    /// Executes one tool call with policy, approval, and live output streaming.
    pub async fn execute(
        &self,
        request: ToolExecuteRequest,
        tx: &mpsc::Sender<AgentEvent>,
        cancel: &CancellationToken,
    ) -> Value {
        let workspace = request
            .workspace_root
            .as_ref()
            .map(|root| self.workspace_from_root(root));

        let Some(tool) = self.tools.get_tool(&request.name) else {
            return serde_json::json!({ "error": "unknown tool" });
        };

        let mapped = Self::map_tool_class(tool.class());
        let decision = self.policy.evaluate(
            &mapped,
            &request.name,
            &request.arguments,
            workspace.as_ref(),
            &request.mode,
            Some(&request.run_id.to_string()),
        );

        match decision {
            Decision::Deny => {
                let reason = self.policy.explain_denial(
                    &mapped,
                    &request.name,
                    &request.arguments,
                    workspace.as_ref(),
                );
                return Self::report_denied(
                    &request.tool_call_id,
                    &request.name,
                    &request.arguments,
                    &reason,
                    tx,
                )
                .await;
            }
            Decision::NeedsApproval => {
                let auto_allow = matches!(
                    request.mode,
                    RunMode::Agent | RunMode::Build | RunMode::Ask
                )
                    && workspace.as_ref().is_some_and(|ws| ws.is_trusted())
                    && match request.mode {
                        RunMode::Ask => matches!(mapped, ToolClass::Execute),
                        _ => matches!(mapped, ToolClass::Execute | ToolClass::Write),
                    };

                if !auto_allow {
                    let _ = tx
                        .send(AgentEvent::ApprovalRequired {
                            tool_call_id: request.tool_call_id.clone(),
                            name: request.name.clone(),
                            arguments: request.arguments.clone(),
                            justification: Some(format!("Allow {}?", request.name)),
                        })
                        .await;

                    let rx = self.approvals.register(request.tool_call_id.clone());
                    let outcome = tokio::select! {
                        () = cancel.cancelled() => ApprovalOutcome::Denied,
                        res = rx => res.unwrap_or(ApprovalOutcome::Denied),
                        () = tokio::time::sleep(APPROVAL_TIMEOUT) => ApprovalOutcome::Denied,
                    };

                    if outcome == ApprovalOutcome::Denied {
                        return serde_json::json!({ "error": "denied by user" });
                    }

                    if outcome == ApprovalOutcome::Session {
                        let _ = self.policy.record_grant(
                            mapped,
                            &request.name,
                            &request.arguments,
                            GrantScope::Session,
                            None,
                        );
                    }
                }
            }
            Decision::Allow => {}
        }

        let _ = tx
            .send(AgentEvent::ToolStart {
                tool_call_id: request.tool_call_id.clone(),
                name: request.name.clone(),
                arguments: request.arguments.clone(),
            })
            .await;

        let result = self
            .execute_tool_call(
                tool.as_ref(),
                mapped,
                &request.arguments,
                workspace.as_ref(),
                cancel,
                Some(ToolStreamContext {
                    tx,
                    tool_call_id: &request.tool_call_id,
                }),
            )
            .await;

        let _ = tx
            .send(AgentEvent::ToolEnd {
                tool_call_id: request.tool_call_id.clone(),
                result: result.clone(),
            })
            .await;

        result
    }

    fn map_tool_class(class: ToolsToolClass) -> ToolClass {
        match class {
            ToolsToolClass::Read => ToolClass::Read,
            ToolsToolClass::Write => ToolClass::Write,
            ToolsToolClass::Execute => ToolClass::Execute,
            ToolsToolClass::Network => ToolClass::Network,
            ToolsToolClass::Mcp => ToolClass::Mcp,
            ToolsToolClass::WriteOutsideWorkspace => ToolClass::WriteOutsideWorkspace,
        }
    }

    async fn report_denied(
        tool_call_id: &str,
        name: &str,
        args: &Value,
        reason: &str,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> Value {
        let _ = tx
            .send(AgentEvent::ToolStart {
                tool_call_id: tool_call_id.to_string(),
                name: name.to_string(),
                arguments: args.clone(),
            })
            .await;
        let result = serde_json::json!({ "ok": false, "error": reason });
        let _ = tx
            .send(AgentEvent::ToolEnd {
                tool_call_id: tool_call_id.to_string(),
                result: result.clone(),
            })
            .await;
        result
    }

    async fn execute_tool_call(
        &self,
        tool: &dyn agent_tools::registry::Tool,
        tool_class: ToolClass,
        args: &Value,
        workspace: Option<&Workspace>,
        cancel: &CancellationToken,
        stream: Option<ToolStreamContext<'_>>,
    ) -> Value {
        let cap_bytes = 2 * 1024 * 1024;

        let (cap, timeout_message) = if tool_class == ToolClass::Execute {
            let max_runtime_seconds = self.policy.policy.commands.max_runtime_seconds;
            (
                std::time::Duration::from_secs(u64::from(max_runtime_seconds)),
                format!("command exceeded policy max_runtime_seconds ({max_runtime_seconds}s)"),
            )
        } else {
            let timeout_ms = tool.timeout_ms();
            (
                std::time::Duration::from_millis(u64::from(timeout_ms)),
                format!("{} exceeded its {timeout_ms}ms timeout", tool.name()),
            )
        };

        let execution = async {
            if tool.name() == "run_command" {
                if let (Some(ctx), Some(ws)) = (stream, workspace) {
                    let tx = ctx.tx.clone();
                    let tool_call_id = ctx.tool_call_id.to_string();
                    return execute_run_command_streaming(
                        args.clone(),
                        ws,
                        cap,
                        cap_bytes,
                        move |stream_name, text| {
                            let _ = tx.try_send(AgentEvent::ToolOutputDelta {
                                tool_call_id: tool_call_id.clone(),
                                stream: stream_name.to_string(),
                                text: text.to_string(),
                            });
                        },
                    )
                    .await;
                }
            }
            tool.execute(args.clone(), workspace).await
        };

        tokio::select! {
            () = cancel.cancelled() => serde_json::json!({ "error": "cancelled" }),
            res = tokio::time::timeout(cap, execution) => match res {
                Ok(Ok(value)) => value,
                Ok(Err(e)) => serde_json::json!({ "error": e.to_string() }),
                Err(_) => serde_json::json!({ "error": timeout_message }),
            },
        }
    }
}
