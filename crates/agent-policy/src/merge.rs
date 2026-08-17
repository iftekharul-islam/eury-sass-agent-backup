use crate::schema::{
    AgentPolicy, CommandsPolicy, CostPolicy, DataPolicy, Decision, FilesystemPolicy, GrantScope,
    McpPolicy, ModelsPolicy, NetworkPolicy, TelemetryPolicy, ToolsPolicy, UpdatePolicy,
    WorkspacePolicy,
};
use std::cmp;
use std::collections::HashSet;

#[must_use]
pub fn merge_policies(
    base: &WorkspacePolicy,
    override_policy: &WorkspacePolicy,
) -> WorkspacePolicy {
    WorkspacePolicy {
        schema_version: base.schema_version,
        version: base.version,
        label: base.label.clone(), // Or maybe keep base label?
        policy_max_age_hours: cmp::min(
            base.policy_max_age_hours,
            override_policy.policy_max_age_hours,
        ),

        tools: merge_tools(&base.tools, &override_policy.tools),
        models: merge_models(&base.models, &override_policy.models),
        filesystem: merge_fs(&base.filesystem, &override_policy.filesystem),
        commands: merge_commands(&base.commands, &override_policy.commands),
        network: merge_network(&base.network, &override_policy.network),
        mcp: merge_mcp(&base.mcp, &override_policy.mcp),
        cost: merge_cost(&base.cost, &override_policy.cost),
        data: merge_data(&base.data, &override_policy.data),
        telemetry: merge_telemetry(&base.telemetry, &override_policy.telemetry),
        agent: merge_agent(&base.agent, &override_policy.agent),
        update: merge_update(&base.update, &override_policy.update),
    }
}

fn union_lists<T: Clone + Eq + std::hash::Hash>(a: &[T], b: &[T]) -> Vec<T> {
    let mut set: HashSet<T> = a.iter().cloned().collect();
    set.extend(b.iter().cloned());
    set.into_iter().collect()
}

fn intersect_lists<T: Clone + Eq + std::hash::Hash>(
    a: Option<&Vec<T>>,
    b: Option<&Vec<T>>,
) -> Option<Vec<T>> {
    match (a, b) {
        (Some(a_list), Some(b_list)) => {
            let a_set: HashSet<T> = a_list.iter().cloned().collect();
            let b_set: HashSet<T> = b_list.iter().cloned().collect();
            Some(a_set.intersection(&b_set).cloned().collect())
        }
        (Some(list), None) | (None, Some(list)) => Some(list.clone()),
        (None, None) => None,
    }
}

fn merge_tools(base: &ToolsPolicy, over: &ToolsPolicy) -> ToolsPolicy {
    let mut default_decision = base.default_decision.clone();
    for (k, v) in &over.default_decision {
        let current = default_decision.get(k).copied().unwrap_or(Decision::Allow);
        default_decision.insert(*k, cmp::max(current, *v)); // Deny > NeedsApproval > Allow
    }

    let max_grant_scope = match (&base.max_grant_scope, &over.max_grant_scope) {
        (Some(b), Some(o)) => {
            let mut merged = b.clone();
            for (k, v) in o {
                let current = merged.get(k).copied().unwrap_or(GrantScope::Always);
                merged.insert(*k, cmp::min(current, *v)); // Once < Run < Session < Always
            }
            Some(merged)
        }
        (Some(m), None) | (None, Some(m)) => Some(m.clone()),
        (None, None) => None,
    };

    ToolsPolicy {
        allow: intersect_lists(base.allow.as_ref(), over.allow.as_ref()),
        deny: Some(union_lists(
            &base.deny.clone().unwrap_or_default(),
            &over.deny.clone().unwrap_or_default(),
        )),
        require_approval: Some(union_lists(
            &base.require_approval.clone().unwrap_or_default(),
            &over.require_approval.clone().unwrap_or_default(),
        )),
        default_decision,
        max_grant_scope,
    }
}

fn merge_models(base: &ModelsPolicy, over: &ModelsPolicy) -> ModelsPolicy {
    ModelsPolicy {
        allow: intersect_lists(base.allow.as_ref(), over.allow.as_ref()),
        deny: Some(union_lists(
            &base.deny.clone().unwrap_or_default(),
            &over.deny.clone().unwrap_or_default(),
        )),
        require_managed_gateway: Some(
            base.require_managed_gateway.unwrap_or(false)
                || over.require_managed_gateway.unwrap_or(false),
        ),
        allow_byok: Some(base.allow_byok.unwrap_or(true) && over.allow_byok.unwrap_or(true)),
    }
}

fn merge_fs(base: &FilesystemPolicy, over: &FilesystemPolicy) -> FilesystemPolicy {
    FilesystemPolicy {
        deny_globs: union_lists(&base.deny_globs, &over.deny_globs),
        allow_outside_workspace: base.allow_outside_workspace && over.allow_outside_workspace,
        max_file_write_bytes: match (base.max_file_write_bytes, over.max_file_write_bytes) {
            (Some(a), Some(b)) => Some(cmp::min(a, b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        },
        redact_secrets_in_context: base.redact_secrets_in_context || over.redact_secrets_in_context,
    }
}

fn merge_commands(base: &CommandsPolicy, over: &CommandsPolicy) -> CommandsPolicy {
    CommandsPolicy {
        deny_patterns: union_lists(&base.deny_patterns, &over.deny_patterns),
        allow_patterns: intersect_lists(base.allow_patterns.as_ref(), over.allow_patterns.as_ref()),
        max_runtime_seconds: cmp::min(base.max_runtime_seconds, over.max_runtime_seconds),
        network_during_execute: base.network_during_execute && over.network_during_execute,
    }
}

fn merge_network(base: &NetworkPolicy, over: &NetworkPolicy) -> NetworkPolicy {
    NetworkPolicy {
        block_web_fetch: base.block_web_fetch || over.block_web_fetch,
        allow_image_generation: Some(
            base.allow_image_generation.unwrap_or(true)
                && over.allow_image_generation.unwrap_or(true),
        ),
        allowed_hosts: intersect_lists(base.allowed_hosts.as_ref(), over.allowed_hosts.as_ref()),
        blocked_hosts: Some(union_lists(
            &base.blocked_hosts.clone().unwrap_or_default(),
            &over.blocked_hosts.clone().unwrap_or_default(),
        )),
    }
}

fn merge_mcp(base: &McpPolicy, over: &McpPolicy) -> McpPolicy {
    McpPolicy {
        enabled: base.enabled && over.enabled,
        allowed_servers: intersect_lists(
            base.allowed_servers.as_ref(),
            over.allowed_servers.as_ref(),
        ),
        blocked_servers: Some(union_lists(
            &base.blocked_servers.clone().unwrap_or_default(),
            &over.blocked_servers.clone().unwrap_or_default(),
        )),
        allow_local_servers: base.allow_local_servers && over.allow_local_servers,
        allow_workspace_config: base.allow_workspace_config && over.allow_workspace_config,
        require_signed_manifest: base.require_signed_manifest || over.require_signed_manifest,
        // scope narrowing
        require_approval: if base.require_approval == "once" || over.require_approval == "once" {
            "once".into()
        } else if base.require_approval == "run" || over.require_approval == "run" {
            "run".into()
        } else {
            "session".into()
        },
        allowed_transports: intersect_lists(
            Some(&base.allowed_transports),
            Some(&over.allowed_transports),
        )
        .unwrap_or_default(),
        registry_url: base.registry_url.clone().or(over.registry_url.clone()),
    }
}

fn merge_cost(base: &CostPolicy, over: &CostPolicy) -> CostPolicy {
    CostPolicy {
        max_cost_per_run_usd_micros: match (
            base.max_cost_per_run_usd_micros,
            over.max_cost_per_run_usd_micros,
        ) {
            (Some(a), Some(b)) => Some(cmp::min(a, b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        },
        max_cost_per_day_usd_micros: match (
            base.max_cost_per_day_usd_micros,
            over.max_cost_per_day_usd_micros,
        ) {
            (Some(a), Some(b)) => Some(cmp::min(a, b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        },
        max_tokens_per_run: match (base.max_tokens_per_run, over.max_tokens_per_run) {
            (Some(a), Some(b)) => Some(cmp::min(a, b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        },
    }
}

fn merge_data(base: &DataPolicy, over: &DataPolicy) -> DataPolicy {
    DataPolicy {
        audit_upload_required: base.audit_upload_required || over.audit_upload_required,
        audit_include_paths: base.audit_include_paths || over.audit_include_paths,
        audit_payload_mode: if base.audit_payload_mode == "full_payload"
            || over.audit_payload_mode == "full_payload"
        {
            "full_payload".into()
        } else {
            "metadata_only".into()
        },
        allow_cloud_sync: base.allow_cloud_sync && over.allow_cloud_sync,
        allow_screenshots: base.allow_screenshots && over.allow_screenshots,
        residency: base.residency.clone().or(over.residency.clone()), // Base is authoritative
    }
}

fn merge_telemetry(base: &TelemetryPolicy, over: &TelemetryPolicy) -> TelemetryPolicy {
    TelemetryPolicy {
        mode: if base.mode == "off" || over.mode == "off" {
            "off".into()
        } else if base.mode == "required" || over.mode == "required" {
            "required".into()
        } else {
            "user_choice".into()
        },
        crash_reports: base.crash_reports && over.crash_reports, // off is more restrictive? Actually privacy: off is more restrictive.
        endpoint: base.endpoint.clone().or(over.endpoint.clone()),
    }
}

fn merge_agent(base: &AgentPolicy, over: &AgentPolicy) -> AgentPolicy {
    AgentPolicy {
        allowed_modes: intersect_lists(Some(&base.allowed_modes), Some(&over.allowed_modes))
            .unwrap_or_default(),
        allow_sub_agents: base.allow_sub_agents && over.allow_sub_agents,
        max_parallel_tools: cmp::min(base.max_parallel_tools, over.max_parallel_tools),
        max_turns_per_run: cmp::min(base.max_turns_per_run, over.max_turns_per_run),
        require_plan_before_write: base.require_plan_before_write || over.require_plan_before_write,
    }
}

fn merge_update(base: &UpdatePolicy, over: &UpdatePolicy) -> UpdatePolicy {
    UpdatePolicy {
        min_version: base.min_version.clone().or(over.min_version.clone()), // For strings, we can't easily min/max without semver. Let's just keep base or over.
        channel: base.channel.clone().or(over.channel.clone()),
        auto_update: base.auto_update || over.auto_update, // require auto_update is more restrictive?
    }
}
