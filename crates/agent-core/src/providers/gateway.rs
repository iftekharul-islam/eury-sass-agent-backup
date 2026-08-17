use crate::accounting::{CostEstimator, CostGuard, CostGuardConfig};
use crate::auth::{AuthStore, AuthTokens};
use crate::engine::AgentEngine;
use agent_types::capabilities::{EngineCapabilities, ToolDefinition};
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::{RunMode, RunOutcome, RunRequest, RunStatus};
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const DEFAULT_GATEWAY_URL: &str = "http://localhost:3001/agent/v1/chat/stream";

/// How long the stream may go completely silent — no chunk, of any kind —
/// before the run gives up. The client's overall request timeout is 5
/// minutes, which is the right ceiling for a slow-but-alive generation; it
/// is the wrong answer to a upstream connection that has quietly stalled,
/// which left users watching "waiting for the model…" for minutes with
/// nothing but a manual Esc to save them. This fires much sooner and turns
/// that silence into a clear, retryable error.
const STREAM_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(45);

/// How long to wait for the server to even start responding — connect
/// through response headers — before giving up.
///
/// This is the phase [`STREAM_IDLE_TIMEOUT`] does NOT cover: it only guards
/// the body once bytes are already flowing. A multi-round agent run opens a
/// fresh connection per round (`open_stream` is called again for round 2,
/// round 3, ...), and if the server hangs before sending headers on any of
/// those later rounds, the read loop is never entered — so the per-chunk
/// guard never starts, and the only ceiling left was the 5-minute overall
/// client timeout. That is exactly the case where round 1 streams real text,
/// the user sees it, and round 2 then sits silent for minutes with nothing
/// to show for it.
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GatewayStreamRequest {
    run_id: String,
    provider: String,
    model: String,
    mode: String,
    messages: Vec<GatewayMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct GatewayMessage {
    role: String,
    content: Vec<GatewayMessagePart>,
}

#[derive(Debug, Serialize)]
struct GatewayMessagePart {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "attachmentId", skip_serializing_if = "Option::is_none")]
    attachment_id: Option<String>,
    #[serde(rename = "mediaType", skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(rename = "dataBase64", skip_serializing_if = "Option::is_none")]
    data_base64: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GatewayStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    text: Option<String>,
    #[serde(alias = "promptTokens")]
    prompt_tokens: Option<u32>,
    #[serde(alias = "completionTokens")]
    completion_tokens: Option<u32>,
    #[serde(alias = "costUsdMicros")]
    cost_usd_micros: Option<u64>,
    finish_reason: Option<String>,
    code: Option<String>,
    message: Option<String>,
    // `tool_call` events, per the cloud API contract:
    // `{ type: "tool_call", id, name, argumentsDelta?, arguments? }`.
    id: Option<String>,
    name: Option<String>,
    arguments: Option<Value>,
    #[serde(rename = "argumentsDelta")]
    arguments_delta: Option<String>,
}

/// A tool call assembled from the gateway's structured `tool_call` events.
///
/// Providers may stream a call's arguments incrementally as
/// `argumentsDelta` string fragments, so deltas are accumulated per call id
/// and parsed once the stream completes; a call that arrives with a whole
/// `arguments` object is used directly.
#[derive(Debug, Default)]
pub struct ToolCallAccumulator {
    /// Insertion-ordered call ids, so tool calls execute in the order the
    /// model emitted them.
    order: Vec<String>,
    names: std::collections::HashMap<String, String>,
    complete_args: std::collections::HashMap<String, Value>,
    partial_args: std::collections::HashMap<String, String>,
}

impl ToolCallAccumulator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn record(&mut self, event: &GatewayStreamEvent) {
        // An id is required to correlate deltas; fall back to the tool name
        // when a provider omits it for a single, non-streamed call.
        let Some(id) = event.id.clone().or_else(|| event.name.clone()) else {
            return;
        };
        if !self.names.contains_key(&id) {
            self.order.push(id.clone());
        }
        if let Some(name) = &event.name {
            self.names.insert(id.clone(), name.clone());
        }
        if let Some(args) = &event.arguments {
            self.complete_args.insert(id.clone(), args.clone());
        }
        if let Some(delta) = &event.arguments_delta {
            self.partial_args.entry(id).or_default().push_str(delta);
        }
    }

    /// Returns the assembled `(name, arguments)` pairs in emission order.
    /// A call whose accumulated argument fragments don't parse as JSON is
    /// dropped rather than executed with a half-formed payload.
    #[must_use]
    pub fn finish(self) -> Vec<(String, Value)> {
        let mut calls = Vec::new();
        for id in &self.order {
            let Some(name) = self.names.get(id) else { continue };
            let args = if let Some(args) = self.complete_args.get(id) {
                args.clone()
            } else if let Some(raw) = self.partial_args.get(id) {
                match serde_json::from_str(raw) {
                    Ok(parsed) => parsed,
                    Err(_) => continue,
                }
            } else {
                Value::Object(serde_json::Map::new())
            };
            calls.push((name.clone(), args));
        }
        calls
    }
}

pub struct EuryGatewayProvider {
    gateway_url: String,
    cost_config: CostGuardConfig,
}

impl EuryGatewayProvider {
    #[must_use]
    pub fn new() -> Self {
        let gateway_url = std::env::var("EURY_AGENT_GATEWAY_URL")
            .unwrap_or_else(|_| DEFAULT_GATEWAY_URL.to_string());
        Self { gateway_url, cost_config: CostGuardConfig::default() }
    }

    pub fn with_gateway_url(gateway_url: impl Into<String>) -> Self {
        Self { gateway_url: gateway_url.into(), cost_config: CostGuardConfig::default() }
    }

    fn mode_to_string(mode: &RunMode) -> &'static str {
        match mode {
            RunMode::Chat => "chat",
            RunMode::Ask => "ask",
            RunMode::Plan => "plan",
            RunMode::Agent => "agent",
            RunMode::Build => "build",
        }
    }

    fn prompt_needs_run_command(prompt: &str) -> bool {
        let lower = prompt.to_ascii_lowercase();
        [
            "version",
            "nodejs",
            "node.js",
            "node js",
            "npm version",
            "pnpm version",
            "python version",
            "which node",
            "which npm",
            "installed",
            "environment",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
    }

    fn prompt_needs_write_tools(prompt: &str) -> bool {
        let lower = prompt.to_ascii_lowercase();
        [
            "create",
            "scaffold",
            "implement",
            "generate",
            "make ",
            "build ",
            "add ",
            "write ",
            "setup",
            "set up",
            "init",
            "bootstrap",
            "portfolio",
            "project",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
    }

    fn prompt_needs_dev_server(prompt: &str) -> bool {
        let lower = prompt.to_ascii_lowercase();
        [
            "run the app",
            "run that app",
            "start the app",
            "start dev",
            "dev server",
            "npm run dev",
            "pnpm run dev",
            "pnpm dev",
            "yarn dev",
            "run it",
            "launch the app",
            "start the server",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
    }

    /// Sends a terminal `RunError` for a cost/token cap breach and returns
    /// the error the caller propagates.
    async fn report_cost_cap_exceeded(
        run_id: uuid::Uuid,
        message: &str,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> AgentError {
        let _ = tx
            .send(AgentEvent::RunError {
                run_id,
                code: "EURY_COST_CAP_EXCEEDED".to_string(),
                message: message.to_string(),
            })
            .await;
        AgentError::BudgetExceeded
    }

    /// Sends a terminal `RunError` for a stream that has gone completely
    /// silent, and returns the error both read loops surface it as.
    async fn report_stream_stalled(run_id: uuid::Uuid, tx: &mpsc::Sender<AgentEvent>) -> AgentError {
        let _ = tx
            .send(AgentEvent::RunError {
                run_id,
                code: "EURY_STREAM_STALLED".to_string(),
                message: "The model stopped responding. Try again.".to_string(),
            })
            .await;
        AgentError::Internal("stream idle timeout".to_string())
    }

    /// True when an upstream NDJSON line represents real model progress.
    /// Heartbeats, `meta`, and empty deltas must not reset the stall clock.
    fn gateway_event_is_meaningful(event_type: &str, event: &GatewayStreamEvent) -> bool {
        match event_type {
            "delta" | "reasoning" => event.text.as_ref().is_some_and(|text| !text.is_empty()),
            "tool_call" | "usage" | "done" | "error" => true,
            _ => false,
        }
    }

    fn stream_wait_remaining(last_meaningful: std::time::Instant) -> std::time::Duration {
        STREAM_IDLE_TIMEOUT.saturating_sub(last_meaningful.elapsed())
    }

    async fn next_stream_chunk<S>(
        stream: &mut S,
        cancel: &CancellationToken,
        last_meaningful: std::time::Instant,
        run_id: uuid::Uuid,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<Option<bytes::Bytes>, AgentError>
    where
        S: futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    {
        let wait = Self::stream_wait_remaining(last_meaningful);
        if wait.is_zero() {
            return Err(Self::report_stream_stalled(run_id, tx).await);
        }

        tokio::select! {
            () = cancel.cancelled() => {
                let _ = tx.send(AgentEvent::RunComplete {
                    run_id,
                    stop_reason: "aborted".to_string(),
                }).await;
                Err(AgentError::RunCancelled)
            }
            timed_out = tokio::time::timeout(wait, stream.next()) => {
                match timed_out {
                    Err(_) => Err(Self::report_stream_stalled(run_id, tx).await),
                    Ok(None) => Ok(None),
                    Ok(Some(Ok(chunk))) => Ok(Some(chunk)),
                    Ok(Some(Err(e))) => Err(AgentError::Internal(e.to_string())),
                }
            }
        }
    }

    fn text_message(role: &str, text: &str) -> GatewayMessage {
        GatewayMessage {
            role: role.to_string(),
            content: vec![GatewayMessagePart {
                part_type: "text".to_string(),
                text: Some(text.to_string()),
                attachment_id: None,
                media_type: None,
                data_base64: None,
            }],
        }
    }

    fn is_tool_continuation(prompt: &str) -> bool {
        prompt.contains("[tool_result")
    }

    /// Builds the wire request: the system prompt (which is what tells the
    /// model it has tools and a workspace), then the prior turns, then this
    /// turn's prompt and attachments.
    fn build_request(req: &RunRequest, system_prompt: Option<&str>) -> GatewayStreamRequest {
        let mut messages = Vec::new();

        if let Some(system) = system_prompt.filter(|s| !s.trim().is_empty()) {
            messages.push(Self::text_message("system", system));
            // Mirror the desktop code chat's sticky priming so the model does
            // not answer "run this in your terminal" when run_command is listed.
            if req.history.is_empty() && system.contains("run_command") {
                let (sticky_user, sticky_assistant) =
                    if Self::prompt_needs_dev_server(&req.prompt) {
                        (
                            "[sticky] The user wants to run or start an app/dev server. Read \
                             package.json if needed, run install when node_modules is missing, then \
                             start the dev server with run_command. Emit ```tool_call``` fences for \
                             every step — never only describe plans.",
                            "Understood. I will inspect scripts, install dependencies if needed, \
                             then start the dev server.",
                        )
                    } else if Self::prompt_needs_write_tools(&req.prompt) {
                        (
                            "[sticky] The user wants files created or a project scaffolded. You have \
                             write_file, edit_file, and run_command — emit ```tool_call``` JSON \
                             fences now. Never say workspace or terminal tools are unavailable.",
                            "Understood. I will scaffold with write_file and use run_command to \
                             install or run.",
                        )
                    } else if Self::prompt_needs_run_command(&req.prompt) {
                        (
                            "[sticky] This turn needs run_command (not read_file) for shell/version \
                             checks.",
                            "Understood. I will call run_command with the right command argument.",
                        )
                    } else {
                        (
                            "[sticky] Use run_command for version/env checks when asked. Use \
                             write_file and edit_file when the user wants files created or changed.",
                            "Understood. I will address the user's current request with the right \
                             tools.",
                        )
                    };
                messages.push(Self::text_message("user", sticky_user));
                messages.push(Self::text_message("assistant", sticky_assistant));
            } else if system.contains("run_command") && Self::prompt_needs_run_command(&req.prompt)
                && !Self::is_tool_continuation(&req.prompt)
            {
                messages.push(Self::text_message(
                    "user",
                    "[sticky] This turn needs run_command (not read_file) for shell/version checks.",
                ));
                messages.push(Self::text_message(
                    "assistant",
                    "Understood. I will call run_command with the right command argument.",
                ));
            } else if system.contains("write_file") && Self::prompt_needs_write_tools(&req.prompt)
                && !Self::is_tool_continuation(&req.prompt)
            {
                messages.push(Self::text_message(
                    "user",
                    "[sticky] The user wants files created or a project scaffolded. You have \
                     write_file, edit_file, and run_command — emit ```tool_call``` JSON fences now. \
                     Never say workspace or terminal tools are unavailable.",
                ));
                messages.push(Self::text_message(
                    "assistant",
                    "Understood. I will scaffold with write_file and use run_command to install or run.",
                ));
            } else if system.contains("run_command") && Self::prompt_needs_dev_server(&req.prompt)
                && !Self::is_tool_continuation(&req.prompt)
            {
                messages.push(Self::text_message(
                    "user",
                    "[sticky] The user wants to run or start an app/dev server. Before \
                     `npm run dev` or `pnpm dev`, run install if node_modules is missing. If dev \
                     fails with UNRESOLVED_IMPORT or missing packages, run npm/pnpm install, then \
                     retry. Use run_command — never tell the user to run commands manually.",
                ));
                messages.push(Self::text_message(
                    "assistant",
                    "Understood. I will install dependencies if needed, then start the dev server.",
                ));
            }
        }

        for turn in &req.history {
            if turn.content.trim().is_empty() {
                continue;
            }
            let role = if turn.role == "assistant" { "assistant" } else { "user" };
            messages.push(Self::text_message(role, &turn.content));
        }

        let mut user_parts: Vec<GatewayMessagePart> = Vec::new();

        if !req.prompt.is_empty() {
            let text = if Self::is_tool_continuation(&req.prompt) {
                req.prompt.clone()
            } else if req.history.is_empty() {
                req.prompt.clone()
            } else {
                format!(
                    "Latest user request (answer THIS — you still have every tool in the system \
                     prompt; use write_file and run_command as needed. Earlier messages are \
                     conversation context only):\n\n{}",
                    req.prompt
                )
            };
            user_parts.push(GatewayMessagePart {
                part_type: "text".to_string(),
                text: Some(text),
                attachment_id: None,
                media_type: None,
                data_base64: None,
            });
        }

        for att in &req.attachments {
            if att.content_type.starts_with("image/") {
                user_parts.push(GatewayMessagePart {
                    part_type: "image".to_string(),
                    text: None,
                    attachment_id: Some(att.id.clone()),
                    media_type: Some(att.content_type.clone()),
                    data_base64: Some(att.data_base64.clone()),
                });
            }
        }

        if !user_parts.is_empty() {
            messages.push(GatewayMessage { role: "user".to_string(), content: user_parts });
        }

        GatewayStreamRequest {
            run_id: req.run_id.to_string(),
            provider: req.model.provider.clone(),
            model: req.model.model.clone(),
            mode: Self::mode_to_string(&req.mode).to_string(),
            messages,
            max_output_tokens: req.model.max_tokens,
            temperature: Some(req.model.temperature),
        }
    }

    fn get_tokens() -> Result<AuthTokens, AgentError> {
        AuthStore::get_tokens().map_err(|_| {
            AgentError::Unauthorized("no active session — sign in to continue".to_string())
        })
    }

    /// Opens the NDJSON gateway stream, sending a `RunError` event and returning
    /// an error if the HTTP request fails or the upstream responds with an error status.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client fails to build, the request fails, or the
    /// upstream responds with a non-success status.
    async fn open_stream(
        &self,
        tokens: &AuthTokens,
        body: &GatewayStreamRequest,
        run_id: uuid::Uuid,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<impl futures::Stream<Item = reqwest::Result<bytes::Bytes>>, AgentError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_mins(5))
            .build()
            .map_err(|e| AgentError::Internal(e.to_string()))?;

        let response = match tokio::time::timeout(
            CONNECT_TIMEOUT,
            client.post(&self.gateway_url).bearer_auth(&tokens.access_token).json(body).send(),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                let message = format!("Gateway request failed: {e}");
                let _ = tx
                    .send(AgentEvent::RunError {
                        run_id,
                        code: "EURY_UPSTREAM_FAILED".to_string(),
                        message: message.clone(),
                    })
                    .await;
                return Err(AgentError::Internal(message));
            }
            Err(_) => {
                let _ = tx
                    .send(AgentEvent::RunError {
                        run_id,
                        code: "EURY_STREAM_STALLED".to_string(),
                        message: "The model did not start responding in time. Try again."
                            .to_string(),
                    })
                    .await;
                return Err(AgentError::Internal("connect timeout".to_string()));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            let _ = tx
                .send(AgentEvent::RunError {
                    run_id,
                    code: "EURY_UPSTREAM_FAILED".to_string(),
                    message: format!("Gateway error {status}: {text}"),
                })
                .await;
            return Err(AgentError::Internal(format!("Gateway HTTP {status}")));
        }

        Ok(response.bytes_stream())
    }

    /// Records a "usage" event's token deltas against the running totals and cost guard,
    /// returning the `CostUpdate` event to forward to the UI.
    fn account_usage(
        cost_guard: &mut CostGuard,
        model_id: &str,
        total_prompt: &mut u32,
        total_completion: &mut u32,
        event: &GatewayStreamEvent,
    ) -> AgentEvent {
        let p = event.prompt_tokens.unwrap_or(0);
        let c = event.completion_tokens.unwrap_or(0);
        *total_prompt += p;
        *total_completion += c;
        cost_guard.record_usage(model_id, p, c);
        let cost = event
            .cost_usd_micros
            .unwrap_or_else(|| CostEstimator::estimate_cost_micros(model_id, p, c));
        AgentEvent::CostUpdate {
            tokens_prompt: *total_prompt,
            tokens_completion: *total_completion,
            cost_usd_micros: cost,
        }
    }

    /// Appends `chunk` to `buffer` and drains any complete, non-empty NDJSON lines from it.
    fn drain_ndjson_lines(buffer: &mut String, chunk: &[u8]) -> Vec<String> {
        buffer.push_str(&String::from_utf8_lossy(chunk));
        let mut lines = Vec::new();
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            *buffer = buffer[pos + 1..].to_string();
            if !line.is_empty() {
                lines.push(line);
            }
        }
        lines
    }

    /// Stream one gateway turn, forwarding events and returning the assistant
    /// text plus any structured `tool_call`s the model emitted.
    ///
    /// Tool calls come from the gateway's typed `tool_call` NDJSON events, not
    /// from scanning the completed text for ` ```tool_call ` fences — text
    /// that merely *looks* like a tool call (a fenced example in an
    /// explanation, say) must never be executed as one.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails, the HTTP request fails, the upstream
    /// responds with a non-success status, or an NDJSON line fails to parse.
    pub async fn stream_collecting(
        &self,
        request: &RunRequest,
        system_prompt: Option<&str>,
        tx: &mpsc::Sender<AgentEvent>,
        cancel: &CancellationToken,
    ) -> Result<(String, Vec<(String, Value)>, RunOutcome), AgentError> {
        let run_id = request.run_id;
        let model_id = request.model.model.clone();
        let tokens = Self::get_tokens()?;
        let body = Self::build_request(request, system_prompt);
        let mut cost_guard = CostGuard::new(self.cost_config.clone());
        let mut accumulated = String::new();
        let mut tool_calls = ToolCallAccumulator::new();
        let mut stream = self.open_stream(&tokens, &body, run_id, tx).await?;
        let mut buffer = String::new();
        let mut last_meaningful = std::time::Instant::now();

        loop {
            let Some(chunk) = Self::next_stream_chunk(
                &mut stream,
                cancel,
                last_meaningful,
                run_id,
                tx,
            )
            .await?
            else {
                break;
            };

            for line in Self::drain_ndjson_lines(&mut buffer, &chunk) {
                let event: GatewayStreamEvent = serde_json::from_str(&line)
                    .map_err(|e| AgentError::Internal(format!("Invalid NDJSON: {e}")))?;

                if Self::gateway_event_is_meaningful(&event.event_type, &event) {
                    last_meaningful = std::time::Instant::now();
                }

                match event.event_type.as_str() {
                    "delta" => {
                        if let Some(text) = event.text {
                            accumulated.push_str(&text);
                            let _ = tx.send(AgentEvent::TextDelta { text }).await;
                        }
                    }
                    "reasoning" => {
                        if let Some(text) = event.text {
                            let _ = tx.send(AgentEvent::ThinkingDelta { text }).await;
                        }
                    }
                    "tool_call" => tool_calls.record(&event),
                    "usage" => {
                        let p = event.prompt_tokens.unwrap_or(0);
                        let c = event.completion_tokens.unwrap_or(0);
                        cost_guard.record_usage(&model_id, p, c);
                        let cost = event.cost_usd_micros.unwrap_or_else(|| {
                            CostEstimator::estimate_cost_micros(&model_id, p, c)
                        });
                        let _ = tx
                            .send(AgentEvent::CostUpdate {
                                tokens_prompt: p,
                                tokens_completion: c,
                                cost_usd_micros: cost,
                            })
                            .await;
                    }
                    "done" => {
                        let outcome = RunOutcome {
                            run_id,
                            status: RunStatus::Complete,
                            total_tokens: cost_guard.total_tokens(),
                            total_cost_micros: cost_guard.total_cost_micros(),
                        };
                        return Ok((accumulated, tool_calls.finish(), outcome));
                    }
                    "error" => {
                        let code = event.code.unwrap_or_else(|| "EURY_UPSTREAM_FAILED".to_string());
                        let message = event.message.unwrap_or_else(|| "Gateway error".to_string());
                        let _ = tx.send(AgentEvent::RunError { run_id, code, message }).await;
                        return Err(AgentError::Internal("Gateway stream error".to_string()));
                    }
                    _ => {}
                }
            }
        }

        Ok((
            accumulated,
            tool_calls.finish(),
            RunOutcome {
                run_id,
                status: RunStatus::Complete,
                total_tokens: cost_guard.total_tokens(),
                total_cost_micros: cost_guard.total_cost_micros(),
            },
        ))
    }
}

impl Default for EuryGatewayProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentEngine for EuryGatewayProvider {
    async fn run_stream(
        &self,
        request: RunRequest,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError> {
        self.run_stream_with_system(request, None, tx, cancel).await
    }

    fn tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![]
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_streaming: true,
            supports_thinking: true,
            supports_subagents: false,
        }
    }
}

impl EuryGatewayProvider {
    /// Single-turn stream with an explicit system prompt. The trait's
    /// `run_stream` is this with no system prompt.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails, the HTTP request fails, the
    /// upstream responds with a non-success status, or a stream line fails to
    /// parse.
    pub async fn run_stream_with_system(
        &self,
        request: RunRequest,
        system_prompt: Option<&str>,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError> {
        let run_id = request.run_id;
        let model_id = request.model.model.clone();
        let tokens = Self::get_tokens()?;
        let body = Self::build_request(&request, system_prompt);
        let mut cost_guard = CostGuard::new(self.cost_config.clone());

        let _ = tx.send(AgentEvent::Meta { run_id, status: "streaming".to_string() }).await;

        let mut stream = self.open_stream(&tokens, &body, run_id, &tx).await?;
        let mut buffer = String::new();
        let mut total_prompt = 0u32;
        let mut total_completion = 0u32;
        let mut last_meaningful = std::time::Instant::now();

        loop {
            if cost_guard.is_exceeded() {
                return Err(Self::report_cost_cap_exceeded(run_id, "Run cost or token cap exceeded", &tx).await);
            }

            let Some(chunk) = Self::next_stream_chunk(
                &mut stream,
                &cancel,
                last_meaningful,
                run_id,
                &tx,
            )
            .await?
            else {
                break;
            };

            for line in Self::drain_ndjson_lines(&mut buffer, &chunk) {
                let event: GatewayStreamEvent = serde_json::from_str(&line)
                    .map_err(|e| AgentError::Internal(format!("Invalid NDJSON: {e}")))?;

                if Self::gateway_event_is_meaningful(&event.event_type, &event) {
                    last_meaningful = std::time::Instant::now();
                }

                match event.event_type.as_str() {
                    "delta" => {
                        if let Some(text) = event.text {
                            if cost_guard.would_exceed(&model_id, 50, 50) {
                                return Err(Self::report_cost_cap_exceeded(
                                    run_id,
                                    "Projected run cost exceeds cap",
                                    &tx,
                                )
                                .await);
                            }
                            let _ = tx.send(AgentEvent::TextDelta { text }).await;
                        }
                    }
                    "reasoning" => {
                        if let Some(text) = event.text {
                            let _ = tx.send(AgentEvent::ThinkingDelta { text }).await;
                        }
                    }
                    "usage" => {
                        let evt = Self::account_usage(
                            &mut cost_guard,
                            &model_id,
                            &mut total_prompt,
                            &mut total_completion,
                            &event,
                        );
                        let _ = tx.send(evt).await;
                    }
                    "done" => {
                        let reason = event.finish_reason.unwrap_or_else(|| "stop".to_string());
                        let _ =
                            tx.send(AgentEvent::RunComplete { run_id, stop_reason: reason }).await;
                        return Ok(RunOutcome {
                            run_id,
                            status: RunStatus::Complete,
                            total_tokens: cost_guard.total_tokens(),
                            total_cost_micros: cost_guard.total_cost_micros(),
                        });
                    }
                    "error" => {
                        let code = event.code.unwrap_or_else(|| "EURY_UPSTREAM_FAILED".to_string());
                        let message = event.message.unwrap_or_else(|| "Gateway error".to_string());
                        let _ = tx.send(AgentEvent::RunError { run_id, code, message }).await;
                        return Err(AgentError::Internal("Gateway stream error".to_string()));
                    }
                    _ => {}
                }
            }
        }

        let _ = tx.send(AgentEvent::RunComplete { run_id, stop_reason: "stop".to_string() }).await;

        Ok(RunOutcome {
            run_id,
            status: RunStatus::Complete,
            total_tokens: cost_guard.total_tokens(),
            total_cost_micros: cost_guard.total_cost_micros(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{EuryGatewayProvider, GatewayStreamEvent, ToolCallAccumulator};
    use agent_types::events::AgentEvent;
    use agent_types::requests::{HistoryMessage, ModelConfig, RunMode, RunRequest};
    use serde_json::json;
    use tokio::io::AsyncReadExt as _;

    fn parse(line: &str) -> Result<GatewayStreamEvent, serde_json::Error> {
        serde_json::from_str(line)
    }

    fn request_with_history() -> RunRequest {
        RunRequest {
            run_id: uuid::Uuid::now_v7(),
            conversation_id: uuid::Uuid::now_v7(),
            mode: RunMode::Agent,
            prompt: "and now delete it".to_string(),
            history: vec![
                HistoryMessage { role: "user".into(), content: "list the files".into() },
                HistoryMessage { role: "assistant".into(), content: "There is one: a.rs".into() },
                HistoryMessage { role: "assistant".into(), content: "   ".into() },
            ],
            attachments: vec![],
            workspace_id: None,
            workspace_root: None,
            model: ModelConfig {
                provider: "OpenAI".into(),
                model: "gpt-5.6".into(),
                temperature: 0.7,
                max_tokens: None,
            },
            plan_context: None,
        }
    }

    #[test]
    fn sends_the_system_prompt_and_prior_turns_before_this_one() {
        let body = EuryGatewayProvider::build_request(
            &request_with_history(),
            Some("You are Eury Agent."),
        );

        let roles: Vec<&str> = body.messages.iter().map(|m| m.role.as_str()).collect();
        // The blank history turn is dropped rather than sent as an empty message.
        assert_eq!(roles, vec!["system", "user", "assistant", "user"]);
        assert_eq!(body.messages[0].content[0].text.as_deref(), Some("You are Eury Agent."));
        assert_eq!(body.messages[1].content[0].text.as_deref(), Some("list the files"));
        assert_eq!(
            body.messages[3].content[0].text.as_deref(),
            Some(
                "Latest user request (answer THIS — you still have every tool in the system \
                 prompt; use write_file and run_command as needed. Earlier messages are \
                 conversation context only):\n\nand now delete it"
            )
        );
    }

    #[test]
    fn omits_the_system_message_when_there_is_no_prompt_to_send() {
        let body = EuryGatewayProvider::build_request(&request_with_history(), Some("  "));
        assert_eq!(body.messages[0].role, "user");
    }

    #[test]
    fn accumulates_a_tool_call_delivered_as_one_complete_event()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut acc = ToolCallAccumulator::new();
        acc.record(&parse(
            r#"{"type":"tool_call","id":"c1","name":"read_file","arguments":{"path":"a.rs"}}"#,
        )?);

        let calls = acc.finish();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "read_file");
        assert_eq!(calls[0].1, json!({"path": "a.rs"}));
        Ok(())
    }

    #[test]
    fn assembles_streamed_argument_deltas() -> Result<(), Box<dyn std::error::Error>> {
        let mut acc = ToolCallAccumulator::new();
        acc.record(&parse(r#"{"type":"tool_call","id":"c1","name":"write_file"}"#)?);
        acc.record(&parse(r#"{"type":"tool_call","id":"c1","argumentsDelta":"{\"path\":"}"#)?);
        acc.record(&parse(r#"{"type":"tool_call","id":"c1","argumentsDelta":"\"b.rs\"}"}"#)?);

        let calls = acc.finish();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "write_file");
        assert_eq!(calls[0].1, json!({"path": "b.rs"}));
        Ok(())
    }

    #[test]
    fn preserves_emission_order_across_multiple_calls() -> Result<(), Box<dyn std::error::Error>> {
        let mut acc = ToolCallAccumulator::new();
        acc.record(&parse(r#"{"type":"tool_call","id":"c1","name":"first","arguments":{}}"#)?);
        acc.record(&parse(r#"{"type":"tool_call","id":"c2","name":"second","arguments":{}}"#)?);

        let names: Vec<String> = acc.finish().into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["first", "second"]);
        Ok(())
    }

    #[test]
    fn drops_a_call_whose_argument_fragments_never_form_valid_json()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut acc = ToolCallAccumulator::new();
        acc.record(&parse(r#"{"type":"tool_call","id":"c1","name":"broken"}"#)?);
        acc.record(&parse(r#"{"type":"tool_call","id":"c1","argumentsDelta":"{\"path\":"}"#)?);

        assert!(
            acc.finish().is_empty(),
            "a truncated argument payload must not be executed as a tool call"
        );
        Ok(())
    }

    #[test]
    fn the_typed_accumulator_ignores_delta_text_entirely() -> Result<(), Box<dyn std::error::Error>>
    {
        // This accumulator is the *typed* path: it must only ever act on
        // `tool_call` events. Recovering calls from assistant text is a
        // separate, deliberately-fallback concern handled by
        // `crate::tool_calls`, because no server in this stack emits typed
        // events yet.
        //
        // Note this is not the same as "prose can never trigger a tool" — the
        // fence fallback means it can. The guarantee that actually holds is
        // enforced in `tool_calls`: only a *registered* tool name executes.
        // See `tool_calls::tests::drops_unregistered_tool_names`.
        let mut acc = ToolCallAccumulator::new();
        acc.record(&parse(
            r#"{"type":"delta","text":"```tool_call\n{\"name\":\"run_command\",\"arguments\":{\"command\":\"rm -rf /\"}}\n```"}"#,
        )?);

        assert!(acc.finish().is_empty(), "a delta event carries no typed tool call");
        Ok(())
    }

    #[test]
    fn meta_lines_do_not_count_as_meaningful_progress() {
        let meta = GatewayStreamEvent {
            event_type: "meta".into(),
            text: None,
            prompt_tokens: None,
            completion_tokens: None,
            cost_usd_micros: None,
            finish_reason: None,
            code: None,
            message: None,
            id: None,
            name: None,
            arguments: None,
            arguments_delta: None,
        };
        assert!(!EuryGatewayProvider::gateway_event_is_meaningful("meta", &meta));

        let delta = GatewayStreamEvent { event_type: "delta".into(), text: Some("hi".into()), ..meta };
        assert!(EuryGatewayProvider::gateway_event_is_meaningful("delta", &delta));
    }

    /// Reproduces the reported hang directly: a server that accepts the
    /// connection, reads the request, and then never writes a response —
    /// exactly what a backend stuck before it starts streaming looks like
    /// from this client's side. Before the connect timeout this waited on
    /// the 5-minute overall client timeout with nothing to show the user;
    /// it must now fail fast with a clear, terminal event.
    #[tokio::test]
    async fn a_server_that_never_responds_fails_fast_instead_of_hanging() -> Result<(), Box<dyn std::error::Error>>
    {
        crate::auth::AuthStore::set_cached_tokens(crate::auth::AuthTokens {
            access_token: "test".into(),
            refresh_token: "test".into(),
            device_id: None,
            expires_at: None,
        });

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/stream", listener.local_addr()?);
        tokio::spawn(async move {
            let Ok((mut socket, _)) = listener.accept().await else { return };
            let mut scratch = vec![0u8; 4096];
            let _ = socket.read(&mut scratch).await;
            // Deliberately never write a response — the socket stays open,
            // connected, and silent.
            std::future::pending::<()>().await;
        });

        let provider = EuryGatewayProvider::with_gateway_url(url);
        let request = RunRequest {
            run_id: uuid::Uuid::now_v7(),
            conversation_id: uuid::Uuid::now_v7(),
            mode: RunMode::Agent,
            prompt: "hi".into(),
            history: vec![],
            attachments: vec![],
            workspace_id: None,
            workspace_root: None,
            model: ModelConfig {
                provider: "OpenAI".into(),
                model: "gpt-5.6".into(),
                temperature: 0.0,
                max_tokens: None,
            },
            plan_context: None,
        };
        let run_id = request.run_id;

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        let cancel = tokio_util::sync::CancellationToken::new();

        let started = std::time::Instant::now();
        let result = provider.stream_collecting(&request, None, &tx, &cancel).await;
        let elapsed = started.elapsed();

        assert!(result.is_err(), "a server that never responds must not be treated as success");
        assert!(
            elapsed < std::time::Duration::from_secs(35),
            "connect timeout did not fire promptly: waited {elapsed:?}",
        );

        let mut saw_run_error = false;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::RunError { run_id: id, code, .. } = event {
                assert_eq!(id, run_id);
                assert_eq!(code, "EURY_STREAM_STALLED");
                saw_run_error = true;
            }
        }
        assert!(saw_run_error, "a hung connect must surface a terminal RunError to the UI");
        Ok(())
    }
}
