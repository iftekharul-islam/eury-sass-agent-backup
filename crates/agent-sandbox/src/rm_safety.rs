//! Blocks catastrophic `rm` invocations while allowing workspace cleanup
//! (`rm -rf cit230`, `rm -rf node_modules`, etc.).

fn basename(executable: &str) -> String {
    executable.rsplit('/').next().unwrap_or(executable).to_string()
}

fn contains_rm_word(command: &str) -> bool {
    command
        .split_whitespace()
        .any(|token| basename(token) == "rm")
}

/// True when `rm` is invoked with both recursive and force flags (any order).
pub fn rm_has_recursive_and_force(command: &str) -> bool {
    if !contains_rm_word(command) {
        return false;
    }

    let is_recursive_token = |t: &str| {
        (t.starts_with('-') && !t.starts_with("--") && (t.contains('r') || t.contains('R')))
            || t == "--recursive"
    };
    let is_force_token =
        |t: &str| (t.starts_with('-') && !t.starts_with("--") && t.contains('f')) || t == "--force";

    let tokens: Vec<&str> = command.split_whitespace().collect();
    tokens.iter().any(|t| is_recursive_token(t)) && tokens.iter().any(|t| is_force_token(t))
}

fn has_recursive_and_force_flags(args: &[String]) -> bool {
    let is_recursive_token = |t: &str| {
        (t.starts_with('-') && !t.starts_with("--") && (t.contains('r') || t.contains('R')))
            || t == "--recursive"
    };
    let is_force_token =
        |t: &str| (t.starts_with('-') && !t.starts_with("--") && t.contains('f')) || t == "--force";

    args.iter().any(|t| is_recursive_token(t)) && args.iter().any(|t| is_force_token(t))
}

fn rm_path_arguments(tokens: &[String]) -> Vec<String> {
    let mut paths = Vec::new();
    let mut i = 1;
    while i < tokens.len() {
        let arg = &tokens[i];
        if arg == "--" {
            i += 1;
            continue;
        }
        if arg.starts_with('-') {
            i += 1;
            continue;
        }
        paths.push(arg.clone());
        i += 1;
    }
    paths
}

/// Paths that must never be passed to `rm`, regardless of flags.
pub fn is_dangerous_rm_target(target: &str) -> bool {
    let t = target.trim();
    if t.is_empty() {
        return true;
    }

    let normalized = t.replace('\\', "/");
    if normalized == "/" || normalized == "/*" || normalized == "/**" {
        return true;
    }
    if normalized.starts_with("/*/") {
        return true;
    }
    if normalized.split('/').any(|seg| seg == "..") {
        return true;
    }
    if t.starts_with('~') {
        return true;
    }

    let lower = normalized.to_ascii_lowercase();
    for prefix in ["/dev", "/proc", "/sys", "/boot", "/run"] {
        if lower == prefix || lower.starts_with(&format!("{prefix}/")) {
            return true;
        }
    }

    false
}

/// Whether a parsed `rm` argv list must be blocked outright.
pub fn is_forbidden_rm_argv(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    if basename(&tokens[0]) != "rm" {
        return false;
    }

    let paths = rm_path_arguments(tokens);
    if paths.is_empty() {
        return has_recursive_and_force_flags(&tokens[1..]);
    }

    paths.iter().any(|p| is_dangerous_rm_target(p))
}

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

fn split_shell_segments(payload: &str) -> Vec<String> {
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

/// String-level guard used by policy and [`crate::command::CommandGuard`].
pub fn is_forbidden_rm_command(command: &str) -> bool {
    if let Some(tokens) = shlex_split(command) {
        if basename(&tokens[0]) == "rm" && is_forbidden_rm_argv(&tokens) {
            return true;
        }
    }

    for segment in split_shell_segments(command) {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if let Some(tokens) = shlex_split(segment) {
            if basename(&tokens[0]) == "rm" && is_forbidden_rm_argv(&tokens) {
                return true;
            }
        }
    }

    false
}
