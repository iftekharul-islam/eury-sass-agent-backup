use std::collections::BTreeMap;
use std::path::Path;

/// An environment safe to hand to a PTY-spawned shell: built allowlist-first
/// from the current process environment, then filtered a second time by a
/// denylist as defense in depth. Never built by inheriting the parent
/// environment wholesale — the desktop process loads `agent/.env` via
/// `dotenvy` at startup, so a naive inherit-then-strip approach would leak
/// whatever lands there (gateway URLs today, credentials tomorrow) into
/// every shell the user — and anything running inside it — can read.
#[derive(Debug, Clone, Default)]
pub struct SanitizedEnv(BTreeMap<String, String>);

impl SanitizedEnv {
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn set(&mut self, key: &str, value: impl Into<String>) {
        self.0.insert(key.to_string(), value.into());
    }
}

#[cfg(unix)]
const ALLOWLIST: &[&str] = &[
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "PATH",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "TZ",
    "TMPDIR",
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_RUNTIME_DIR",
];

#[cfg(windows)]
const ALLOWLIST: &[&str] = &[
    "SYSTEMROOT",
    "WINDIR",
    "PATH",
    "PATHEXT",
    "USERPROFILE",
    "USERNAME",
    "APPDATA",
    "LOCALAPPDATA",
    "TEMP",
    "TMP",
    "COMSPEC",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
];

/// Names (case-insensitive substring or prefix match) that must never reach
/// a PTY shell even if they somehow made it onto the allowlist above. This
/// is deliberately redundant with the allowlist: it is the guard against a
/// future contributor widening `ALLOWLIST` carelessly.
const DENY_SUBSTRINGS: &[&str] =
    &["TOKEN", "SECRET", "KEY", "PASSWORD", "PASSWD", "CREDENTIAL", "SESSION", "COOKIE", "AUTH"];
const DENY_PREFIXES: &[&str] =
    &["EURY_", "ANTHROPIC_", "OPENAI_", "AWS_", "AZURE_", "GCP_", "GOOGLE_"];

fn is_denied(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    DENY_PREFIXES.iter().any(|p| upper.starts_with(p))
        || DENY_SUBSTRINGS.iter().any(|s| upper.contains(s))
}

/// Builds the environment for a new PTY shell from `source` (the raw
/// candidate environment — real callers pass `std::env::vars()`).
///
/// Takes `source` as a parameter rather than reading `std::env::vars()`
/// internally so this stays a pure, directly testable function: the
/// allowlist/denylist security property is exactly what
/// `tests/pty_env_test.rs` verifies, by handing in a synthetic environment
/// containing planted secrets rather than mutating the real process
/// environment (which recent Rust makes `unsafe`, forbidden in this crate).
///
/// `workspace_root` becomes `PWD`; `app_version` is surfaced as
/// `TERM_PROGRAM_VERSION` so shells and prompts can identify the host.
#[must_use]
pub fn sanitize(
    workspace_root: &Path,
    app_version: &str,
    source: impl IntoIterator<Item = (String, String)>,
) -> SanitizedEnv {
    let mut env = SanitizedEnv::default();
    let source: BTreeMap<String, String> = source.into_iter().collect();

    for name in ALLOWLIST {
        if is_denied(name) {
            continue;
        }
        if let Some(value) = source.get(*name) {
            env.set(name, value.clone());
        }
    }

    env.set("TERM", "xterm-256color");
    env.set("COLORTERM", "truecolor");
    env.set("TERM_PROGRAM", "Eury");
    env.set("TERM_PROGRAM_VERSION", app_version);
    env.set("PWD", workspace_root.to_string_lossy().into_owned());

    env
}
