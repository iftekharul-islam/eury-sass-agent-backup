//! Exercises `agent_sandbox::os::probe_capabilities()` against the real
//! platform this test runs on — not a mock. On macOS this actually spawns
//! `sandbox-exec`-wrapped children and checks that forbidden filesystem/
//! egress operations were genuinely denied.

use agent_sandbox::os::probe_capabilities;

const EXPECTED_PROBE_IDS: [&str; 5] = [
    "outside_root_read_denied",
    "outside_root_write_denied",
    "process_tree_contained",
    "privilege_escalation_denied",
    "egress_denied",
];

#[test]
fn reports_the_expected_probe_set_and_schema_version() {
    let caps = probe_capabilities();
    assert_eq!(caps.schema_version, 1);
    assert_eq!(caps.probes.len(), EXPECTED_PROBE_IDS.len());
    for expected_id in EXPECTED_PROBE_IDS {
        assert!(
            caps.probes.iter().any(|p| p.id == expected_id),
            "missing probe {expected_id:?} in {:?}",
            caps.probes.iter().map(|p| &p.id).collect::<Vec<_>>()
        );
    }
}

#[test]
fn every_failing_probe_carries_a_reason_code() {
    let caps = probe_capabilities();
    for probe in &caps.probes {
        assert!(
            probe.passed || probe.reason_code.is_some(),
            "probe {:?} failed with no reason_code — never silently claim or silently fail",
            probe.id
        );
    }
}

#[test]
fn top_level_flags_are_consistent_with_their_probes() {
    let caps = probe_capabilities();
    let probe_passed = |id: &str| caps.probes.iter().any(|p| p.id == id && p.passed);

    assert_eq!(
        caps.filesystem_guard,
        probe_passed("outside_root_read_denied") && probe_passed("outside_root_write_denied"),
        "filesystem_guard must reflect the actual read+write denial probes, not be hardcoded"
    );
    assert_eq!(
        caps.egress_enforcement,
        probe_passed("egress_denied"),
        "egress_enforcement must reflect the actual egress probe"
    );
    // privileged_tools_enabled must never be true unless every probe passed —
    // this is the exact guarantee the original mocked implementation broke.
    if caps.privileged_tools_enabled {
        assert!(
            caps.probes.iter().all(|p| p.passed),
            "privileged_tools_enabled=true but not every probe passed: {:?}",
            caps.probes
        );
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_reports_seatbelt_and_actually_verifies_containment() {
    let caps = probe_capabilities();
    assert_eq!(caps.os, "macos");
    assert_eq!(caps.mechanism.as_deref(), Some("seatbelt"));

    // These are real, runtime-verified probes on macOS (via sandbox-exec) —
    // if this ever regresses to a hardcoded `true`, this test can't tell the
    // difference, but if the real probe implementation is present and
    // sandbox-exec is available (true on every macOS CI runner), containment
    // must actually hold.
    let read_probe = caps.probes.iter().find(|p| p.id == "outside_root_read_denied");
    let write_probe = caps.probes.iter().find(|p| p.id == "outside_root_write_denied");
    assert!(read_probe.is_some_and(|p| p.passed), "outside-root read must be genuinely denied");
    assert!(write_probe.is_some_and(|p| p.passed), "outside-root write must be genuinely denied");

    let fork_probe = caps.probes.iter().find(|p| p.id == "process_tree_contained");
    let escalation_probe = caps.probes.iter().find(|p| p.id == "privilege_escalation_denied");
    assert!(fork_probe.is_some_and(|p| p.passed), "a sandboxed process must not be able to fork");
    assert!(
        escalation_probe.is_some_and(|p| p.passed),
        "a sandboxed process must not be able to exec a setuid-root binary",
    );

    // With every probe verified, the app may enable privileged tools — this is
    // the flag the desktop's containment banner reads.
    assert!(caps.privileged_tools_enabled);
}

/// Every macOS probe must be a real experiment. A stub that reports
/// `probe_not_implemented` is what kept containment unverifiable — and left
/// the desktop showing its "containment could not be verified" banner on a
/// machine where Seatbelt works fine.
#[cfg(target_os = "macos")]
#[test]
fn macos_has_no_unimplemented_probes_left() {
    let caps = probe_capabilities();
    let stubs: Vec<&str> = caps
        .probes
        .iter()
        .filter(|p| p.reason_code.as_deref() == Some("probe_not_implemented"))
        .map(|p| p.id.as_str())
        .collect();
    assert!(stubs.is_empty(), "these probes are still stubs: {stubs:?}");
}

#[cfg(target_os = "linux")]
#[test]
fn linux_honestly_reports_unimplemented_rather_than_lying() {
    let caps = probe_capabilities();
    assert_eq!(caps.os, "linux");
    assert_eq!(caps.mechanism.as_deref(), Some("landlock_seccomp"));
    assert!(!caps.privileged_tools_enabled);
    assert!(caps.probes.iter().all(|p| !p.passed));
}

#[cfg(target_os = "windows")]
#[test]
fn windows_honestly_reports_unimplemented_rather_than_lying() {
    let caps = probe_capabilities();
    assert_eq!(caps.os, "windows");
    assert_eq!(caps.mechanism.as_deref(), Some("restricted_token_job_broker"));
    assert!(!caps.privileged_tools_enabled);
    assert!(caps.probes.iter().all(|p| !p.passed));
}
