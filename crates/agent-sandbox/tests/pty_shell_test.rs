use agent_sandbox::pty::resolve_shell;

#[test]
fn explicit_nonexistent_shell_errors() {
    let result = resolve_shell(Some("/definitely/not/a/real/shell/binary"));
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn explicit_valid_shell_is_honored() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(spec) = resolve_shell(Some("/bin/sh")) else {
        return Err("expected /bin/sh to resolve".into());
    };
    assert_eq!(spec.program, std::path::PathBuf::from("/bin/sh"));
    assert!(spec.degraded.is_none());
    Ok(())
}

#[test]
fn default_resolution_finds_a_usable_shell() {
    // A stale $SHELL must not be treated as fatal — resolve_shell falls
    // back rather than erroring when a fallback exists.
    let result = resolve_shell(None);
    assert!(result.is_ok(), "expected a default shell to resolve on this platform");
}
