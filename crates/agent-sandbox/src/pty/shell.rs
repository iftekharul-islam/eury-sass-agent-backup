use std::path::{Path, PathBuf};

use super::PtyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradeReason {
    ConPtyUnavailable,
    PowerShellMissing,
    WinPtyFallback,
}

impl DegradeReason {
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::ConPtyUnavailable => {
                "ConPTY is unavailable on this Windows build; resize and full-screen programs may misbehave"
            }
            Self::PowerShellMissing => "PowerShell 7 was not found; falling back to cmd.exe",
            Self::WinPtyFallback => {
                "Using the legacy WinPTY backend; some ANSI sequences may render incorrectly"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub degraded: Option<DegradeReason>,
}

/// Resolves the shell to launch, never following a login-shell path.
///
/// A login shell (`-l`) re-sources profile files (`~/.zprofile`, etc.),
/// which can reintroduce exactly the secrets `env::sanitize` strips. PTY
/// sessions are spawned interactive, non-login.
///
/// # Errors
///
/// Returns [`PtyError::NoShell`] if `explicit` names a non-executable path,
/// or if no usable shell can be found on the platform's fallback list.
pub fn resolve_shell(explicit: Option<&str>) -> Result<ShellSpec, PtyError> {
    if let Some(path) = explicit {
        let candidate = PathBuf::from(path);
        if is_executable_file(&candidate) {
            return Ok(ShellSpec { program: candidate, args: Vec::new(), degraded: None });
        }
        return Err(PtyError::NoShell(format!(
            "requested shell {path} does not exist or is not executable"
        )));
    }

    #[cfg(unix)]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            let candidate = PathBuf::from(&shell);
            if is_executable_file(&candidate) {
                return Ok(ShellSpec { program: candidate, args: Vec::new(), degraded: None });
            }
        }

        for fallback in ["/bin/bash", "/bin/sh"] {
            let candidate = PathBuf::from(fallback);
            if is_executable_file(&candidate) {
                return Ok(ShellSpec { program: candidate, args: Vec::new(), degraded: None });
            }
        }

        Err(PtyError::NoShell(
            "no usable shell found ($SHELL, /bin/bash, /bin/sh all unavailable)".into(),
        ))
    }

    #[cfg(windows)]
    {
        if let Some(pwsh) = find_on_path("pwsh.exe") {
            return Ok(ShellSpec { program: pwsh, args: vec!["-NoLogo".into()], degraded: None });
        }
        if let Some(powershell) = find_on_path("powershell.exe") {
            return Ok(ShellSpec {
                program: powershell,
                args: vec!["-NoLogo".into()],
                degraded: Some(DegradeReason::PowerShellMissing),
            });
        }
        if let Some(cmd) = find_on_path("cmd.exe") {
            return Ok(ShellSpec {
                program: cmd,
                args: Vec::new(),
                degraded: Some(DegradeReason::PowerShellMissing),
            });
        }

        Err(PtyError::NoShell(
            "no usable shell found (pwsh.exe, powershell.exe, cmd.exe all unavailable)".into(),
        ))
    }
}

fn is_executable_file(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        std::fs::metadata(path).is_ok_and(|m| m.is_file())
    }
}

#[cfg(windows)]
fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
