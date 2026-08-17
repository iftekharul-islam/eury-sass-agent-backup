use agent_sandbox::command::{CommandGuard, CommandGuardError};

fn assert_forbidden(cmd: &str) {
    assert!(
        matches!(CommandGuard::parse_and_verify(cmd), Err(CommandGuardError::ForbiddenCommand(_))),
        "expected ForbiddenCommand for {cmd:?}"
    );
}

fn parse_ok(cmd: &str) -> agent_sandbox::command::CommandShape {
    match CommandGuard::parse_and_verify(cmd) {
        Ok(shape) => shape,
        Err(err) => panic!("expected Ok for {cmd:?}: {err:?}"),
    }
}

fn assert_allowed(cmd: &str) {
    let _ = parse_ok(cmd);
}

#[test]
fn find_with_negation_is_allowed() {
    assert_allowed("find . -mindepth 1 -maxdepth 1 ! -name AGENTS.md ! -name node_modules");
}

#[test]
fn rm_rf_workspace_dir_is_allowed() {
    assert_allowed("rm -rf cit230");
    assert_allowed("rm -rf node_modules");
}

#[test]
fn blocks_direct_forbidden_command() {
    assert_forbidden("sudo rm -rf /");
    assert_forbidden("rm -rf /");
    assert_forbidden("su root");
    assert_forbidden("mkfs.ext4 /dev/sda1");
    assert_forbidden("dd if=/dev/zero of=/dev/sda");
}

#[test]
fn blocks_forbidden_command_wrapped_in_bash_dash_c() {
    assert_forbidden(r#"bash -c "sudo rm -rf /""#);
    assert_forbidden(r#"sh -c "rm -rf /""#);
    assert_forbidden(r#"zsh -c "su root""#);
}

#[test]
fn blocks_forbidden_command_via_absolute_shell_path() {
    assert_forbidden(r#"/bin/bash -c "rm -rf /""#);
    assert_forbidden(r#"/usr/bin/sh -c "sudo rm -rf /""#);
}

#[test]
fn blocks_forbidden_command_chained_with_operators() {
    assert_forbidden(r#"bash -c "echo hi && rm -rf /""#);
    assert_forbidden(r#"bash -c "echo hi; sudo su""#);
    assert_forbidden(r#"bash -c "echo hi || rm -rf /""#);
    assert_forbidden(r#"bash -c "cat foo | rm -rf /""#);
    assert_forbidden(r#"bash -c "rm -rf / &""#);
}

#[test]
fn blocks_forbidden_command_in_backtick_substitution() {
    assert_forbidden(r#"bash -c "echo `rm -rf /`""#);
}

#[test]
fn blocks_forbidden_command_in_dollar_paren_substitution() {
    assert_forbidden(r#"bash -c "echo $(sudo rm -rf /)""#);
}

#[test]
fn blocks_forbidden_command_via_env_indirection() {
    assert_forbidden("env bash -c \"sudo rm -rf /\"");
    assert_forbidden("env FOO=bar rm -rf /");
}

#[test]
fn blocks_forbidden_command_nested_two_shells_deep() {
    assert_forbidden(r#"bash -c "bash -c \"sudo rm -rf /\"""#);
}

#[test]
fn allows_plain_non_shell_commands() {
    assert_allowed("ls -la");
    assert_allowed("grep foo bar.txt");
    assert_allowed("echo hello world");
}

#[test]
fn allows_legitimate_shell_eval_pipelines() {
    assert_allowed(r#"bash -c "cat foo.txt | grep bar""#);
    assert_allowed(r#"bash -c "echo hello world""#);
    assert_allowed(r#"sh -c "ls -la && echo done""#);
}

#[test]
fn marks_shell_invocations_as_shell_eval() {
    assert!(matches!(
        CommandGuard::parse_and_verify(r#"bash -c "echo hi""#),
        Ok(shape) if shape.is_shell_eval
    ));
    assert!(matches!(
        CommandGuard::parse_and_verify("ls -la"),
        Ok(shape) if !shape.is_shell_eval
    ));
}

#[test]
fn wraps_chained_commands_in_shell_eval() {
    let cmd = "pwd && find . -maxdepth 2 -type f -print | sort | head -n 20";
    let shape = parse_ok(cmd);
    assert!(shape.is_shell_eval, "&& and | must run under sh -c");
    assert_eq!(shape.executable, "sh");
    assert_eq!(shape.args, vec!["-c", cmd]);
}

#[test]
fn allows_simple_pwd_without_shell_wrapper() {
    let shape = parse_ok("pwd");
    assert!(!shape.is_shell_eval);
    assert_eq!(shape.executable, "pwd");
}

#[test]
fn rejects_empty_command() {
    assert!(matches!(CommandGuard::parse_and_verify(""), Err(CommandGuardError::ParseError(_))));
    assert!(matches!(CommandGuard::parse_and_verify("   "), Err(CommandGuardError::ParseError(_))));
}

#[test]
fn rejects_unclosed_quotes() {
    assert!(matches!(
        CommandGuard::parse_and_verify("echo 'unclosed"),
        Err(CommandGuardError::ParseError(_))
    ));
}
