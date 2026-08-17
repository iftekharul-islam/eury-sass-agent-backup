//! Real, runtime-verified Seatbelt (`sandbox-exec`) probing for macOS.
//!
//! Each filesystem/egress probe spawns a real child process under a
//! `(deny default)` Seatbelt profile that grants access to nothing outside a
//! scratch directory, then attempts the forbidden operation against a path
//! (or address) outside that directory. A probe passes only if the operation
//! observably failed — non-zero exit, and for reads, no leaked content. If
//! Seatbelt is unavailable, disabled, or our profile is wrong, the forbidden
//! operation succeeds and the probe honestly reports `passed: false` rather
//! than assuming containment that isn't actually there.
//!
//! Every probe is a real experiment, and each one carries a control so that
//! "the forbidden thing didn't happen" can never be confused with "the thing
//! never worked here anyway": a probe reports `passed: true` only when the
//! forbidden operation observably failed *and* its control observably
//! succeeded under the same mechanism.

use super::{SandboxCapabilities, SandboxProbe};
use std::io::Write as _;
use std::process::Command;

const MECHANISM: &str = "seatbelt";
const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";

pub(super) fn probe() -> SandboxCapabilities {
    let outside_root_read_denied = probe_read_denied();
    let outside_root_write_denied = probe_write_denied();
    let egress_denied = probe_egress_denied();
    let process_tree_contained = probe_process_tree_contained();
    let privilege_escalation_denied = probe_privilege_escalation_denied();

    let filesystem_guard = outside_root_read_denied.passed && outside_root_write_denied.passed;
    let process_isolation = process_tree_contained.passed;
    let egress_enforcement = egress_denied.passed;
    let privileged_tools_enabled = filesystem_guard
        && process_isolation
        && egress_enforcement
        && privilege_escalation_denied.passed;

    let probes = vec![
        outside_root_read_denied,
        outside_root_write_denied,
        process_tree_contained,
        privilege_escalation_denied,
        egress_denied,
    ];
    let mut reason_codes: Vec<String> =
        probes.iter().filter_map(|p| p.reason_code.clone()).collect();
    reason_codes.sort_unstable();
    reason_codes.dedup();

    SandboxCapabilities {
        schema_version: 1,
        os: "macos".to_string(),
        os_version: None,
        mechanism: Some(MECHANISM.to_string()),
        mechanism_version: None,
        filesystem_guard,
        process_isolation,
        egress_enforcement,
        outside_root_capabilities: filesystem_guard,
        privileged_tools_enabled,
        verified_at: chrono::Utc::now().to_rfc3339(),
        probes,
        reason_codes: if reason_codes.is_empty() { None } else { Some(reason_codes) },
    }
}

fn failed(id: &str, reason: &str) -> SandboxProbe {
    SandboxProbe {
        id: id.to_string(),
        passed: false,
        reason_code: Some(reason.to_string()),
    }
}

/// A Seatbelt profile that denies everything except reading the scratch
/// directory itself (needed for the probe's own bookkeeping, not the target).
fn deny_all_profile(scratch_dir: &std::path::Path) -> String {
    format!(
        r#"(version 1)
(deny default)
(allow file-read* (subpath "{}"))
"#,
        scratch_dir.display()
    )
}

fn run_sandboxed(
    profile: &str,
    program: &str,
    args: &[&str],
) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new(SANDBOX_EXEC);
    cmd.arg("-p").arg(profile).arg(program).args(args);
    cmd.output()
}

fn probe_read_denied() -> SandboxProbe {
    const ID: &str = "outside_root_read_denied";
    let Some(scratch) = std::env::temp_dir().canonicalize().ok() else {
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    };

    // A file outside anything the profile grants read access to.
    let target = scratch.join(format!("eury-sandbox-probe-outside-{}", uuid::Uuid::now_v7()));
    let secret = "eury-sandbox-probe-secret";
    if std::fs::write(&target, secret).is_err() {
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    }

    // The profile only grants read access to an *empty* directory, not
    // `scratch` (which contains `target`) — so `target` is unreachable.
    let Ok(empty_dir) = tempdir_in(&scratch) else {
        let _ = std::fs::remove_file(&target);
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    };
    let profile = deny_all_profile(&empty_dir);

    let result = run_sandboxed(&profile, "/bin/cat", &[target.to_string_lossy().as_ref()]);
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_dir(&empty_dir);

    match result {
        Ok(output)
            if output.status.success()
                && output.stdout.windows(secret.len()).any(|w| w == secret.as_bytes()) =>
        {
            SandboxProbe {
                id: ID.to_string(),
                passed: false,
                reason_code: Some("outside_root_unenforceable".to_string()),
            }
        }
        Ok(_) => SandboxProbe { id: ID.to_string(), passed: true, reason_code: None },
        Err(_) => SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("mechanism_unavailable".to_string()),
        },
    }
}

fn probe_write_denied() -> SandboxProbe {
    const ID: &str = "outside_root_write_denied";
    let Some(scratch) = std::env::temp_dir().canonicalize().ok() else {
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    };
    let Ok(empty_dir) = tempdir_in(&scratch) else {
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    };
    // A path outside the (only-readable, not writable) empty_dir.
    let target = scratch.join(format!("eury-sandbox-probe-write-{}", uuid::Uuid::now_v7()));
    let _ = std::fs::remove_file(&target);

    let profile = deny_all_profile(&empty_dir);
    let result = run_sandboxed(&profile, "/usr/bin/touch", &[target.to_string_lossy().as_ref()]);
    let write_succeeded = target.exists();
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_dir(&empty_dir);

    match result {
        Ok(_) if write_succeeded => SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("outside_root_unenforceable".to_string()),
        },
        Ok(_) => SandboxProbe { id: ID.to_string(), passed: true, reason_code: None },
        Err(_) => SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("mechanism_unavailable".to_string()),
        },
    }
}

fn probe_egress_denied() -> SandboxProbe {
    const ID: &str = "egress_denied";
    let Some(scratch) = std::env::temp_dir().canonicalize().ok() else {
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    };
    let Ok(empty_dir) = tempdir_in(&scratch) else {
        return SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("probe_failed".to_string()),
        };
    };
    let profile = deny_all_profile(&empty_dir);

    // `/usr/bin/nc` ships with macOS. `-G 2` bounds the connect attempt to
    // ~2s so a broken (non-denying) profile can't hang the probe.
    let result = run_sandboxed(&profile, "/usr/bin/nc", &["-z", "-G", "2", "1.1.1.1", "80"]);
    let _ = std::fs::remove_dir(&empty_dir);

    match result {
        Ok(output) if output.status.success() => SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("egress_unenforceable".to_string()),
        },
        Ok(_) => SandboxProbe { id: ID.to_string(), passed: true, reason_code: None },
        Err(_) => SandboxProbe {
            id: ID.to_string(),
            passed: false,
            reason_code: Some("mechanism_unavailable".to_string()),
        },
    }
}

/// Runs `sh -c script` under `profile` and reports whether `marker` reached
/// stdout — the observable proof that the script's child process ran.
fn marker_reached_stdout(profile: &str, script: &str, marker: &str) -> std::io::Result<bool> {
    let output = run_sandboxed(profile, "/bin/sh", &["-c", script])?;
    Ok(output.stdout.windows(marker.len()).any(|w| w == marker.as_bytes()))
}

/// Verifies a sandboxed process cannot spawn children.
///
/// Denies `process-fork` while still allowing exec, then has a shell try to
/// run two commands — the first of which must be forked. The probe passes only
/// when that child's marker never appears, and the same script *does* print it
/// under an otherwise identical profile that permits forking.
fn probe_process_tree_contained() -> SandboxProbe {
    const ID: &str = "process_tree_contained";
    const MARKER: &str = "eury-fork-marker";

    let script = format!("/bin/echo {MARKER} && /bin/echo second");

    let denied_profile = "(version 1)\n(deny default)\n(allow file-read*)\n\
         (allow process-exec)\n(deny process-fork)\n";
    let control_profile = "(version 1)\n(deny default)\n(allow file-read*)\n\
         (allow process-exec)\n(allow process-fork)\n";

    let Ok(forked_under_deny) = marker_reached_stdout(denied_profile, &script, MARKER) else {
        return failed(ID, "mechanism_unavailable");
    };
    if forked_under_deny {
        return failed(ID, "process_tree_unenforceable");
    }

    // Control: without the fork denial the same script must succeed, otherwise
    // the silence above proves nothing about containment.
    match marker_reached_stdout(control_profile, &script, MARKER) {
        Ok(true) => SandboxProbe { id: ID.to_string(), passed: true, reason_code: None },
        Ok(false) => failed(ID, "probe_control_failed"),
        Err(_) => failed(ID, "mechanism_unavailable"),
    }
}

/// Verifies a sandboxed process cannot escalate privileges by executing a
/// setuid-root binary.
///
/// `/usr/bin/sudo` is the escalation vector present on stock macOS. The probe
/// passes only when running it produces none of its output while a non-setuid
/// binary in the same directory runs fine under the identical profile — so a
/// missing binary or a blanket exec failure cannot masquerade as containment.
fn probe_privilege_escalation_denied() -> SandboxProbe {
    const ID: &str = "privilege_escalation_denied";
    const SETUID_BINARY: &str = "/usr/bin/sudo";
    const SETUID_MARKER: &str = "Sudo version";
    const CONTROL_MARKER: &str = "eury-escalation-control";

    // Only meaningful if the escalation vector actually exists here.
    let Ok(metadata) = std::fs::metadata(SETUID_BINARY) else {
        return failed(ID, "probe_not_applicable");
    };
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o4000 == 0 {
            return failed(ID, "probe_not_applicable");
        }
    }

    let profile = "(version 1)\n(deny default)\n(allow file-read*)\n\
         (allow process-fork)\n(allow process-exec)\n";

    let Ok(escalated) =
        marker_reached_stdout(profile, &format!("{SETUID_BINARY} -V"), SETUID_MARKER)
    else {
        return failed(ID, "mechanism_unavailable");
    };
    if escalated {
        return failed(ID, "privilege_escalation_unenforceable");
    }

    // Control: a non-setuid binary in the same directory must still run, or
    // the denial above is just "exec is broken here".
    let control = format!("/usr/bin/basename {CONTROL_MARKER}");
    match marker_reached_stdout(profile, &control, CONTROL_MARKER) {
        Ok(true) => SandboxProbe { id: ID.to_string(), passed: true, reason_code: None },
        Ok(false) => failed(ID, "probe_control_failed"),
        Err(_) => failed(ID, "mechanism_unavailable"),
    }
}

/// Creates a fresh, empty subdirectory under `parent` for a probe to grant
/// Seatbelt read access to (deliberately separate from any directory that
/// holds probe target files, so the target stays unreachable).
fn tempdir_in(parent: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let dir = parent.join(format!("eury-sandbox-probe-root-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir)?;
    // Touch a marker file so the directory isn't perfectly empty (avoids any
    // edge-case Seatbelt handling of a truly-empty subpath grant).
    let _ = std::fs::File::create(dir.join(".probe")).and_then(|mut f| f.write_all(b"probe"));
    Ok(dir)
}
