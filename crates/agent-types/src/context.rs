use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    User,
    Rule,
    Memory,
    Plan,
    Workspace,
    Tool,
    Web,
    Mcp,
    Attachment,
    OtherUser,
    Summary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Trusted,
    SemiTrusted,
    Untrusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBlock {
    pub id: String,
    pub source: ContextSource,
    pub source_id: String,
    pub trust: TrustLevel,
    pub media_type: String,
    pub byte_length: usize,
    pub sha256: String,
    pub transform_chain: Vec<String>,
    pub data_base64: String,
}

impl ContextBlock {
    #[must_use]
    pub fn is_trusted(&self) -> bool {
        self.trust == TrustLevel::Trusted
    }
}
