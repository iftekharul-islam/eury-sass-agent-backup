//! Self-restricting Linux sandbox probe helper.
//!
//! Each subcommand applies Landlock (and, where needed, seccomp / no-new-privs)
//! to the current process, then attempts a forbidden operation. Exit codes mirror
//! what `agent-sandbox` expects when interpreting probe results:
//! - `0` — forbidden operation was denied (probe passed)
//! - `1` — forbidden operation succeeded (probe failed)
//! - `2` — mechanism unavailable
//! - `3` — control check failed

#![allow(unsafe_code)]

use landlock::{
    path_beneath_rules, AccessFs, AccessNet, CompatLevel, Compatible, LandlockStatus,
    Ruleset, RulesetAttr, RulesetCreatedAttr, ABI,
};
use seccompiler::{apply_filter, BpfProgram, SeccompAction, SeccompFilter, TargetArch};
use std::collections::BTreeMap;
use std::convert::TryInto as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const MARKER_FORK: &str = "eury-fork-marker";
const MARKER_ESCALATION: &str = "eury-escalation-control";

fn main() -> ExitCode {
    let Some(subcommand) = std::env::args().nth(1) else {
        eprintln!("usage: agent-sandbox-probe <subcommand> [args...]");
        return ExitCode::from(2);
    };

    match subcommand.as_str() {
        "read-denied" => read_denied(),
        "write-denied" => write_denied(),
        "egress-denied" => egress_denied(),
        "process-tree-denied" => process_tree_denied(),
        "process-tree-control" => process_tree_control(),
        "privilege-escalation-denied" => privilege_escalation_denied(),
        other => {
            eprintln!("unknown subcommand: {other}");
            ExitCode::from(2)
        }
    }
}

fn read_denied() -> ExitCode {
    let Some(allowed_dir) = arg_path(2) else {
        return ExitCode::from(2);
    };
    let Some(target) = arg_path(3) else {
        return ExitCode::from(2);
    };
    let Some(secret) = std::env::args().nth(4) else {
        return ExitCode::from(2);
    };

    if apply_read_only_filesystem(&allowed_dir).is_err() {
        return ExitCode::from(2);
    }

    match std::fs::read(&target) {
        Ok(bytes) if bytes.windows(secret.len()).any(|w| w == secret.as_bytes()) => ExitCode::from(1),
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::SUCCESS,
    }
}

fn write_denied() -> ExitCode {
    let Some(allowed_dir) = arg_path(2) else {
        return ExitCode::from(2);
    };
    let Some(target) = arg_path(3) else {
        return ExitCode::from(2);
    };

    if apply_read_only_filesystem(&allowed_dir).is_err() {
        return ExitCode::from(2);
    }

    let _ = fs::remove_file(&target);
    match fs::File::create(&target) {
        Ok(_) => {
            let _ = fs::remove_file(&target);
            ExitCode::from(1)
        }
        Err(_) => ExitCode::SUCCESS,
    }
}

fn egress_denied() -> ExitCode {
    if apply_egress_denied().is_err() {
        return ExitCode::from(2);
    }

    match std::net::TcpStream::connect("1.1.1.1:80") {
        Ok(_) => ExitCode::from(1),
        Err(_) => ExitCode::SUCCESS,
    }
}

fn process_tree_denied() -> ExitCode {
    if apply_fork_denied().is_err() {
        return ExitCode::from(2);
    }

    // SAFETY: fork is the syscall under test; a negative result with EPERM means
    // the seccomp filter denied process creation.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EPERM) {
            return ExitCode::SUCCESS;
        }
        return ExitCode::from(2);
    }
    if pid == 0 {
        std::process::exit(0);
    }
    ExitCode::from(1)
}

fn process_tree_control() -> ExitCode {
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return ExitCode::from(2);
    }
    if pid == 0 {
        print!("{MARKER_FORK}");
        std::process::exit(0);
    }
    let mut status = 0;
    // SAFETY: wait for the child started above.
    if unsafe { libc::waitpid(pid, &mut status, 0) } < 0 {
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

fn privilege_escalation_denied() -> ExitCode {
    const SETUID_BINARY: &str = "/usr/bin/sudo";
    const SETUID_MARKER: &str = "Sudo version";

    let Ok(metadata) = fs::metadata(SETUID_BINARY) else {
        // No setuid binary to test — treat as passed for this machine.
        return ExitCode::SUCCESS;
    };
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o4000 == 0 {
            return ExitCode::SUCCESS;
        }
    }

    if set_no_new_privs().is_err() {
        return ExitCode::from(2);
    }

    let escalated = Command::new(SETUID_BINARY).arg("-V").output();
    let escalation_succeeded = escalated.as_ref().is_ok_and(|out| {
        out.status.success()
            && out.stdout.windows(SETUID_MARKER.len()).any(|w| w == SETUID_MARKER.as_bytes())
    });
    if escalation_succeeded {
        return ExitCode::from(1);
    }

    let control = format!("/usr/bin/basename {MARKER_ESCALATION}");
    match Command::new("/bin/sh").arg("-c").arg(control).output() {
        Ok(out) if out.stdout.windows(MARKER_ESCALATION.len()).any(|w| w == MARKER_ESCALATION.as_bytes()) => {
            ExitCode::SUCCESS
        }
        Ok(_) => ExitCode::from(3),
        Err(_) => ExitCode::from(2),
    }
}

fn arg_path(index: usize) -> Option<PathBuf> {
    std::env::args().nth(index).map(PathBuf::from)
}

fn apply_read_only_filesystem(allowed_dir: &Path) -> Result<(), ProbeError> {
    let abi = ABI::V5;
    let read_access = AccessFs::from_read(abi);
    let write_access = AccessFs::from_write(abi);
    let status = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(read_access | write_access)?
        .create()?
        .add_rules(path_beneath_rules([allowed_dir], read_access | write_access))?
        .restrict_self()?;
    ensure_enforced(status.landlock)?;
    Ok(())
}

fn apply_egress_denied() -> Result<(), ProbeError> {
    let status = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(AccessNet::ConnectTcp)?
        .create()?
        .restrict_self()?;
    ensure_enforced(status.landlock)?;
    Ok(())
}

fn apply_fork_denied() -> Result<(), ProbeError> {
    set_no_new_privs()?;

    let arch: TargetArch = std::env::consts::ARCH
        .try_into()
        .map_err(|_| ProbeError::UnsupportedArch(std::env::consts::ARCH.to_string()))?;

    let mut rules = BTreeMap::new();
    for syscall in [libc::SYS_clone, libc::SYS_fork, libc::SYS_vfork] {
        rules.insert(syscall, Vec::new());
    }

    let bpf: BpfProgram = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        arch,
    )
    .map_err(|_| ProbeError::MechanismUnavailable)?
    .try_into()
    .map_err(|_| ProbeError::MechanismUnavailable)?;

    apply_filter(&bpf).map_err(|_| ProbeError::MechanismUnavailable)?;
    Ok(())
}

fn set_no_new_privs() -> Result<(), ProbeError> {
    // SAFETY: PR_SET_NO_NEW_PRIVS is a well-defined prctl that cannot fail on
    // a modern Linux kernel unless the argument is invalid.
    let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if rc == 0 { Ok(()) } else { Err(ProbeError::MechanismUnavailable) }
}

fn ensure_enforced(status: LandlockStatus) -> Result<(), ProbeError> {
    match status {
        LandlockStatus::Available { .. } => Ok(()),
        LandlockStatus::NotEnabled | LandlockStatus::NotImplemented => {
            Err(ProbeError::MechanismUnavailable)
        }
    }
}

#[derive(Debug)]
enum ProbeError {
    MechanismUnavailable,
    UnsupportedArch(String),
    Landlock(landlock::RulesetError),
}

impl From<landlock::RulesetError> for ProbeError {
    fn from(value: landlock::RulesetError) -> Self {
        Self::Landlock(value)
    }
}
