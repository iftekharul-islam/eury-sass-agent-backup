use agent_policy::audit::{AuditEvent, AuditQueue};
use agent_policy::engine::PolicyEngine;
use agent_policy::presets::standard_preset;
use agent_policy::schema::ToolClass;
use agent_policy::store::GrantStore;
use serde_json::json;
use std::error::Error;

#[test]
fn test_audit_hash_chain_is_intact_after_appends() -> Result<(), Box<dyn Error>> {
    let mut queue = AuditQueue::new_in_memory()?;

    queue.append_event(AuditEvent::new(
        "grant_decision".to_string(),
        json!({"tool": "test", "decision": "allow"}),
    ))?;
    queue.append_event(AuditEvent::new(
        "grant_decision".to_string(),
        json!({"tool": "test2", "decision": "deny"}),
    ))?;

    let events = queue.list_events()?;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].seq, 1, "seq starts at 1");
    assert_eq!(events[1].seq, 2, "seq increments");
    assert!(queue.verify_chain()?, "a freshly written chain must verify");

    Ok(())
}

#[test]
fn test_audit_chain_detects_tampering() -> Result<(), Box<dyn Error>> {
    // A shared in-memory database, so the tampering connection and the
    // verifying queue see the same rows without touching the filesystem
    // (this crate is outside the sandbox boundary that owns file access).
    let uri = format!("file:audit_tamper_{}?mode=memory&cache=shared", uuid::Uuid::now_v7());
    let keep_alive = rusqlite::Connection::open(&uri)?;

    let mut queue = AuditQueue::new(&uri)?;
    queue.append_event(AuditEvent::new("policy_decision".to_string(), json!({"a": 1})))?;
    queue.append_event(AuditEvent::new("policy_decision".to_string(), json!({"b": 2})))?;
    assert!(queue.verify_chain()?, "a freshly written chain must verify");

    // Edit a payload behind the log's back — the recorded `hash` for that row
    // no longer matches its contents, which is exactly what the chain exists
    // to make detectable.
    keep_alive.execute("UPDATE audit_events SET payload = ?1 WHERE seq = 1", ["{\"a\":999}"])?;

    assert!(!queue.verify_chain()?, "an edited payload must break chain verification");

    Ok(())
}

#[test]
fn test_policy_engine_audits_every_evaluated_tool_call() -> Result<(), Box<dyn Error>> {
    let engine = PolicyEngine::new(standard_preset(), GrantStore::new_in_memory()?)?;
    let run_id = "run-audit-1";

    // An allowed read, an approval-gated write, and an outright-denied command.
    engine.evaluate(
        &ToolClass::Read,
        "read_file",
        &json!({"path": "src/main.rs"}),
        None,
        &agent_types::requests::RunMode::Agent,
        Some(run_id),
    );
    engine.evaluate(
        &ToolClass::Write,
        "write_file",
        &json!({"path": "src/main.rs", "content": "x"}),
        None,
        &agent_types::requests::RunMode::Agent,
        Some(run_id),
    );
    engine.evaluate(
        &ToolClass::Execute,
        "run_command",
        &json!({"command": "sudo rm -rf /"}),
        None,
        &agent_types::requests::RunMode::Agent,
        Some(run_id),
    );

    engine.with_audit(|audit| -> Result<(), Box<dyn Error>> {
        let events = audit.list_events()?;
        assert_eq!(events.len(), 3, "every evaluate() call must be audited");
        assert!(audit.verify_chain()?, "the live-written chain must verify");

        for event in &events {
            assert_eq!(event.event_type, "policy_decision");
            assert_eq!(event.run_id, run_id);
        }

        let decisions: Vec<&str> =
            events.iter().filter_map(|e| e.payload["decision"].as_str()).collect();
        assert_eq!(decisions, vec!["Allow", "NeedsApproval", "Deny"]);

        let denied = &events[2];
        assert_eq!(denied.severity, "warning", "a denial is recorded at warning severity");
        assert_eq!(denied.payload["risk"], "Forbidden");
        assert_eq!(denied.payload["toolName"], "run_command");

        Ok(())
    })?;

    Ok(())
}
