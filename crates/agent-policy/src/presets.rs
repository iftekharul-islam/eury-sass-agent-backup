use crate::schema::{
    AgentPolicy, CommandsPolicy, CostPolicy, DataPolicy, Decision, FilesystemPolicy, GrantScope,
    McpPolicy, ModelsPolicy, NetworkPolicy, TelemetryPolicy, ToolClass, ToolsPolicy, UpdatePolicy,
    WorkspacePolicy,
};
use std::collections::HashMap;

#[must_use]
pub fn permissive_preset() -> WorkspacePolicy {
    let mut default_decision = HashMap::new();
    default_decision.insert(ToolClass::Read, Decision::Allow);
    default_decision.insert(ToolClass::Write, Decision::Allow);
    default_decision.insert(ToolClass::Execute, Decision::NeedsApproval);
    default_decision.insert(ToolClass::Network, Decision::NeedsApproval);
    default_decision.insert(ToolClass::Mcp, Decision::NeedsApproval);
    default_decision.insert(ToolClass::WriteOutsideWorkspace, Decision::Deny);

    let mut max_grant_scope = HashMap::new();
    max_grant_scope.insert(ToolClass::Execute, GrantScope::Session);
    max_grant_scope.insert(ToolClass::Network, GrantScope::Session);

    WorkspacePolicy {
        schema_version: 1,
        version: 1,
        label: Some("Permissive".into()),
        policy_max_age_hours: 168,
        tools: ToolsPolicy {
            allow: None,
            deny: None,
            require_approval: Some(vec![ToolClass::Execute, ToolClass::Network]),
            default_decision,
            max_grant_scope: Some(max_grant_scope),
        },
        models: ModelsPolicy {
            allow: None,
            deny: None,
            require_managed_gateway: Some(false),
            allow_byok: Some(true),
        },
        filesystem: FilesystemPolicy {
            deny_globs: vec![],
            allow_outside_workspace: false,
            max_file_write_bytes: Some(50 * 1024 * 1024),
            redact_secrets_in_context: false,
        },
        commands: CommandsPolicy {
            deny_patterns: vec![],
            allow_patterns: None,
            max_runtime_seconds: 600,
            network_during_execute: true,
        },
        network: NetworkPolicy {
            block_web_fetch: false,
            allow_image_generation: Some(true),
            allowed_hosts: None,
            blocked_hosts: None,
        },
        mcp: McpPolicy {
            enabled: true,
            allowed_servers: None,
            blocked_servers: None,
            allow_local_servers: true,
            allow_workspace_config: true,
            require_signed_manifest: false,
            require_approval: "once".into(),
            allowed_transports: vec!["stdio".into(), "https".into()],
            registry_url: None,
        },
        cost: CostPolicy {
            max_cost_per_run_usd_micros: None,
            max_cost_per_day_usd_micros: None,
            max_tokens_per_run: None,
        },
        data: DataPolicy {
            audit_upload_required: false,
            audit_include_paths: true,
            audit_payload_mode: "metadata_only".into(),
            allow_cloud_sync: true,
            allow_screenshots: true,
            residency: None,
        },
        telemetry: TelemetryPolicy {
            mode: "user_choice".into(),
            crash_reports: true,
            endpoint: None,
        },
        agent: AgentPolicy {
            allowed_modes: vec![
                "chat".into(),
                "agent".into(),
                "plan".into(),
                "ask".into(),
                "build".into(),
            ],
            allow_sub_agents: true,
            max_parallel_tools: 10,
            max_turns_per_run: 100,
            require_plan_before_write: false,
        },
        update: UpdatePolicy {
            min_version: None,
            channel: Some("stable".into()),
            auto_update: true,
        },
    }
}

#[must_use]
pub fn standard_preset() -> WorkspacePolicy {
    let mut policy = permissive_preset();
    policy.label = Some("Standard".into());

    let mut default_decision = HashMap::new();
    default_decision.insert(ToolClass::Read, Decision::Allow);
    default_decision.insert(ToolClass::Write, Decision::NeedsApproval);
    default_decision.insert(ToolClass::Execute, Decision::NeedsApproval);
    default_decision.insert(ToolClass::Network, Decision::NeedsApproval);
    default_decision.insert(ToolClass::Mcp, Decision::NeedsApproval);
    default_decision.insert(ToolClass::WriteOutsideWorkspace, Decision::Deny);

    policy.tools.default_decision = default_decision;
    policy.tools.require_approval =
        Some(vec![ToolClass::Write, ToolClass::Execute, ToolClass::Network, ToolClass::Mcp]);

    policy.filesystem.deny_globs = vec![".env*".into(), "**/*.pem".into(), "~/.ssh/**".into()];
    policy.filesystem.redact_secrets_in_context = true;
    policy.commands.deny_patterns = vec!["\\brm\\s+-rf\\b".into(), "\\bsudo\\b".into()];

    policy
}

#[must_use]
pub fn strict_preset() -> WorkspacePolicy {
    let mut policy = standard_preset();
    policy.label = Some("Strict".into());

    policy.network.block_web_fetch = true;
    policy.mcp.allow_local_servers = false;
    policy.data.audit_upload_required = true;

    policy
}

#[must_use]
pub fn regulated_preset() -> WorkspacePolicy {
    let mut policy = strict_preset();
    policy.label = Some("Regulated".into());

    policy.models.require_managed_gateway = Some(true);
    policy.models.allow_byok = Some(false);
    policy.data.allow_cloud_sync = false;
    policy.data.allow_screenshots = false;
    policy.agent.require_plan_before_write = true;

    policy
}
