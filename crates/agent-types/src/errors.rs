use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "code", content = "message")]
pub enum AgentError {
    #[error("Internal agent error: {0}")]
    #[serde(rename = "EURY_AGENT_INTERNAL")]
    Internal(String),

    #[error("Provider rate limit exceeded: {0}")]
    #[serde(rename = "EURY_PROVIDER_RATE_LIMIT")]
    ProviderRateLimit(String),

    #[error("Provider network timeout: {0}")]
    #[serde(rename = "EURY_PROVIDER_TIMEOUT")]
    ProviderTimeout(String),

    #[error("Provider error: {0}")]
    #[serde(rename = "EURY_PROVIDER_ERROR")]
    ProviderError(String),

    #[error("Context window limit exceeded")]
    #[serde(rename = "EURY_CONTEXT_LIMIT")]
    ContextLimitExceeded,

    #[error("Run cancelled by user")]
    #[serde(rename = "EURY_RUN_CANCELLED")]
    RunCancelled,

    #[error("Invalid state transition: {0}")]
    #[serde(rename = "EURY_RUN_INVALID_TRANSITION")]
    InvalidTransition(String),

    #[error("Cost budget exceeded")]
    #[serde(rename = "EURY_BUDGET_EXCEEDED")]
    BudgetExceeded,

    #[error("Security policy violation: {0}")]
    #[serde(rename = "EURY_SECURITY_VIOLATION")]
    SecurityViolation(String),

    /// No usable session. Distinct from [`Self::SecurityViolation`]: the user
    /// has not done anything wrong, they just need to sign in again.
    #[error("Unauthorized: {0}")]
    #[serde(rename = "EURY_AUTH_UNAUTHORIZED")]
    Unauthorized(String),
}
