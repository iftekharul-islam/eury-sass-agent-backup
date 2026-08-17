use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandGuardError {
    #[error("Command is forbidden: {0}")]
    ForbiddenCommand(String),
    #[error("Command parsing failed: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub struct CommandShape {
    pub executable: String,
    pub args: Vec<String>,
    pub requires_egress: bool,
    pub is_shell_eval: bool,
}

/// Shell executables (matched by basename) that evaluate a `-c` payload as a
/// full shell script rather than treating their arguments as literal argv.
const SHELL_EXECUTABLES: &[&str] = &["sh", "bash", "zsh", "dash", "ksh"];

/// Maximum recursion depth when following `-c` payloads, command
/// substitutions (`` `...` `` / `$(...)`), and `env`-style indirection.
/// Bounds work on adversarial input; legitimate commands never nest this deep.
const MAX_NEST_DEPTH: u8 = 8;

pub struct CommandGuard;

impl CommandGuard {
    /// # Errors
    ///
    /// Returns [`CommandGuardError::ParseError`] if `raw_cmd` is empty, has
    /// unclosed quotes, or is empty after shell-token splitting; returns
    /// [`CommandGuardError::ForbiddenCommand`] if the executable — or, for
    /// shell-eval invocations (`sh -c`, `bash -c`, ...), any command reachable
    /// from the `-c` payload via chaining (`;`, `&&`, `||`, `|`, `&`), command
    /// substitution (`` `...` ``, `$(...)`), or `env` indirection — is on the
    /// forbidden list.
    ///
    /// This is a string-level guard, not a sandbox: it cannot see through
    /// variable indirection (`x=rm; $x -rf /`), encoded payloads, or other
    /// forbidden binaries reached via less common wrappers than `env`. It
    /// exists to block the common, direct forms of the forbidden commands;
    /// real containment against a determined adversary requires OS-level
    /// sandboxing (tracked separately).
    pub fn parse_and_verify(raw_cmd: &str) -> Result<CommandShape, CommandGuardError> {
        if raw_cmd.trim().is_empty() {
            return Err(CommandGuardError::ParseError("Empty command".into()));
        }

        let tokens: Vec<String> = shlex_split(raw_cmd)
            .ok_or_else(|| CommandGuardError::ParseError("Unclosed quotes".into()))?;
        if tokens.is_empty() {
            return Err(CommandGuardError::ParseError("Empty command after shlex".into()));
        }

        let basename = Self::basename(&tokens[0]);
        let is_explicit_shell = SHELL_EXECUTABLES.contains(&basename.as_str());

        if is_explicit_shell {
            Self::verify_invocation(&tokens, 0)?;
            let args = tokens[1..].to_vec();
            let requires_egress = Self::requires_egress(&basename, raw_cmd);
            return Ok(CommandShape {
                executable: tokens[0].clone(),
                args,
                requires_egress,
                is_shell_eval: true,
            });
        }

        if needs_shell_wrapper(raw_cmd, &tokens) {
            Self::verify_shell_payload(raw_cmd, 0)?;
            return Ok(CommandShape {
                executable: "sh".to_string(),
                args: vec!["-c".into(), raw_cmd.to_string()],
                requires_egress: Self::requires_egress("", raw_cmd),
                is_shell_eval: true,
            });
        }

        Self::verify_invocation(&tokens, 0)?;
        let args = tokens[1..].to_vec();
        let requires_egress = Self::requires_egress(&basename, raw_cmd);

        Ok(CommandShape {
            executable: tokens[0].clone(),
            args,
            requires_egress,
            is_shell_eval: false,
        })
    }

    fn requires_egress(basename: &str, raw_cmd: &str) -> bool {
        if matches!(basename, "curl" | "wget" | "npm" | "cargo") {
            return true;
        }
        let lower = raw_cmd.to_ascii_lowercase();
        ["curl ", "wget ", "npm ", "pnpm ", "yarn ", "cargo "]
            .iter()
            .any(|needle| lower.contains(needle))
    }

    /// Checks whether `tokens` (an executable followed by its arguments)
    /// itself, or anything it would transitively execute, is forbidden.
    fn verify_invocation(tokens: &[String], depth: u8) -> Result<(), CommandGuardError> {
        if depth > MAX_NEST_DEPTH {
            return Err(CommandGuardError::ParseError(
                "Command nesting exceeds the maximum allowed depth".into(),
            ));
        }
        let Some(raw_executable) = tokens.first() else { return Ok(()) };
        let basename = Self::basename(raw_executable);

        // `mkfs` is commonly invoked through per-filesystem aliases
        // (`mkfs.ext4`, `mkfs.vfat`, ...) that are the same multicall binary.
        if Self::forbidden_commands().contains(basename.as_str()) || basename.starts_with("mkfs.") {
            return Err(CommandGuardError::ForbiddenCommand(raw_executable.clone()));
        }

        if basename == "rm" && crate::rm_safety::is_forbidden_rm_argv(tokens) {
            return Err(CommandGuardError::ForbiddenCommand(raw_executable.clone()));
        }

        // `env [-iu] [NAME=VALUE ...] cmd [args...]` runs `cmd` — peel the
        // wrapper off and verify what it actually launches.
        if basename == "env" {
            if let Some(inner) = Self::strip_env_wrapper(&tokens[1..]) {
                return Self::verify_invocation(&inner, depth + 1);
            }
            return Ok(());
        }

        if SHELL_EXECUTABLES.contains(&basename.as_str())
            && let Some(payload) = Self::extract_dash_c_payload(&tokens[1..])
        {
            Self::verify_shell_payload(&payload, depth + 1)?;
        }

        Ok(())
    }

    /// Verifies every command reachable from a shell `-c` payload: each
    /// `;`/`&&`/`||`/`|`/`&`/newline-separated segment, and the contents of
    /// any `` `...` `` or `$(...)` command substitution, recursively.
    fn verify_shell_payload(payload: &str, depth: u8) -> Result<(), CommandGuardError> {
        if depth > MAX_NEST_DEPTH {
            return Err(CommandGuardError::ParseError(
                "Shell nesting exceeds the maximum allowed depth".into(),
            ));
        }

        for segment in split_shell_commands(payload) {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }
            let tokens = shlex_split(segment).ok_or_else(|| {
                CommandGuardError::ParseError("Unclosed quotes in shell payload".into())
            })?;
            if tokens.is_empty() {
                continue;
            }
            Self::verify_invocation(&tokens, depth)?;
        }

        for substitution in extract_command_substitutions(payload) {
            Self::verify_shell_payload(&substitution, depth + 1)?;
        }

        Ok(())
    }

    /// Strips leading `env` flags (`-i`, `-u NAME`, `--ignore-environment`,
    /// `--unset=NAME`, ...) and `NAME=VALUE` assignments to find the command
    /// `env` would actually launch. Returns `None` if `env` was given no
    /// command to run (e.g. `env` alone, or only flags/assignments).
    fn strip_env_wrapper(args: &[String]) -> Option<Vec<String>> {
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg == "-i" || arg == "--ignore-environment" {
                i += 1;
            } else if arg == "-u" || arg == "--unset" {
                i += 2;
            } else if arg.starts_with('-') || (arg.contains('=') && !arg.starts_with('/')) {
                i += 1;
            } else {
                break;
            }
        }
        if i >= args.len() { None } else { Some(args[i..].to_vec()) }
    }

    fn basename(executable: &str) -> String {
        executable.rsplit('/').next().unwrap_or(executable).to_string()
    }

    fn extract_dash_c_payload(args: &[String]) -> Option<String> {
        let idx = args.iter().position(|a| a == "-c")?;
        args.get(idx + 1).cloned()
    }

    fn forbidden_commands() -> HashSet<&'static str> {
        let mut set = HashSet::new();
        set.insert("sudo");
        set.insert("su");
        set.insert("mkfs");
        set.insert("dd");
        set
    }
}

/// Splits a shell payload into command segments on `;`, `&&`, `||`, `|`, `&`,
/// and newlines, honoring quotes so separators inside `'...'`/`"..."` don't
/// split the segment.
fn split_shell_commands(payload: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut chars = payload.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            current.push(c);
            if c == quote_char {
                in_quotes = false;
            }
            continue;
        }

        match c {
            '\'' | '"' => {
                in_quotes = true;
                quote_char = c;
                current.push(c);
            }
            ';' | '\n' => segments.push(std::mem::take(&mut current)),
            '&' | '|' => {
                if chars.peek() == Some(&c) {
                    chars.next();
                }
                segments.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    segments.push(current);
    segments
}

/// Finds the contents of every `` `...` `` and `$(...)` command substitution
/// in `payload`, honoring nested parentheses inside `$(...)`.
fn extract_command_substitutions(payload: &str) -> Vec<String> {
    let mut results = Vec::new();
    let chars: Vec<char> = payload.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut j = i + 1;
            let mut inner = String::new();
            while j < chars.len() && chars[j] != '`' {
                inner.push(chars[j]);
                j += 1;
            }
            if j < chars.len() {
                results.push(inner);
                i = j + 1;
                continue;
            }
            break;
        }
        if chars[i] == '$' && chars.get(i + 1) == Some(&'(') {
            let mut depth = 1;
            let mut j = i + 2;
            let mut inner = String::new();
            while j < chars.len() && depth > 0 {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                inner.push(chars[j]);
                j += 1;
            }
            if depth == 0 {
                results.push(inner);
                i = j + 1;
                continue;
            }
            break;
        }
        i += 1;
    }
    results
}

/// True when the command must run under `sh -c` because it uses shell syntax
/// (`&&`, pipes, redirects, command substitution) that cannot be expressed
/// as a single executable plus argv.
fn needs_shell_wrapper(raw_cmd: &str, tokens: &[String]) -> bool {
    const OP_TOKENS: &[&str] = &["&&", "||", "|", ";", "&", "|&", ">", ">>", "<", "<<"];
    if tokens.len() > 1 && tokens[1..].iter().any(|t| OP_TOKENS.contains(&t.as_str())) {
        return true;
    }
    shell_syntax_outside_quotes(raw_cmd)
}

/// Detects shell metacharacters in the raw string that shlex does not surface
/// as standalone tokens (for example `$(...)` or a trailing `&`).
fn shell_syntax_outside_quotes(raw_cmd: &str) -> bool {
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let chars: Vec<char> = raw_cmd.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_quotes {
            if c == quote_char {
                in_quotes = false;
            }
            i += 1;
            continue;
        }
        match c {
            '\'' | '"' => {
                in_quotes = true;
                quote_char = c;
            }
            '$' if chars.get(i + 1) == Some(&'(') => return true,
            '`' => return true,
            '&' | '|' | ';' => return true,
            '>' | '<' => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

// Simple shlex split implementation for phase 5
fn shlex_split(cmd: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut escape = false;

    for c in cmd.chars() {
        if escape {
            current.push(c);
            escape = false;
            continue;
        }

        if c == '\\' {
            escape = true;
            continue;
        }

        if in_quotes {
            if c == quote_char {
                in_quotes = false;
            } else {
                current.push(c);
            }
        } else if c == '\'' || c == '"' {
            in_quotes = true;
            quote_char = c;
        } else if c.is_whitespace() {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    if in_quotes || escape {
        return None;
    }

    if !current.is_empty() {
        args.push(current);
    }

    Some(args)
}
