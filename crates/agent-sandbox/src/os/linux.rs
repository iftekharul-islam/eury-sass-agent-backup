//! Real, runtime-verified Landlock (+ seccomp / no-new-privs) probing for Linux.
//!
//! Each probe spawns the `agent-sandbox-probe` helper, which restricts *itself*
//! before attempting the forbidden operation. A probe passes only when the
//! forbidden operation observably failed — never when we merely assume Landlock
//! is enforcing anything.

use super::{SandboxCapabilities, SandboxProbe};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

const MECHANISM: &str = "landlock_seccomp";
const PROBE_BINARY: &str = "agent-sandbox-probe";

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
        os: "linux".to_string(),
        os_version: linux_os_version(),
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

fn probe_read_denied() -> SandboxProbe {
    const ID: &str = "outside_root_read_denied";
    let Some(probe) = probe_binary() else {
        return failed(ID, "mechanism_unavailable");
    };
    let Some(scratch) = std::env::temp_dir().canonicalize().ok() else {
        return failed(ID, "probe_failed");
    };
    let target = scratch.join(format!("eury-sandbox-probe-outside-{}", uuid::Uuid::now_v7()));
    let secret = "eury-sandbox-probe-secret";
    if std::fs::write(&target, secret).is_err() {
        return failed(ID, "probe_failed");
    }
    let Ok(empty_dir) = tempdir_in(&scratch) else {
        let _ = std::fs::remove_file(&target);
        return failed(ID, "probe_failed");
    };

    let status = Command::new(&probe)
        .arg("read-denied")
        .arg(&empty_dir)
        .arg(&target)
        .arg(secret)
        .status();
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_dir(&empty_dir);

    interpret_probe_status(ID, status, "outside_root_unenforceable")
}

fn probe_write_denied() -> SandboxProbe {
    const ID: &str = "outside_root_write_denied";
    let Some(probe) = probe_binary() else {
        return failed(ID, "mechanism_unavailable");
    };
    let Some(scratch) = std::env::temp_dir().canonicalize().ok() else {
        return failed(ID, "probe_failed");
    };
    let Ok(empty_dir) = tempdir_in(&scratch) else {
        return failed(ID, "probe_failed");
    };
    let target = scratch.join(format!("eury-sandbox-probe-write-{}", uuid::Uuid::now_v7()));
    let _ = std::fs::remove_file(&target);

    let status = Command::new(&probe)
        .arg("write-denied")
        .arg(&empty_dir)
        .arg(&target)
        .status();
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_dir(&empty_dir);

    interpret_probe_status(ID, status, "outside_root_unenforceable")
}

fn probe_egress_denied() -> SandboxProbe {
    const ID: &str = "egress_denied";
    let Some(probe) = probe_binary() else {
        return failed(ID, "mechanism_unavailable");
    };
    interpret_probe_status(ID, Command::new(&probe).arg("egress-denied").status(), "egress_unenforceable")
}

fn probe_process_tree_contained() -> SandboxProbe {
    const ID: &str = "process_tree_contained";
    let Some(probe) = probe_binary() else {
        return failed(ID, "mechanism_unavailable");
    };

    let Ok(denied_status) = Command::new(&probe).arg("process-tree-denied").status() else {
        return failed(ID, "mechanism_unavailable");
    };
    match denied_status.code() {
        Some(0) => {}
        Some(1) => return failed(ID, "process_tree_unenforceable"),
        _ => return failed(ID, "mechanism_unavailable"),
    }

    match Command::new(&probe).arg("process-tree-control").status() {
        Ok(control_status) if control_status.code() == Some(0) => {
            SandboxProbe { id: ID.to_string(), passed: true, reason_code: None }
        }
        Ok(_) => failed(ID, "probe_control_failed"),
        Err(_) => failed(ID, "mechanism_unavailable"),
    }
}

fn probe_privilege_escalation_denied() -> SandboxProbe {
    const ID: &str = "privilege_escalation_denied";
    let Some(probe) = probe_binary() else {
        return failed(ID, "mechanism_unavailable");
    };
    interpret_probe_status(
        ID,
        Command::new(&probe).arg("privilege-escalation-denied").status(),
        "privilege_escalation_unenforceable",
    )
}

fn interpret_probe_status(
    id: &str,
    status: std::io::Result<std::process::ExitStatus>,
    unenforceable_reason: &str,
) -> SandboxProbe {
    match status {
        Ok(exit) => match exit.code() {
            Some(0) => SandboxProbe { id: id.to_string(), passed: true, reason_code: None },
            Some(1) => failed(id, unenforceable_reason),
            Some(3) => failed(id, "probe_control_failed"),
            _ => failed(id, "mechanism_unavailable"),
        },
        Err(_) => failed(id, "mechanism_unavailable"),
    }
}

fn probe_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("AGENT_SANDBOX_PROBE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let sibling = parent.join(PROBE_BINARY);
            if sibling.is_file() {
                return Some(sibling);
            }
        }
    }
    workspace_debug_probe()
}

fn workspace_debug_probe() -> Option<PathBuf> {
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        for subdir in ["debug", "release"] {
            let candidate = PathBuf::from(&target_dir).join(subdir).join(PROBE_BINARY);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_target = manifest_dir.join("../../target");
    for profile in ["debug", "release"] {
        let candidate = workspace_target.join(profile).join(PROBE_BINARY);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn linux_os_version() -> Option<String> {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn tempdir_in(parent: &Path) -> std::io::Result<PathBuf> {
    let dir = parent.join(format!("eury-sandbox-probe-root-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir)?;
    let _ = std::fs::File::create(dir.join(".probe")).and_then(|mut f| f.write_all(b"probe"));
    Ok(dir)
}
