//! Validates the security property of `pty::sanitize`: secret-shaped
//! environment variables never survive into a PTY shell's environment,
//! even when present in the source environment (as `agent/.env`, loaded via
//! `dotenvy` at desktop app startup, puts them into the real process env).

use agent_sandbox::pty::sanitize;
use std::path::PathBuf;

fn source_env() -> Vec<(String, String)> {
    vec![
        ("HOME".into(), "/home/tester".into()),
        ("PATH".into(), "/usr/bin:/bin".into()),
        ("SHELL".into(), "/bin/bash".into()),
        // Planted secrets, shaped like what `agent/.env` or the OS keychain
        // bridge could put in the process environment.
        ("EURY_AGENT_GATEWAY_TOKEN".into(), "super-secret-token".into()),
        ("ANTHROPIC_API_KEY".into(), "sk-ant-fake-key".into()),
        ("AWS_SECRET_ACCESS_KEY".into(), "aws-fake-secret".into()),
        ("GITHUB_TOKEN".into(), "ghp_fake".into()),
        ("SESSION_COOKIE".into(), "fake-session".into()),
        ("SOME_RANDOM_PASSWORD".into(), "hunter2".into()),
    ]
}

#[test]
fn strips_all_secret_shaped_variables() -> Result<(), Box<dyn std::error::Error>> {
    let env = sanitize(&PathBuf::from("/workspace"), "1.0.0", source_env());

    for secret_name in [
        "EURY_AGENT_GATEWAY_TOKEN",
        "ANTHROPIC_API_KEY",
        "AWS_SECRET_ACCESS_KEY",
        "GITHUB_TOKEN",
        "SESSION_COOKIE",
        "SOME_RANDOM_PASSWORD",
    ] {
        if env.get(secret_name).is_some() {
            return Err(format!("{secret_name} leaked into the sanitized PTY environment").into());
        }
    }
    Ok(())
}

#[test]
fn preserves_allowlisted_variables() {
    let env = sanitize(&PathBuf::from("/workspace"), "1.0.0", source_env());

    #[cfg(unix)]
    {
        assert_eq!(env.get("HOME"), Some("/home/tester"));
        assert_eq!(env.get("SHELL"), Some("/bin/bash"));
    }
    assert_eq!(env.get("PATH"), Some("/usr/bin:/bin"));
}

#[test]
fn sets_terminal_identity_variables() -> Result<(), Box<dyn std::error::Error>> {
    let env = sanitize(&PathBuf::from("/some/workspace"), "9.9.9", source_env());

    assert_eq!(env.get("TERM"), Some("xterm-256color"));
    assert_eq!(env.get("TERM_PROGRAM"), Some("Eury"));
    assert_eq!(env.get("TERM_PROGRAM_VERSION"), Some("9.9.9"));
    let Some(pwd) = env.get("PWD") else {
        return Err("expected PWD to be set".into());
    };
    assert!(pwd.ends_with("some/workspace") || pwd.ends_with("some\\workspace"));
    Ok(())
}

#[test]
fn ignores_variables_outside_the_allowlist() {
    let mut source = source_env();
    source.push(("SOME_UNRELATED_APP_SETTING".into(), "value".into()));
    let env = sanitize(&PathBuf::from("/workspace"), "1.0.0", source);

    assert_eq!(env.get("SOME_UNRELATED_APP_SETTING"), None);
}
