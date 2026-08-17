//! Windows sandbox capability probing.
//!
//! The documented mechanism is a restricted token plus a Job object (kill-on
//! -close, process/UI restrictions) and brokered handles for outside-root
//! access. None of that is implemented yet — building and, critically,
//! *verifying* it needs a real Windows machine (`CreateRestrictedToken`,
//! `CreateProcessAsUser`, Job object limit/network policy calls can't be
//! meaningfully authored or tested from this session's macOS environment).
//! Consistent with the whole point of this audit fix, we report the gap
//! honestly (`passed: false`, `probe_not_implemented`) instead of assuming
//! containment that was never built or verified.

use super::{SandboxCapabilities, SandboxProbe};

const MECHANISM: &str = "restricted_token_job_broker";

pub(super) fn probe() -> SandboxCapabilities {
    let probe_ids = [
        "outside_root_read_denied",
        "outside_root_write_denied",
        "process_tree_contained",
        "privilege_escalation_denied",
        "egress_denied",
    ];
    let probes: Vec<SandboxProbe> = probe_ids
        .iter()
        .map(|id| SandboxProbe {
            id: (*id).to_string(),
            passed: false,
            reason_code: Some("probe_not_implemented".to_string()),
        })
        .collect();

    SandboxCapabilities {
        schema_version: 1,
        os: "windows".to_string(),
        os_version: None,
        mechanism: Some(MECHANISM.to_string()),
        mechanism_version: None,
        filesystem_guard: false,
        process_isolation: false,
        egress_enforcement: false,
        outside_root_capabilities: false,
        privileged_tools_enabled: false,
        verified_at: chrono::Utc::now().to_rfc3339(),
        probes,
        reason_codes: Some(vec!["mechanism_unavailable".to_string()]),
    }
}
