use crate::approval::{ApprovalOutcome, SharedApprovalWaiter};
use crate::assembly::PromptAssembler;
use crate::engine::AgentEngine;
use crate::providers::gateway::EuryGatewayProvider;
use crate::tool_calls;
use agent_policy::engine::PolicyEngine;
use agent_policy::presets::standard_preset;
use agent_policy::schema::{Decision, GrantScope, ToolClass};
use agent_policy::store::GrantStore;
use agent_sandbox::workspace::{TrustStore, Workspace};
use agent_tools::registry::{ToolClass as ToolsToolClass, ToolRegistry};
use agent_tools::registry_factory::default_registry;
use agent_tools::shell::run::execute_run_command_streaming;
use agent_types::capabilities::{EngineCapabilities, ToolDefinition};
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::{HistoryMessage, RunMode, RunOutcome, RunRequest, RunStatus};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// How many assistant→tool→assistant rounds a single run may take.
///
/// A real refactor routinely needs to read several files before editing, so
/// this has to be well above a handful; three (the previous value) cut most
/// multi-file work off mid-task. It is still bounded so a looping model can't
/// run unattended forever.
const MAX_TOOL_ROUNDS: usize = 16;

/// How long the run may wait for a human approval before auto-denying.
/// Matches the product spec: no auto-approve, but also no indefinite hang.
const APPROVAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

/// Builds the user prompt fed back into the model after a tool round.
fn build_tool_continuation_prompt(user_request: &str, tool_results: &str) -> String {
    if tool_results_indicate_failure(tool_results) {
        format!(
            "{tool_results}\n\nThe last command failed. The user's request was: \"{user_request}\". \
             Read stdout/stderr above. If dependencies are missing (UNRESOLVED_IMPORT, cannot \
             resolve package, or no node_modules), run `npm install` or `pnpm install` first, \
             then retry the dev/start command. Emit ```tool_call``` fences to fix the problem — \
             do not tell the user to run commands manually or say tools did not return a result."
        )
    } else if user_request_needs_action(user_request) {
        format!(
            "{tool_results}\n\nContinue the task: \"{user_request}\". \
             The work is NOT finished until the app is running or the request is fully complete. \
             Use the results above — if you still need package.json, dependency install, or a dev \
             server, emit ```tool_call``` fences NOW. Do not only describe what you plan to do; \
             run the next step. When everything is done, summarize what you ran and the outcome."
        )
    } else {
        format!(
            "{tool_results}\n\nAnswer the user's question: \"{user_request}\". \
             Use the tool results above — state the outcome clearly in plain language \
             (for example the version number from stdout). Do not ask the user to repeat \
             the task or say no task was given."
        )
    }
}

fn user_request_needs_action(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    [
        "run the",
        "run that",
        "run this",
        "run my",
        "run it",
        "run app",
        "start the",
        "start dev",
        "start server",
        "launch",
        "create ",
        "scaffold",
        "build ",
        "install",
        "fix ",
        "deploy",
        "write ",
        "add ",
        "implement",
        "generate",
        "make ",
        "setup",
        "set up",
        "bootstrap",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
        || lower.starts_with("run ")
}

fn tool_results_indicate_failure(tool_results: &str) -> bool {
    let lower = tool_results.to_ascii_lowercase();
    lower.contains("\"error\"")
        || lower.contains("unresolved_import")
        || lower.contains("could not resolve")
        || lower.contains("module not found")
        || lower.contains("enoent")
        || (lower.contains("\"exit_code\"") && !lower.contains("\"exit_code\":0"))
}

/// One tool invocation this run — used to decide whether an action task still
/// needs more work before the loop may stop.
#[derive(Debug, Clone)]
struct ExecutedStep {
    name: String,
    arguments: Value,
    result: Value,
}

fn task_marked_complete(assistant_text: &str) -> bool {
    assistant_text.to_uppercase().contains("TASK_COMPLETE")
}

fn is_run_app_request(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    (lower.contains("run") || lower.contains("start") || lower.contains("launch"))
        && (lower.contains("app") || lower.contains("dev") || lower.contains("server") || lower.contains("locally"))
}

fn is_scaffold_request(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    ["create", "scaffold", "generate", "make ", "bootstrap", "set up", "setup"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn output_shows_dev_server(result: &Value) -> bool {
    let stdout = result.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
    let stderr = result.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
    let combined = format!("{stdout}{stderr}");
    combined.contains("localhost")
        || combined.contains("127.0.0.1")
        || combined.contains("Local:")
}

fn action_task_satisfied(user_request: &str, executed: &[ExecutedStep]) -> bool {
    if executed.is_empty() {
        return false;
    }
    let lower = user_request.to_ascii_lowercase();
    if is_run_app_request(&lower) {
        if executed.iter().any(|step| step.name == "run_command" && output_shows_dev_server(&step.result))
        {
            return true;
        }
        let commands = executed
            .iter()
            .filter(|step| step.name == "run_command")
            .filter(|step| step.result.get("error").is_none())
            .count();
        return commands >= 2;
    }
    if is_scaffold_request(&lower) {
        return executed.iter().any(|step| step.name == "write_file");
    }
    false
}

fn assistant_looks_incomplete(assistant_text: &str) -> bool {
    let lower = assistant_text.to_ascii_lowercase();
    [
        "i will",
        "i'll",
        "going to",
        "let me",
        "next,",
        "then i",
        "need to",
        "first,",
        "install",
        "start the",
        "run the",
        "inspect",
        "check the",
        "read the",
        "look at",
        "about to",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

/// Keep looping when the model stops with planning text but the action task is
/// clearly not finished yet (vibe-coding style: understand → do → do → done).
fn should_auto_continue_action_task(
    user_request: &str,
    assistant_text: &str,
    round: usize,
    executed: &[ExecutedStep],
) -> bool {
    if !user_request_needs_action(user_request) {
        return false;
    }
    if round + 1 >= MAX_TOOL_ROUNDS {
        return false;
    }
    if task_marked_complete(assistant_text) {
        return false;
    }
    if action_task_satisfied(user_request, executed) {
        return false;
    }
    if assistant_looks_incomplete(assistant_text) {
        return true;
    }
    // Stopped after only exploration (list_dir / read_file) on an action task.
    !executed.is_empty()
        && executed.iter().all(|step| matches!(step.name.as_str(), "list_dir" | "read_file" | "glob" | "grep"))
}

fn build_auto_continue_prompt(
    user_request: &str,
    assistant_text: &str,
    executed: &[ExecutedStep],
) -> String {
    let steps_done = if executed.is_empty() {
        "(none yet)".to_string()
    } else {
        executed
            .iter()
            .map(|step| format!("- {} {:?}", step.name, step.arguments))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "You replied with text but called no tools:\n\"{}\"\n\n\
         The user's task is NOT finished: \"{}\".\n\
         Steps completed so far:\n{steps_done}\n\n\
         Continue immediately — emit ```tool_call``` for the NEXT step only (read package.json, \
         npm/pnpm install, npm/pnpm run dev, write_file, etc.). Do not repeat finished steps. \
         When the task is truly done, end your final message with TASK_COMPLETE on its own line.",
        assistant_text.trim(),
        user_request,
    )
}

/// Lets [`AgentLoopEngine::execute_tool_call`] stream `run_command` stdout/stderr to the UI.
struct ToolStreamContext<'a> {
    tx: &'a mpsc::Sender<AgentEvent>,
    tool_call_id: &'a str,
}

/// Per-round state for [`AgentLoopEngine::run_tool_round`], grouped so the
/// signature stays readable as the loop learns more about the run.
struct ToolRound<'a> {
    round: usize,
    request: &'a RunRequest,
    workspace: Option<&'a Workspace>,
    /// Calls that already failed this run; an identical retry is refused.
    failed_calls: &'a mut HashSet<String>,
    executed_steps: &'a mut Vec<ExecutedStep>,
}

pub struct AgentLoopEngine {
    gateway: EuryGatewayProvider,
    tools: ToolRegistry,
    policy: PolicyEngine,
    approvals: SharedApprovalWaiter,
    trust: Arc<TrustStore>,
}

impl AgentLoopEngine {
    /// # Errors
    ///
    /// Returns [`AgentError::Internal`] if the in-memory grant store or audit
    /// log cannot be opened or initialized.
    pub fn new(
        approvals: SharedApprovalWaiter,
        trust: Arc<TrustStore>,
    ) -> Result<Self, AgentError> {
        let grant_store = GrantStore::new_in_memory()
            .map_err(|e| AgentError::Internal(format!("failed to open grant store: {e}")))?;
        let policy = PolicyEngine::new(standard_preset(), grant_store)
            .map_err(|e| AgentError::Internal(format!("failed to open audit log: {e}")))?;
        Ok(Self {
            gateway: EuryGatewayProvider::new(),
            tools: default_registry(),
            policy,
            approvals,
            trust,
        })
    }

    /// Same engine, pointed at an explicit gateway URL.
    ///
    /// Exists so the loop can be exercised end to end against a scripted
    /// gateway — the real prompt, extractor, policy, tools and filesystem,
    /// with only the model's replies stubbed.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError::Internal`] if the grant store or audit log cannot
    /// be opened.
    pub fn with_gateway_url(
        approvals: SharedApprovalWaiter,
        trust: Arc<TrustStore>,
        gateway_url: &str,
    ) -> Result<Self, AgentError> {
        let mut engine = Self::new(approvals, trust)?;
        engine.gateway = EuryGatewayProvider::with_gateway_url(gateway_url);
        Ok(engine)
    }

    /// Which modes run the tool loop.
    ///
    /// `Ask` is included because the registry grants it read-class tools
    /// (`read_file`, `list_dir`, `glob`, `grep`) — leaving it out meant the
    /// model was told it had no tools while answering questions about code it
    /// was perfectly entitled to read. `Chat` stays out: it is conversation
    /// only, by definition.
    fn should_use_agent_loop(mode: &RunMode) -> bool {
        matches!(mode, RunMode::Agent | RunMode::Build | RunMode::Plan | RunMode::Ask)
    }

    /// Builds the `Workspace` for this run, reading its trust state from the
    /// shared [`TrustStore`] rather than assuming trust. A root the user has
    /// not explicitly trusted comes back `Untrusted`, which the policy engine
    /// enforces as read-only.
    fn workspace_from_request(&self, request: &RunRequest) -> Option<Workspace> {
        let root = request.workspace_root.as_ref()?;
        Some(Workspace {
            id: Uuid::new_v4(),
            root_path: root.clone(),
            trust_state: self.trust.trust_state(root),
        })
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

    async fn run_agent_loop(
        &self,
        request: RunRequest,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError> {
        let run_id = request.run_id;
        let workspace = self.workspace_from_request(&request);
        let tool_defs =
            PromptAssembler::tools_for_run(&self.tools, &request.mode, workspace.as_ref());
        // The model only knows it has tools and a workspace because this says
        // so — without it the agent loop degrades to a plain chat turn.
        let system_prompt = PromptAssembler::system_prompt(
            &request,
            workspace.as_ref(),
            &tool_defs,
        );
        let user_request = request.prompt.clone();
        let mut loop_history = request.history.clone();
        let mut prompt = user_request.clone();
        let mut total_tokens = 0u32;
        let mut total_cost = 0u64;
        // Calls that already failed, so an identical retry can be cut off
        // instead of burning every remaining round on the same mistake.
        let mut failed_calls: HashSet<String> = HashSet::new();
        let mut executed_steps: Vec<ExecutedStep> = Vec::new();

        let _ = tx.send(AgentEvent::Meta { run_id, status: "agent_loop".to_string() }).await;

        for round in 0..MAX_TOOL_ROUNDS {
            if cancel.is_cancelled() {
                return Err(AgentError::RunCancelled);
            }

            let mut round_request = request.clone();
            round_request.history = loop_history.clone();
            round_request.prompt = prompt.clone();

            let (raw_text, typed_calls, outcome) =
                self.gateway
                    .stream_collecting(&round_request, Some(&system_prompt), &tx, &cancel)
                    .await?;

            total_tokens = outcome.total_tokens;
            total_cost = outcome.total_cost_micros;

            // Typed `tool_call` stream events are the documented contract and
            // take precedence — but no server in this stack emits them yet, so
            // fall back to recovering calls from the assistant's own text,
            // which is what the prompts actually instruct the model to produce.
            let (assistant_text, tool_calls): (String, Vec<(String, Value)>) =
                if typed_calls.is_empty() {
                let (cleaned, parsed) = tool_calls::extract_tool_calls(&raw_text, &|name| {
                    self.tools.get_tool(name).is_some()
                });
                (
                    cleaned,
                    parsed.into_iter().map(|c| (c.name, c.arguments)).collect(),
                )
            } else {
                (
                    raw_text,
                    typed_calls
                        .into_iter()
                        .map(|(name, args)| tool_calls::repair_tool_call(&name, args))
                        .collect(),
                )
            };

            if tool_calls.is_empty() {
                if should_auto_continue_action_task(
                    &user_request,
                    &assistant_text,
                    round,
                    &executed_steps,
                ) {
                    if round == 0 {
                        loop_history.push(HistoryMessage {
                            role: "user".to_string(),
                            content: user_request.clone(),
                        });
                    }
                    if !assistant_text.trim().is_empty() {
                        loop_history.push(HistoryMessage {
                            role: "assistant".to_string(),
                            content: assistant_text.trim().to_string(),
                        });
                    }
                    prompt = build_auto_continue_prompt(
                        &user_request,
                        &assistant_text,
                        &executed_steps,
                    );
                    continue;
                }

                // The turn is over. Without this the run ends server-side but
                // the UI never hears a terminal event, so it waits on a run
                // that already finished.
                let _ = tx
                    .send(AgentEvent::RunComplete { run_id, stop_reason: "stop".to_string() })
                    .await;
                return Ok(RunOutcome {
                    run_id,
                    status: RunStatus::Complete,
                    total_tokens,
                    total_cost_micros: total_cost,
                });
            }

            let tool_results = self
                .run_tool_round(
                    &tool_calls,
                    &mut ToolRound {
                        round,
                        request: &request,
                        workspace: workspace.as_ref(),
                        failed_calls: &mut failed_calls,
                        executed_steps: &mut executed_steps,
                    },
                    &tx,
                    &cancel,
                )
                .await?;

            if round == 0 {
                loop_history.push(HistoryMessage {
                    role: "user".to_string(),
                    content: user_request.clone(),
                });
            }
            if !assistant_text.trim().is_empty() {
                loop_history.push(HistoryMessage {
                    role: "assistant".to_string(),
                    content: assistant_text.trim().to_string(),
                });
            }

            prompt = build_tool_continuation_prompt(&user_request, &tool_results);
        }

        let _ = tx
            .send(AgentEvent::RunComplete { run_id, stop_reason: "max_rounds".to_string() })
            .await;

        Ok(RunOutcome {
            run_id,
            status: RunStatus::Complete,
            total_tokens,
            total_cost_micros: total_cost,
        })
    }

    /// Surfaces a refused call as a finished tool card and returns the
    /// `[tool_result]` block telling the model why.
    ///
    /// The tool does not run — but the run continues. Aborting it left the user
    /// with a red "denied by policy" and no idea why or what to do; feeding the
    /// reason back lets the model explain the remedy in the same turn.
    async fn report_denied(
        tool_call_id: &str,
        name: &str,
        args: &Value,
        workspace: Option<&Workspace>,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> String {
        let reason = Self::denial_reason(name, workspace);
        let _ = tx
            .send(AgentEvent::ToolStart {
                tool_call_id: tool_call_id.to_string(),
                name: name.to_string(),
                arguments: args.clone(),
            })
            .await;
        let _ = tx
            .send(AgentEvent::ToolEnd {
                tool_call_id: tool_call_id.to_string(),
                result: serde_json::json!({ "ok": false, "error": reason }),
            })
            .await;
        tool_calls::format_tool_result(name, &serde_json::json!({ "error": reason }).to_string())
    }

    /// The answer to a call this run already tried and lost.
    fn already_failed(name: &str) -> String {
        tool_calls::format_tool_result(
            name,
            "{\"error\":\"This exact call already failed. Do not repeat it — change the \
             arguments or use a different tool, or tell the user what is blocking you.\"}",
        )
    }

    /// Why a tool call was refused, in terms the user can act on.
    fn denial_reason(name: &str, workspace: Option<&Workspace>) -> String {
        if workspace.is_some_and(|ws| !ws.is_trusted()) {
            return format!(
                "{name} was blocked: this project is not trusted, so Eury can only read it. \
                 Trust the project in the app to allow writes and commands."
            );
        }
        format!("{name} was blocked by the workspace policy.")
    }

    /// Executes the tool calls extracted from a single assistant turn,
    /// applying policy decisions and (when required) waiting for user
    /// approval, and returns the accumulated `[tool_result ...]` text block
    /// to feed back into the next round's prompt.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError::SecurityViolation`] if the policy engine denies
    /// one of the requested tool calls.
    async fn run_tool_round(
        &self,
        tool_calls: &[(String, Value)],
        ctx: &mut ToolRound<'_>,
        tx: &mpsc::Sender<AgentEvent>,
        cancel: &CancellationToken,
    ) -> Result<String, AgentError> {
        use std::fmt::Write as _;

        let ToolRound { round, request, workspace, failed_calls, executed_steps } = ctx;
        let round = *round;
        let workspace = *workspace;
        let run_id = request.run_id;
        let mut tool_results = String::new();
        for (idx, (name, args)) in tool_calls.iter().enumerate() {
            let tool_call_id = format!("{run_id}_{round}_{idx}");

            // Repeating a call that already failed cannot succeed, and the
            // model will keep doing it until the rounds run out — which reads
            // to the user as the agent spinning in a loop.
            let signature = format!("{name}:{args}");
            if failed_calls.contains(&signature) {
                let _ = write!(tool_results, "\n{}\n", Self::already_failed(name));
                continue;
            }
            let Some(tool) = self.tools.get_tool(name) else {
                let _ = write!(
                    tool_results,
                    "\n{}\n",
                    tool_calls::format_tool_result(name, "{\"error\":\"unknown tool\"}")
                );
                continue;
            };

            let mapped = Self::map_tool_class(tool.class());
            let decision = self.policy.evaluate(
                &mapped,
                name,
                args,
                workspace,
                &request.mode,
                Some(&run_id.to_string()),
            );

            match decision {
                Decision::Deny => {
                    failed_calls.insert(signature);
                    let refusal =
                        Self::report_denied(&tool_call_id, name, args, workspace, tx).await;
                    let _ = write!(tool_results, "\n{refusal}\n");
                    continue;
                }
                Decision::NeedsApproval => {
                    let _ = tx
                        .send(AgentEvent::ApprovalRequired {
                            tool_call_id: tool_call_id.clone(),
                            name: name.clone(),
                            arguments: args.clone(),
                            justification: Some(format!("Allow {name}?")),
                        })
                        .await;

                    let rx = self.approvals.register(tool_call_id.clone());
                    let outcome = tokio::select! {
                        () = cancel.cancelled() => ApprovalOutcome::Denied,
                        res = rx => res.unwrap_or(ApprovalOutcome::Denied),
                        () = tokio::time::sleep(APPROVAL_TIMEOUT) => ApprovalOutcome::Denied,
                    };
                    if outcome == ApprovalOutcome::Denied {
                        let _ = write!(
                            tool_results,
                            "\n{}\n",
                            tool_calls::format_tool_result(name, "{\"error\":\"denied by user\"}")
                        );
                        continue;
                    }

                    if outcome == ApprovalOutcome::Session {
                        let _ = self.policy.record_grant(
                            mapped,
                            name,
                            args,
                            GrantScope::Session,
                            None,
                        );
                    }
                }
                Decision::Allow => {}
            }

            let _ = tx
                .send(AgentEvent::ToolStart {
                    tool_call_id: tool_call_id.clone(),
                    name: name.clone(),
                    arguments: args.clone(),
                })
                .await;

            let result = self
                .execute_tool_call(
                    tool.as_ref(),
                    mapped,
                    args,
                    workspace,
                    cancel,
                    Some(ToolStreamContext {
                        tx,
                        tool_call_id: &tool_call_id,
                    }),
                )
                .await;

            // Remember failures so an identical retry is short-circuited.
            if result.get("error").is_some() {
                failed_calls.insert(signature);
            }

            let _ = tx
                .send(AgentEvent::ToolEnd {
                    tool_call_id: tool_call_id.clone(),
                    result: result.clone(),
                })
                .await;

            executed_steps.push(ExecutedStep {
                name: name.clone(),
                arguments: args.clone(),
                result: result.clone(),
            });

            let _ = write!(
                tool_results,
                "\n{}\n",
                tool_calls::format_tool_result(
                    name,
                    &serde_json::to_string(&result).unwrap_or_else(|_| "{}".into())
                )
            );
        }

        Ok(tool_results)
    }

    /// Runs one already-approved tool call under a timeout.
    ///
    /// Every tool declares `timeout_ms()`, but only `Execute`-class calls
    /// (shell commands) ever had it enforced — `write_file`/`edit_file`/
    /// `read_file`/`list_dir` ran with no ceiling at all, so a tool that
    /// hung (a stuck lock, a blocked filesystem call) hung the entire run
    /// with no recovery but the user cancelling by hand. Every class is
    /// bounded now.
    ///
    /// `Execute` still uses the workspace policy's `commands.max_runtime_seconds`
    /// rather than the tool's own `timeout_ms()`: the policy limit is the
    /// actual governing ceiling for shell commands and can be tighter, so it
    /// wraps execution here instead of requiring every `Tool` impl to know
    /// about `PolicyEngine`.
    ///
    /// Never returns an `Err` — failures and timeouts are reported as an
    /// `{"error": ...}` tool result, which the model sees and can react to.
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

#[async_trait]
impl AgentEngine for AgentLoopEngine {
    async fn run_stream(
        &self,
        request: RunRequest,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError> {
        if Self::should_use_agent_loop(&request.mode) && request.workspace_root.is_some() {
            return self.run_agent_loop(request, tx, cancel).await;
        }

        // No workspace (or a conversation-only mode): still send a system
        // prompt, so the model says it cannot see the user's files instead of
        // asking them to paste a path it could never read anyway.
        let workspace = self.workspace_from_request(&request);
        let system_prompt =
            PromptAssembler::system_prompt(&request, workspace.as_ref(), &[]);
        self.gateway.run_stream_with_system(request, Some(&system_prompt), tx, cancel).await
    }

    fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.get_definitions_for_mode(&RunMode::Agent)
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_streaming: true,
            supports_thinking: true,
            supports_subagents: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentLoopEngine, Arc, ToolClass, TrustStore};
    use agent_policy::engine::PolicyEngine;
    use agent_policy::presets::standard_preset;
    use agent_policy::store::GrantStore;
    use agent_tools::registry::{RiskLevel, Tool, ToolClass as ToolsToolClass, ToolError};
    use agent_tools::registry_factory::default_registry;
    use agent_types::requests::{RunMode, RunRequest};
    use async_trait::async_trait;
    use serde_json::{Value, json};
    use std::error::Error;
    use tokio_util::sync::CancellationToken;

    /// A tool that just sleeps for a configured duration, to exercise the
    /// `max_runtime_seconds` timeout wrapper without touching a real shell.
    struct SleepyTool {
        sleep: std::time::Duration,
        class: ToolsToolClass,
        timeout_ms: u32,
    }

    impl SleepyTool {
        fn execute_class(sleep: std::time::Duration, class: ToolsToolClass, timeout_ms: u32) -> Self {
            Self { sleep, class, timeout_ms }
        }
    }

    impl Default for SleepyTool {
        fn default() -> Self {
            Self {
                sleep: std::time::Duration::from_secs(5),
                class: ToolsToolClass::Execute,
                timeout_ms: 60_000,
            }
        }
    }

    #[async_trait]
    impl Tool for SleepyTool {
        fn name(&self) -> &'static str {
            "sleepy"
        }
        fn version(&self) -> u32 {
            1
        }
        fn title(&self) -> &'static str {
            "Sleepy"
        }
        fn description(&self) -> &'static str {
            "Sleeps, for tests."
        }
        fn input_schema(&self) -> Value {
            json!({})
        }
        fn class(&self) -> ToolsToolClass {
            self.class
        }
        fn risk(&self) -> RiskLevel {
            RiskLevel::Elevated
        }
        fn modes(&self) -> Vec<RunMode> {
            vec![]
        }
        fn idempotent(&self) -> bool {
            false
        }
        fn mutates(&self) -> bool {
            false
        }
        fn checkpointed(&self) -> bool {
            false
        }
        fn timeout_ms(&self) -> u32 {
            self.timeout_ms
        }
        fn result_cap_tokens(&self) -> u32 {
            1_000
        }
        async fn execute(
            &self,
            _args: Value,
            _workspace: Option<&agent_sandbox::workspace::Workspace>,
        ) -> Result<Value, ToolError> {
            tokio::time::sleep(self.sleep).await;
            Ok(json!({"ok": true}))
        }
    }

    fn engine_with_max_runtime_seconds(
        max_runtime_seconds: u32,
    ) -> Result<AgentLoopEngine, Box<dyn Error>> {
        let gateway = crate::providers::gateway::EuryGatewayProvider::new();
        let mut policy = standard_preset();
        policy.commands.max_runtime_seconds = max_runtime_seconds;
        let grant_store = GrantStore::new_in_memory()?;
        Ok(AgentLoopEngine {
            gateway,
            tools: default_registry(),
            policy: PolicyEngine::new(policy, grant_store)?,
            approvals: std::sync::Arc::new(crate::approval::ApprovalWaiter::new()),
            trust: Arc::new(TrustStore::new()),
        })
    }

    #[test]
    fn workspace_from_request_reads_trust_from_the_store_instead_of_assuming_it()
    -> Result<(), Box<dyn Error>> {
        let engine = engine_with_max_runtime_seconds(60)?;
        // A path that doesn't exist on disk: `TrustStore` falls back to the
        // literal path when canonicalization fails, so trust lookups stay
        // consistent without this crate touching the filesystem (which the
        // agent-sandbox boundary check forbids outside that crate).
        let root =
            std::path::PathBuf::from(format!("/nonexistent/eury-trust-{}", uuid::Uuid::now_v7()));

        let request = RunRequest {
            run_id: uuid::Uuid::now_v7(),
            conversation_id: uuid::Uuid::now_v7(),
            prompt: String::new(),
            history: vec![],
            attachments: vec![],
            workspace_id: None,
            mode: RunMode::Agent,
            model: agent_types::requests::ModelConfig {
                provider: "stub".into(),
                model: "stub".into(),
                max_tokens: None,
                temperature: 0.0,
            },
            workspace_root: Some(root.clone()),
            plan_context: None,
        };

        // A root nobody has trusted must come back untrusted — the old code
        // hardcoded `TrustState::Trusted` here regardless.
        let ws = engine.workspace_from_request(&request);
        assert!(ws.is_some_and(|w| !w.is_trusted()), "an untrusted root must not report trusted");

        engine.trust.set_trust(&root, true);
        let ws = engine.workspace_from_request(&request);
        assert!(
            ws.is_some_and(|w| w.is_trusted()),
            "an explicitly trusted root must report trusted"
        );

        Ok(())
    }

    #[tokio::test]
    async fn execute_tool_call_times_out_execute_class_tools_past_max_runtime_seconds()
    -> Result<(), Box<dyn Error>> {
        let engine = engine_with_max_runtime_seconds(1)?;
        let tool = SleepyTool::default();

        let cancel = CancellationToken::new();
        let result = engine
            .execute_tool_call(&tool, ToolClass::Execute, &json!({}), None, &cancel, None)
            .await;

        let Some(error) = result["error"].as_str() else {
            return Err("expected a timeout error result".into());
        };
        assert!(
            error.contains("max_runtime_seconds"),
            "expected a max_runtime_seconds timeout error, got: {error}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn execute_tool_call_lets_execute_class_tools_finish_within_max_runtime_seconds()
    -> Result<(), Box<dyn Error>> {
        let engine = engine_with_max_runtime_seconds(5)?;
        let tool = SleepyTool { sleep: std::time::Duration::from_millis(10), ..Default::default() };

        let cancel = CancellationToken::new();
        let result = engine
            .execute_tool_call(&tool, ToolClass::Execute, &json!({}), None, &cancel, None)
            .await;

        assert_eq!(result["ok"], true);
        Ok(())
    }

    #[tokio::test]
    async fn execute_tool_call_does_not_apply_the_execute_policy_cap_to_other_tool_classes()
    -> Result<(), Box<dyn Error>> {
        // A non-Execute tool sleeping past max_runtime_seconds must not be cut
        // off by *that* cap — it is specifically an Execute-class ceiling. It
        // is still bounded by its own timeout_ms() (60s here), which this
        // 1.2s sleep does not come close to.
        let engine = engine_with_max_runtime_seconds(1)?;
        let tool = SleepyTool {
            sleep: std::time::Duration::from_millis(1_200),
            ..Default::default()
        };

        let cancel = CancellationToken::new();
        let result = engine
            .execute_tool_call(&tool, ToolClass::Read, &json!({}), None, &cancel, None)
            .await;

        assert_eq!(result["ok"], true);
        Ok(())
    }

    /// The bug from production: `write_file` (Write class) has no execution
    /// path that ever applied its own declared `timeout_ms()` — only
    /// `Execute`-class calls were bounded, so a hung filesystem tool hung the
    /// whole run with the UI stuck on "running a tool…" and no way out but a
    /// manual cancel. Every class must now be bounded by its own timeout.
    #[tokio::test]
    async fn a_hung_write_class_tool_is_cut_off_by_its_own_timeout() -> Result<(), Box<dyn Error>> {
        let engine = engine_with_max_runtime_seconds(600)?; // generous — must not be what saves this
        let tool = SleepyTool::execute_class(
            std::time::Duration::from_secs(30),
            ToolsToolClass::Write,
            50, // ms — far shorter than the 30s the tool actually sleeps
        );

        let started = std::time::Instant::now();
        let cancel = CancellationToken::new();
        let result = engine
            .execute_tool_call(&tool, ToolClass::Write, &json!({}), None, &cancel, None)
            .await;

        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "a Write-class tool must not be allowed to hang past its own timeout",
        );
        let error = result["error"].as_str().unwrap_or_default();
        assert!(error.contains("sleepy"), "the error must name the tool: {error}");
        assert!(error.contains("timeout"), "got: {error}");
        Ok(())
    }

    #[test]
    fn continuation_prompt_after_failure_asks_to_install_deps() {
        use super::{build_tool_continuation_prompt, tool_results_indicate_failure};

        let failed = r#"[tool_result name=run_command]
{"exit_code":1,"stdout":"","stderr":"UNRESOLVED_IMPORT Could not resolve 'vite'"}
[/tool_result]"#;
        assert!(tool_results_indicate_failure(failed));
        let prompt = build_tool_continuation_prompt("run that app", failed);
        assert!(prompt.contains("npm install") || prompt.contains("pnpm install"));
        assert!(prompt.contains("tool_call"));
    }

    #[test]
    fn continuation_prompt_after_success_asks_for_plain_answer() {
        use super::build_tool_continuation_prompt;

        let ok = r#"[tool_result name=run_command]
{"exit_code":0,"stdout":"11.5.1\n","stderr":""}
[/tool_result]"#;
        let prompt = build_tool_continuation_prompt("what is my pnpm version", ok);
        assert!(prompt.contains("plain language"));
        assert!(!prompt.contains("emit ```tool_call```"));
    }

    #[test]
    fn continuation_prompt_after_list_dir_keeps_action_tasks_going() {
        use super::{build_tool_continuation_prompt, user_request_needs_action};

        assert!(user_request_needs_action("run the app locally"));
        let listed = r#"[tool_result name=list_dir]
{"entries":["package.json","src","vite.config.js"]}
[/tool_result]"#;
        let prompt = build_tool_continuation_prompt("run the app locally", listed);
        assert!(prompt.contains("Continue the task"));
        assert!(prompt.contains("tool_call"));
        assert!(prompt.contains("NOT finished"));
    }

    #[test]
    fn auto_continue_when_model_stops_after_planning_text() {
        use super::should_auto_continue_action_task;

        assert!(should_auto_continue_action_task(
            "run the app locally",
            "I'll inspect package.json, install dependencies, and start the dev server.",
            0,
            &[],
        ));
    }

    #[test]
    fn auto_continue_after_list_dir_only_on_run_app() {
        use super::{ExecutedStep, should_auto_continue_action_task};
        use serde_json::json;

        let executed = vec![ExecutedStep {
            name: "list_dir".into(),
            arguments: json!({"path": "."}),
            result: json!({"entries": ["package.json"]}),
        }];
        assert!(should_auto_continue_action_task(
            "run the app locally",
            "Found package.json.",
            1,
            &executed,
        ));
    }

    #[test]
    fn stops_when_task_marked_complete() {
        use super::should_auto_continue_action_task;

        assert!(!should_auto_continue_action_task(
            "run the app locally",
            "Dev server is up at http://localhost:5173\nTASK_COMPLETE",
            2,
            &[],
        ));
    }

    #[test]
    fn stops_for_simple_questions_without_action() {
        use super::should_auto_continue_action_task;

        assert!(!should_auto_continue_action_task(
            "what is my pnpm version",
            "Your pnpm version is 11.5.1.",
            0,
            &[],
        ));
    }
}
