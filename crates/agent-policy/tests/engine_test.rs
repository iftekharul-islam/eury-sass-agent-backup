use agent_policy::engine::{PolicyEngine, RiskClassification};
use agent_policy::presets::standard_preset;
use agent_policy::schema::{Decision, GrantScope, ToolClass};
use agent_policy::store::GrantStore;
use agent_types::requests::RunMode;
use serde_json::json;
use std::error::Error;

#[test]
fn test_deny_by_default() -> Result<(), Box<dyn Error>> {
    let policy = standard_preset();
    let grant_store = GrantStore::new_in_memory()?;

    let engine = PolicyEngine::new(policy, grant_store)?;

    // standard_preset usually requires approval for Execute and Write
    let decision = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "/tmp/test"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(decision, Decision::NeedsApproval);

    let decision_exec = engine.evaluate(
        &ToolClass::Execute,
        "run_command",
        &json!({"command": "ls"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(decision_exec, Decision::NeedsApproval);

    let decision_read = engine.evaluate(
        &ToolClass::Read,
        "read_file",
        &json!({"path": "/tmp/test"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(decision_read, Decision::Allow);

    Ok(())
}

#[test]
fn test_shape_hash_is_deterministic_and_argument_sensitive() {
    let args_a = json!({"path": "/tmp/a", "content": "x"});
    let args_a_reordered = json!({"content": "x", "path": "/tmp/a"});
    let args_b = json!({"path": "/tmp/b", "content": "x"});

    assert_eq!(
        PolicyEngine::shape_hash("write_file", &args_a),
        PolicyEngine::shape_hash("write_file", &args_a_reordered),
        "key order must not affect the shape hash"
    );
    assert_ne!(
        PolicyEngine::shape_hash("write_file", &args_a),
        PolicyEngine::shape_hash("write_file", &args_b),
        "different arguments must produce different shape hashes"
    );
    assert_ne!(
        PolicyEngine::shape_hash("write_file", &args_a),
        PolicyEngine::shape_hash("other_tool", &args_a),
        "different tool names must produce different shape hashes"
    );
}

#[test]
fn test_grant_unblocks_only_the_exact_shape_it_was_recorded_for() -> Result<(), Box<dyn Error>> {
    let policy = standard_preset();
    let grant_store = GrantStore::new_in_memory()?;
    let engine = PolicyEngine::new(policy, grant_store)?;
    let run_id = "run-123";
    let args = json!({"path": "/tmp/test"});
    let other_args = json!({"path": "/tmp/other"});

    let decision = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &args,
        None,
        &agent_types::requests::RunMode::Agent,
        Some(run_id),
    );
    assert_eq!(decision, Decision::NeedsApproval);

    engine.record_grant(ToolClass::Write, "write_file", &args, GrantScope::Run, Some(run_id))?;

    // The exact shape that was granted is now allowed within this run.
    let decision_after_grant = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &args,
        None,
        &agent_types::requests::RunMode::Agent,
        Some(run_id),
    );
    assert_eq!(decision_after_grant, Decision::Allow);

    // A different argument shape for the same tool is NOT covered by the
    // grant — this is the "never-widen" guarantee the shape hash exists for.
    let decision_other_shape = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &other_args,
        None,
        &agent_types::requests::RunMode::Agent,
        Some(run_id),
    );
    assert_eq!(decision_other_shape, Decision::NeedsApproval);

    // A `Run`-scoped grant does not carry over to a different run.
    let decision_other_run = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &args,
        None,
        &agent_types::requests::RunMode::Agent,
        Some("run-456"),
    );
    assert_eq!(decision_other_run, Decision::NeedsApproval);

    Ok(())
}

#[test]
fn test_deny_globs_blocks_matching_write_paths() -> Result<(), Box<dyn Error>> {
    let policy = standard_preset(); // deny_globs includes ".env*", "**/*.pem", "~/.ssh/**"
    let grant_store = GrantStore::new_in_memory()?;
    let engine = PolicyEngine::new(policy, grant_store)?;

    let denied_env = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "src/.env", "content": "SECRET=1"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(denied_env, Decision::Deny);

    let denied_pem = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "certs/server.pem", "content": "..."}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(denied_pem, Decision::Deny);

    let allowed = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "src/main.rs", "content": "fn main() {}"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(allowed, Decision::NeedsApproval); // standard preset: Write needs approval, but not denied

    Ok(())
}

#[test]
fn test_max_file_write_bytes_denies_oversized_writes() -> Result<(), Box<dyn Error>> {
    let mut policy = standard_preset();
    policy.filesystem.max_file_write_bytes = Some(4);
    let grant_store = GrantStore::new_in_memory()?;
    let engine = PolicyEngine::new(policy, grant_store)?;

    let too_big = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "src/main.rs", "content": "way too long"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(too_big, Decision::Deny);

    let within_limit = engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "src/main.rs", "content": "ok"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_ne!(within_limit, Decision::Deny);

    Ok(())
}

#[test]
fn test_deny_patterns_blocks_matching_commands() -> Result<(), Box<dyn Error>> {
    let policy = standard_preset(); // deny_patterns includes \brm\s+-rf\b, \bsudo\b
    let grant_store = GrantStore::new_in_memory()?;
    let engine = PolicyEngine::new(policy, grant_store)?;

    let denied = engine.evaluate(
        &ToolClass::Execute,
        "run_command",
        &json!({"command": "sudo apt-get update"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(denied, Decision::Deny);

    let allowed = engine.evaluate(
        &ToolClass::Execute,
        "run_command",
        &json!({"command": "ls -la"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_ne!(allowed, Decision::Deny);

    Ok(())
}

#[test]
fn test_network_during_execute_off_escalates_egress_commands_to_approval()
-> Result<(), Box<dyn Error>> {
    let mut policy = agent_policy::presets::permissive_preset();
    policy.tools.default_decision.insert(ToolClass::Execute, Decision::Allow);
    // Isolate the egress-escalation path from the unconditional
    // require_approval-for-Execute rule the presets otherwise set.
    policy.tools.require_approval = None;
    policy.commands.network_during_execute = false;
    let grant_store = GrantStore::new_in_memory()?;
    let engine = PolicyEngine::new(policy, grant_store)?;

    let escalated = engine.evaluate(
        &ToolClass::Execute,
        "run_command",
        &json!({"command": "curl https://example.com"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(escalated, Decision::NeedsApproval);

    let not_escalated = engine.evaluate(
        &ToolClass::Execute,
        "run_command",
        &json!({"command": "ls -la"}),
        None,
        &agent_types::requests::RunMode::Agent,
        None,
    );
    assert_eq!(not_escalated, Decision::Allow);

    Ok(())
}

fn workspace(
    trust_state: agent_sandbox::workspace::TrustState,
) -> agent_sandbox::workspace::Workspace {
    agent_sandbox::workspace::Workspace {
        id: uuid::Uuid::now_v7(),
        root_path: std::path::PathBuf::from("/tmp/ws"),
        trust_state,
    }
}

#[test]
fn test_untrusted_workspace_is_read_only() -> Result<(), Box<dyn Error>> {
    // Deliberately permissive policy: trust enforcement must not be
    // overridable by policy configuration.
    let mut policy = agent_policy::presets::permissive_preset();
    policy.tools.require_approval = None;
    for class in [ToolClass::Read, ToolClass::Write, ToolClass::Execute, ToolClass::Network] {
        policy.tools.default_decision.insert(class, Decision::Allow);
    }
    let engine = PolicyEngine::new(policy, GrantStore::new_in_memory()?)?;
    let untrusted = workspace(agent_sandbox::workspace::TrustState::Untrusted);

    // Reads still work in an untrusted workspace...
    assert_eq!(
        engine.evaluate(
            &ToolClass::Read,
            "read_file",
            &json!({"path": "src/main.rs"}),
            Some(&untrusted),
            &agent_types::requests::RunMode::Agent,
            None,
        ),
        Decision::Allow
    );

    // ...but writes, execution, network, and MCP are all denied outright.
    for (class, name, args) in [
        (ToolClass::Write, "write_file", json!({"path": "a.txt", "content": "x"})),
        (ToolClass::Execute, "run_command", json!({"command": "ls"})),
        (ToolClass::Network, "web_fetch", json!({"url": "https://example.com"})),
        (ToolClass::Mcp, "mcp_call", json!({})),
    ] {
        assert_eq!(
            engine.evaluate(
                &class,
                name,
                &args,
                Some(&untrusted),
                &agent_types::requests::RunMode::Agent,
                None,
            ),
            Decision::Deny,
            "{name} must be denied in an untrusted workspace"
        );
    }

    Ok(())
}

#[test]
fn test_trusted_workspace_allows_the_full_catalog_subject_to_policy() -> Result<(), Box<dyn Error>>
{
    let mut policy = agent_policy::presets::permissive_preset();
    policy.tools.require_approval = None;
    policy.tools.default_decision.insert(ToolClass::Execute, Decision::Allow);
    let engine = PolicyEngine::new(policy, GrantStore::new_in_memory()?)?;
    let trusted = workspace(agent_sandbox::workspace::TrustState::Trusted);

    assert_eq!(
        engine.evaluate(
            &ToolClass::Execute,
            "run_command",
            &json!({"command": "ls -la"}),
            Some(&trusted),
            &agent_types::requests::RunMode::Agent,
            None,
        ),
        Decision::Allow
    );

    Ok(())
}

fn test_engine() -> Result<PolicyEngine, Box<dyn Error>> {
    PolicyEngine::new(standard_preset(), GrantStore::new_in_memory()?).map_err(Into::into)
}

#[test]
fn test_classify_risk_read_and_network_are_never_forbidden() -> Result<(), Box<dyn Error>> {
    let engine = test_engine()?;
    assert_eq!(
        engine.classify_risk(&ToolClass::Read, "read_file", &json!({"path": "any"})),
        RiskClassification::Low
    );
    assert_eq!(
        engine.classify_risk(&ToolClass::Network, "web_fetch", &json!({"url": "any"})),
        RiskClassification::Medium
    );
    Ok(())
}

#[test]
fn test_classify_risk_execute_distinguishes_ls_from_rm_rf() -> Result<(), Box<dyn Error>> {
    let engine = test_engine()?;
    assert_eq!(
        engine.classify_risk(&ToolClass::Execute, "run_command", &json!({"command": "ls -la"})),
        RiskClassification::Elevated
    );
    assert_eq!(
        engine.classify_risk(&ToolClass::Execute, "run_command", &json!({"command": "rm -rf /"})),
        RiskClassification::Forbidden
    );
    assert_eq!(
        engine.classify_risk(
            &ToolClass::Execute,
            "run_command",
            &json!({"command": "rm -fr /tmp/x"})
        ),
        RiskClassification::Forbidden,
        "flag order (-fr vs -rf) must not matter"
    );
    assert_eq!(
        engine.classify_risk(
            &ToolClass::Execute,
            "run_command",
            &json!({"command": "sudo apt-get update"})
        ),
        RiskClassification::Forbidden
    );
    Ok(())
}

#[test]
fn test_classify_risk_execute_flags_installs_and_shell_metacharacters_as_critical()
-> Result<(), Box<dyn Error>> {
    let engine = test_engine()?;
    assert_eq!(
        engine.classify_risk(
            &ToolClass::Execute,
            "run_command",
            &json!({"command": "npm install left-pad"})
        ),
        RiskClassification::Critical
    );
    assert_eq!(
        engine.classify_risk(
            &ToolClass::Execute,
            "run_command",
            &json!({"command": "echo hi && ls"})
        ),
        RiskClassification::Critical,
        "shell metacharacters (&&) widen the attack surface beyond a plain argv command"
    );
    Ok(())
}

#[test]
fn test_classify_risk_write_flags_path_traversal_as_critical() -> Result<(), Box<dyn Error>> {
    let engine = test_engine()?;
    assert_eq!(
        engine.classify_risk(
            &ToolClass::Write,
            "write_file",
            &json!({"path": "src/main.rs", "content": "fn main() {}"})
        ),
        RiskClassification::Medium
    );
    assert_eq!(
        engine.classify_risk(
            &ToolClass::Write,
            "write_file",
            &json!({"path": "../../etc/passwd", "content": "x"})
        ),
        RiskClassification::Critical
    );
    Ok(())
}

#[test]
fn test_classify_risk_write_outside_workspace_respects_allow_outside_workspace()
-> Result<(), Box<dyn Error>> {
    let mut policy = standard_preset();
    policy.filesystem.allow_outside_workspace = false;
    let engine = PolicyEngine::new(policy, GrantStore::new_in_memory()?)?;
    assert_eq!(
        engine.classify_risk(&ToolClass::WriteOutsideWorkspace, "write_file", &json!({})),
        RiskClassification::Forbidden
    );

    let mut policy_allowed = standard_preset();
    policy_allowed.filesystem.allow_outside_workspace = true;
    let engine_allowed = PolicyEngine::new(policy_allowed, GrantStore::new_in_memory()?)?;
    assert_eq!(
        engine_allowed.classify_risk(&ToolClass::WriteOutsideWorkspace, "write_file", &json!({})),
        RiskClassification::Critical
    );

    Ok(())
}

#[test]
fn session_execute_grant_covers_any_command_shape() -> Result<(), Box<dyn Error>> {
    let engine = PolicyEngine::new(standard_preset(), GrantStore::new_in_memory()?)?;
    let first = json!({"command": "node --version"});
    let second = json!({"command": "pnpm install"});

    engine.record_grant(
        ToolClass::Execute,
        "run_command",
        &first,
        GrantScope::Session,
        None,
    )?;

    assert_eq!(
        engine.evaluate(
            &ToolClass::Execute,
            "run_command",
            &first,
            None,
            &RunMode::Agent,
            Some("run-a"),
        ),
        Decision::Allow,
    );
    assert_eq!(
        engine.evaluate(
            &ToolClass::Execute,
            "run_command",
            &second,
            None,
            &RunMode::Agent,
            Some("run-b"),
        ),
        Decision::Allow,
        "a session grant must cover later commands in a new run",
    );
    Ok(())
}
