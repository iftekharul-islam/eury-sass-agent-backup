use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolClass {
    Read,
    Write,
    Execute,
    Network,
    Mcp,
    WriteOutsideWorkspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GrantScope {
    Once,
    Run,
    Session,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum Decision {
    Allow,
    NeedsApproval,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolsPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<Vec<ToolClass>>,
    pub default_decision: HashMap<ToolClass, Decision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_grant_scope: Option<HashMap<ToolClass, GrantScope>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelsPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_managed_gateway: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_byok: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FilesystemPolicy {
    #[serde(default)]
    pub deny_globs: Vec<String>,
    pub allow_outside_workspace: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_write_bytes: Option<u64>,
    pub redact_secrets_in_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandsPolicy {
    #[serde(default)]
    pub deny_patterns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_patterns: Option<Vec<String>>,
    pub max_runtime_seconds: u32,
    pub network_during_execute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPolicy {
    pub block_web_fetch: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_image_generation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_hosts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_hosts: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// Field set mirrors the workspace-policy JSON schema's `mcp` block; splitting
// into a bitflag/enum would break the JSON contract policy files rely on.
#[allow(clippy::struct_excessive_bools)]
pub struct McpPolicy {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_servers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_servers: Option<Vec<String>>,
    pub allow_local_servers: bool,
    pub allow_workspace_config: bool,
    pub require_signed_manifest: bool,
    pub require_approval: String,
    pub allowed_transports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CostPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_run_usd_micros: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_day_usd_micros: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_run: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// Field set mirrors the workspace-policy JSON schema's `data` block; splitting
// into a bitflag/enum would break the JSON contract policy files rely on.
#[allow(clippy::struct_excessive_bools)]
pub struct DataPolicy {
    pub audit_upload_required: bool,
    pub audit_include_paths: bool,
    pub audit_payload_mode: String,
    pub allow_cloud_sync: bool,
    pub allow_screenshots: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryPolicy {
    pub mode: String,
    pub crash_reports: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPolicy {
    pub allowed_modes: Vec<String>,
    pub allow_sub_agents: bool,
    pub max_parallel_tools: u32,
    pub max_turns_per_run: u32,
    pub require_plan_before_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePolicy {
    pub schema_version: u32,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub policy_max_age_hours: u32,

    pub tools: ToolsPolicy,
    pub models: ModelsPolicy,
    pub filesystem: FilesystemPolicy,
    pub commands: CommandsPolicy,
    pub network: NetworkPolicy,
    pub mcp: McpPolicy,
    pub cost: CostPolicy,
    pub data: DataPolicy,
    pub telemetry: TelemetryPolicy,
    pub agent: AgentPolicy,
    pub update: UpdatePolicy,
}
