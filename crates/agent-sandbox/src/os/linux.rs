//! Linux sandbox capability probing.
//!
//! The documented mechanism is Landlock + seccomp/no-new-privileges. A real
//! probe needs a process to restrict *itself* via `landlock_restrict_self`
//! before attempting the forbidden operation (self-restricting a live,
//! already-running process is exactly what Landlock is for), which in
//! practice means either:
//!
//! - calling the raw `landlock_create_ruleset`/`landlock_add_rule`/
//!   `landlock_restrict_self` syscalls directly, or
//! - using a safe wrapper crate whose *public* API still requires calling
//!   `std::os::unix::process::CommandExt::pre_exec` (itself an `unsafe fn`)
//!   to restrict a spawned child before it execs the probe target.
//!
//! Both routes need `unsafe` at the call site in this crate, and the
//! workspace lints (`unsafe_code = "forbid"`, [Cargo.toml](../../../Cargo.toml))
//! don't allow that here — `forbid` can't be downgraded by a local
//! `#[allow]`. Doing this for real needs either a separate helper binary
//! that self-restricts in its own `main()` before touching any file (and is
//! spawned as a subprocess, the same shape as the macOS `sandbox-exec`
//! probes), or a dedicated crate carved out from the `forbid(unsafe_code)`
//! lint to hold the syscall wrapper. Neither exists yet, so — consistent
//! with the whole point of this audit fix — we report the gap honestly
//! (`passed: false`, `probe_not_implemented`) instead of assuming Landlock
//! is enforcing anything.

use super::{SandboxCapabilities, SandboxProbe};

const MECHANISM: &str = "landlock_seccomp";

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
        os: "linux".to_string(),
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
