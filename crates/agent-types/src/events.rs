use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AgentEvent {
    Meta {
        run_id: Uuid,
        status: String,
    },
    TextDelta {
        text: String,
    },
    ThinkingDelta {
        text: String,
    },
    ToolStart {
        tool_call_id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolEnd {
        tool_call_id: String,
        result: serde_json::Value,
    },
    ToolOutputDelta {
        tool_call_id: String,
        stream: String,
        text: String,
    },
    ApprovalRequired {
        tool_call_id: String,
        name: String,
        arguments: serde_json::Value,
        justification: Option<String>,
    },
    TurnComplete {
        turn_id: Uuid,
    },
    ContextWarning {
        tokens: u32,
        limit: u32,
        message: String,
    },
    CompactStart {
        before_tokens: u32,
    },
    CompactEnd {
        after_tokens: u32,
    },
    SubagentSpawned {
        subagent_id: Uuid,
        role: String,
        task: String,
    },
    SubagentComplete {
        subagent_id: Uuid,
        result: String,
    },
    CostUpdate {
        tokens_prompt: u32,
        tokens_completion: u32,
        cost_usd_micros: u64,
    },
    RunComplete {
        run_id: Uuid,
        stop_reason: String,
    },
    RunError {
        run_id: Uuid,
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub id: String,
    pub timestamp: i64,
    #[serde(flatten)]
    pub event: AgentEvent,
}
