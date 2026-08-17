use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    Chat,
    Ask,
    Plan,
    Agent,
    Build,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanContext {
    pub plan_id: String,
    pub step_id: String,
}

/// One earlier turn of the conversation, replayed to the model so a follow-up
/// message is understood in context instead of standing alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryMessage {
    /// `user` or `assistant`.
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    pub run_id: Uuid,
    pub conversation_id: Uuid,
    pub mode: RunMode,
    pub prompt: String,
    /// Prior turns of this conversation, oldest first, excluding `prompt`.
    #[serde(default)]
    pub history: Vec<HistoryMessage>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    pub workspace_id: Option<String>,
    pub workspace_root: Option<PathBuf>,
    pub model: ModelConfig,
    pub plan_context: Option<PlanContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Assembling,
    Streaming,
    ToolRunning,
    AwaitingApproval,
    Paused,
    Compacting,
    Complete,
    Failed,
    Limited,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOutcome {
    pub run_id: Uuid,
    pub status: RunStatus,
    pub total_tokens: u32,
    pub total_cost_micros: u64,
}
