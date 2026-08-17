use crate::audit::{AuditEvent, AuditQueue};
use crate::schema::{Decision, GrantScope, ToolClass, WorkspacePolicy};
use crate::store::{Grant, GrantStore};
use agent_sandbox::command::CommandGuard;
use agent_sandbox::rm_safety::{is_forbidden_rm_command, rm_has_recursive_and_force};
use agent_sandbox::workspace::Workspace;
use agent_types::requests::RunMode;
use globset::Glob;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::{LazyLock, Mutex, PoisonError};

/// Compiles a list of hardcoded patterns, silently dropping any that fail to
/// parse. The patterns below are fixed strings validated by this crate's own
/// tests, so a compile failure here can only mean a typo introduced during
/// development — never bad runtime input. Dropping rather than panicking
/// keeps this crate free of `unwrap`/`expect` (enforced by
/// `scripts/check-boundaries.mjs`) and fails toward "this one pattern
/// doesn't match" instead of taking the process down.
fn compile_patterns(patterns: &[&str]) -> Vec<Regex> {
    patterns.iter().filter_map(|p| Regex::new(p).ok()).collect()
}

/// Commands that are never acceptable regardless of workspace policy — the
/// "Forbidden" row of the threat model's execute-risk table (privilege
/// escalation, destructive root/device commands, fork bombs, pipe-to-shell,
/// and force/reset/clean-style history-destroying git operations).
static FORBIDDEN_COMMAND_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    compile_patterns(&[
        r"\bsudo\b",
        r"\bsu\b",
        r"\bmkfs(\.\w+)?\b",
        r"\bdd\s+if=",
        r"--no-preserve-root\b",
        r":\(\)\s*\{\s*:\s*\|\s*:\s*&?\s*\}\s*;\s*:",
        r"\bgit\s+push\b[^&|;]*--force\b",
        r"\bgit\s+reset\s+--hard\b",
        r"\bgit\s+clean\s+-\w*f",
        r"\b(curl|wget)\b[^|]*\|\s*(sudo\s+)?(sh|bash|zsh)\b",
    ])
});

/// Commands that need the tightest scrutiny short of an outright deny — the
/// "Execute, elevated" row of the threat model's table (dependency install,
/// container/image builds, workspace `rm -rf`, and shell metacharacters).
static ELEVATED_COMMAND_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    compile_patterns(&[
        r"\b(npm|pnpm|yarn)\s+(i|install|add)\b",
        r"\bpip3?\s+install\b",
        r"\b(apt|apt-get|brew|yum|dnf)\s+install\b",
        r"\bdocker\s+build\b",
        r"\bcargo\s+install\b",
        r"[;&|`]|\$\(",
    ])
});

pub struct PolicyEngine {
    pub policy: WorkspacePolicy,
    pub grant_store: Mutex<GrantStore>,
    audit: Mutex<AuditQueue>,
}

/// Result of a grant lookup or write against the store, for callers that
/// need to distinguish "no grant store access" from "no grant exists" (this
/// crate's `GrantStore` uses `rusqlite::Result`, so re-exporting that type
/// keeps `PolicyEngine`'s public surface free of a `rusqlite` dependency leak
/// beyond this one alias).
pub type GrantStoreResult<T> = Result<T, rusqlite::Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum RiskClassification {
    Low,
    Medium,
    Elevated,
    Critical,
    Forbidden,
}

impl PolicyEngine {
    /// Creates a `PolicyEngine` with an in-memory audit log. Use
    /// [`Self::with_audit_queue`] to give it a persistent one instead (e.g.
    /// once callers have a durable store to point `AuditQueue` at).
    ///
    /// # Errors
    ///
    /// Returns an error if the in-memory audit queue cannot be opened.
    pub fn new(policy: WorkspacePolicy, grant_store: GrantStore) -> GrantStoreResult<Self> {
        let audit = AuditQueue::new_in_memory()?;
        Ok(Self::with_audit_queue(policy, grant_store, audit))
    }

    #[must_use]
    pub fn with_audit_queue(
        policy: WorkspacePolicy,
        grant_store: GrantStore,
        audit: AuditQueue,
    ) -> Self {
        Self { policy, grant_store: Mutex::new(grant_store), audit: Mutex::new(audit) }
    }

    /// Runs `f` against the engine's audit log — e.g. to list events or
    /// verify the hash chain. The queue is behind a `Mutex`, so this
    /// borrow-scoped accessor avoids handing out the lock guard itself.
    pub fn with_audit<T>(&self, f: impl FnOnce(&AuditQueue) -> T) -> T {
        let audit = self.audit.lock().unwrap_or_else(PoisonError::into_inner);
        f(&audit)
    }

    /// Appends a `policy_decision` event to the hash-chain audit log. Best
    /// effort: a logging failure must never block the tool-call decision
    /// path it's recording, so failures are swallowed here rather than
    /// propagated.
    fn append_audit_event(
        &self,
        tool_class: ToolClass,
        tool_name: &str,
        risk: &RiskClassification,
        decision: Decision,
        run_id: Option<&str>,
    ) {
        let payload = serde_json::json!({
            "toolClass": format!("{tool_class:?}"),
            "toolName": tool_name,
            "risk": format!("{risk:?}"),
            "decision": format!("{decision:?}"),
        });
        let mut event = AuditEvent::new("policy_decision".to_string(), payload);
        event.severity = match decision {
            Decision::Deny => "warning".to_string(),
            Decision::NeedsApproval | Decision::Allow => "info".to_string(),
        };
        if let Some(run_id) = run_id {
            event.run_id = run_id.to_string();
        }

        let mut audit = self.audit.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = audit.append_event(event);
    }

    /// Computes a stable "shape" hash for a tool call from its name and the
    /// canonical-JSON form of its arguments. Grants are keyed on
    /// `(tool_name, shape_hash)`, so a grant only ever matches calls with the
    /// exact argument shape it was created for — it never widens to "this
    /// tool, any arguments". `serde_json::Value`'s default `Map` is a
    /// `BTreeMap` (this workspace does not enable `preserve_order`), so
    /// `Value::to_string` serializes object keys in a stable sorted order
    /// regardless of insertion order, making this hash deterministic across
    /// calls that are semantically identical but built in different orders.
    #[must_use]
    pub fn shape_hash(tool_name: &str, args: &Value) -> String {
        let mut hasher = Sha256::new();
        hasher.update(tool_name.as_bytes());
        hasher.update(b"\0");
        hasher.update(args.to_string().as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    /// Shape key for a session-scoped execute grant — covers every command in
    /// the tool for the rest of the app session.
    pub const SESSION_EXECUTE_SHAPE: &str = "*";

    /// Shape key stored for a session-scoped write grant — same file path, any
    /// content.
    #[must_use]
    pub fn session_write_shape_hash(tool_name: &str, args: &Value) -> String {
        let path = args.get("path").and_then(Value::as_str).unwrap_or("");
        Self::shape_hash(tool_name, &serde_json::json!({ "path": path }))
    }

    /// Shape key to store when the user chooses "allow for this session".
    #[must_use]
    pub fn session_grant_shape(tool_class: ToolClass, tool_name: &str, args: &Value) -> String {
        match tool_class {
            ToolClass::Execute => Self::SESSION_EXECUTE_SHAPE.to_string(),
            ToolClass::Write | ToolClass::WriteOutsideWorkspace => {
                Self::session_write_shape_hash(tool_name, args)
            }
            _ => Self::shape_hash(tool_name, args),
        }
    }

    fn has_standing_grant(
        &self,
        tool_class: ToolClass,
        tool_name: &str,
        args: &Value,
        current_run_id: Option<&str>,
    ) -> bool {
        let exact = Self::shape_hash(tool_name, args);
        let store = self.grant_store.lock().unwrap_or_else(PoisonError::into_inner);

        if store.check_grant(tool_name, &exact, current_run_id).ok().flatten().is_some() {
            return true;
        }

        match tool_class {
            ToolClass::Execute => store
                .check_grant(tool_name, Self::SESSION_EXECUTE_SHAPE, current_run_id)
                .ok()
                .flatten()
                .is_some(),
            ToolClass::Write | ToolClass::WriteOutsideWorkspace => store
                .check_grant(
                    tool_name,
                    &Self::session_write_shape_hash(tool_name, args),
                    current_run_id,
                )
                .ok()
                .flatten()
                .is_some(),
            _ => false,
        }
    }

    /// Classifies risk from the actual call — `tool_name`/`args`, not just
    /// `tool_class` — so e.g. `rm -rf /` and `ls` are never bucketed
    /// together just because both are nominally "Execute". This is a
    /// policy-independent safety floor: [`Self::evaluate`] denies outright
    /// on [`RiskClassification::Forbidden`] regardless of how a workspace
    /// policy is configured.
    pub fn classify_risk(
        &self,
        tool_class: &ToolClass,
        _tool_name: &str,
        args: &Value,
    ) -> RiskClassification {
        match tool_class {
            ToolClass::Read => RiskClassification::Low,
            ToolClass::Network | ToolClass::Mcp => RiskClassification::Medium,
            ToolClass::Write => Self::classify_write_risk(args),
            ToolClass::Execute => Self::classify_execute_risk(args),
            ToolClass::WriteOutsideWorkspace => {
                if self.policy.filesystem.allow_outside_workspace {
                    RiskClassification::Critical
                } else {
                    RiskClassification::Forbidden
                }
            }
        }
    }

    /// A write to a path containing `..` traversal is `Critical` (`deny_globs`
    /// is a separate, independent check in [`Self::evaluate`]); an ordinary
    /// write is `Medium` (the default policy already routes `Write` through
    /// approval).
    fn classify_write_risk(args: &Value) -> RiskClassification {
        let Some(path) = args.get("path").and_then(Value::as_str) else {
            return RiskClassification::Medium;
        };
        if path.split(['/', '\\']).any(|segment| segment == "..") {
            return RiskClassification::Critical;
        }
        RiskClassification::Medium
    }

    /// See the threat model's execute-risk table: outright destructive/
    /// privilege-escalating commands are `Forbidden`; installs, builds, and
    /// any use of shell metacharacters are `Critical`; everything else
    /// (tests, linters, compilers, ordinary reads run via a shell tool) is
    /// `Elevated`, matching the previous unconditional default.
    fn classify_execute_risk(args: &Value) -> RiskClassification {
        let Some(command) = args.get("command").and_then(Value::as_str) else {
            return RiskClassification::Elevated;
        };
        if is_forbidden_rm_command(command)
            || FORBIDDEN_COMMAND_PATTERNS.iter().any(|re| re.is_match(command))
        {
            return RiskClassification::Forbidden;
        }
        if rm_has_recursive_and_force(command)
            || ELEVATED_COMMAND_PATTERNS.iter().any(|re| re.is_match(command))
        {
            return RiskClassification::Critical;
        }
        RiskClassification::Elevated
    }

    /// Evaluates the policy decision for a tool call and appends a record of
    /// it to the hash-chain audit log ([`crate::audit::AuditQueue`]) before
    /// returning — every call through this method is audited, not just the
    /// ones that happen to also get logged by a caller.
    pub fn evaluate(
        &self,
        tool_class: &ToolClass,
        tool_name: &str,
        args: &Value,
        workspace: Option<&Workspace>,
        mode: &RunMode,
        current_run_id: Option<&str>,
    ) -> Decision {
        let risk = self.classify_risk(tool_class, tool_name, args);
        let decision = self.evaluate_decision(
            *tool_class,
            tool_name,
            args,
            workspace,
            mode,
            current_run_id,
            &risk,
        );
        self.append_audit_event(*tool_class, tool_name, &risk, decision, current_run_id);
        decision
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_decision(
        &self,
        tool_class: ToolClass,
        tool_name: &str,
        args: &Value,
        workspace: Option<&Workspace>,
        _mode: &RunMode,
        current_run_id: Option<&str>,
        risk: &RiskClassification,
    ) -> Decision {
        // 1. Forbidden check
        if *risk == RiskClassification::Forbidden {
            return Decision::Deny;
        }

        // 1.5 Workspace trust: an untrusted workspace is read-only — no
        // execute, no network, no MCP, and no writes. This is not overridable
        // by policy, and it precedes every policy lookup below.
        if let Some(ws) = workspace
            && !ws.is_trusted()
            && !matches!(tool_class, ToolClass::Read)
        {
            return Decision::Deny;
        }

        // 2. Default decision from policy
        let mut decision = *self
            .policy
            .tools
            .default_decision
            .get(&tool_class)
            .unwrap_or(&Decision::NeedsApproval);

        // 3. Deny lists take precedence
        if let Some(deny) = &self.policy.tools.deny
            && deny.contains(&tool_name.to_string())
        {
            return Decision::Deny;
        }

        // 3.5 Filesystem policy: a write to a path matching `deny_globs`, or
        // whose content exceeds `max_file_write_bytes`, is denied outright —
        // these are hard limits the schema defines but that were previously
        // never read by any enforcement code.
        if matches!(tool_class, ToolClass::Write | ToolClass::WriteOutsideWorkspace) {
            if let Some(path) = args.get("path").and_then(Value::as_str)
                && Self::path_matches_deny_glob(&self.policy.filesystem.deny_globs, path)
            {
                return Decision::Deny;
            }
            if let Some(max_bytes) = self.policy.filesystem.max_file_write_bytes {
                let content_len =
                    args.get("content").and_then(Value::as_str).map_or(0, str::len) as u64;
                if content_len > max_bytes {
                    return Decision::Deny;
                }
            }
        }

        // 3.6 Commands policy: a command matching `deny_patterns` is denied
        // outright. A command that would touch the network (per
        // `CommandGuard`'s egress heuristic) is never silently `Allow`ed when
        // `network_during_execute` is off — it's escalated to approval below,
        // same as the "egress: true" approval/audit shape the docs describe.
        let mut command_requires_egress_approval = false;
        if matches!(tool_class, ToolClass::Execute)
            && let Some(cmd) = args.get("command").and_then(Value::as_str)
        {
            if Self::command_matches_deny_pattern(&self.policy.commands.deny_patterns, cmd) {
                return Decision::Deny;
            }
            if !self.policy.commands.network_during_execute
                && let Ok(shape) = CommandGuard::parse_and_verify(cmd)
            {
                command_requires_egress_approval = shape.requires_egress;
            }
        }

        // 4. Require approval check overrides Allow
        if let Some(req_app) = &self.policy.tools.require_approval
            && req_app.contains(&tool_class)
            && decision == Decision::Allow
        {
            decision = Decision::NeedsApproval;
        }
        if command_requires_egress_approval && decision == Decision::Allow {
            decision = Decision::NeedsApproval;
        }

        // 5. Check if it's needs approval, see if there's a standing grant
        if decision == Decision::NeedsApproval
            && self.has_standing_grant(tool_class, tool_name, args, current_run_id)
        {
            return Decision::Allow;
        }

        decision
    }

    /// User-facing explanation when [`Self::evaluate`] returns [`Decision::Deny`].
    pub fn explain_denial(
        &self,
        tool_class: &ToolClass,
        tool_name: &str,
        args: &Value,
        workspace: Option<&Workspace>,
    ) -> String {
        if let Some(ws) = workspace
            && !ws.is_trusted()
            && !matches!(*tool_class, ToolClass::Read)
        {
            return format!(
                "{tool_name} was blocked: this project is not trusted, so Eury can only read it. \
                 Trust the project in the app to allow writes and commands."
            );
        }

        let risk = self.classify_risk(tool_class, tool_name, args);
        if risk == RiskClassification::Forbidden {
            if matches!(tool_class, ToolClass::Execute)
                && let Some(cmd) = args.get("command").and_then(Value::as_str)
            {
                return format!(
                    "{tool_name} was blocked: `{cmd}` is forbidden by workspace policy \
                     (destructive system path, privilege escalation, or disallowed shell syntax)."
                );
            }
            return format!(
                "{tool_name} was blocked: this action is forbidden by workspace policy."
            );
        }

        if matches!(tool_class, ToolClass::Execute)
            && let Some(cmd) = args.get("command").and_then(Value::as_str)
            && Self::command_matches_deny_pattern(&self.policy.commands.deny_patterns, cmd)
        {
            return format!(
                "{tool_name} was blocked: `{cmd}` matches a blocked command pattern in workspace policy."
            );
        }

        if let Some(deny) = &self.policy.tools.deny
            && deny.contains(&tool_name.to_string())
        {
            return format!("{tool_name} is on the workspace policy deny list.");
        }

        format!("{tool_name} was blocked by the workspace policy.")
    }

    /// Checks `path` against `deny_globs`. Patterns containing `/` (e.g.
    /// `**/*.pem`, `~/.ssh/**`) match against the full path; bare patterns
    /// (e.g. `.env*`) match the basename at any depth — mirroring the
    /// `.gitignore` convention the preset's own patterns are written in.
    fn path_matches_deny_glob(deny_globs: &[String], path: &str) -> bool {
        let basename =
            std::path::Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path);
        deny_globs.iter().any(|pattern| {
            let Ok(glob) = Glob::new(pattern) else { return false };
            let matcher = glob.compile_matcher();
            if pattern.contains('/') { matcher.is_match(path) } else { matcher.is_match(basename) }
        })
    }

    /// Checks `command` against `deny_patterns` (regexes, e.g. `\bsudo\b`).
    fn command_matches_deny_pattern(deny_patterns: &[String], command: &str) -> bool {
        deny_patterns
            .iter()
            .any(|pattern| regex::Regex::new(pattern).is_ok_and(|re| re.is_match(command)))
    }

    /// Records a standing grant for the exact shape of `args`, after the
    /// user has approved a `NeedsApproval` decision for it, so that
    /// identical-shaped calls within the grant's scope don't re-prompt.
    ///
    /// The requested `scope` is clamped to the policy's
    /// `tools.max_grant_scope` for `tool_class`, if one is configured, so a
    /// grant can never be wider than the workspace policy allows —
    /// consistent with the "never-widen" guarantee: the grant only ever
    /// matches this `tool_name` + this exact argument shape, never "this
    /// tool, any arguments".
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying grant store write fails.
    pub fn record_grant(
        &self,
        tool_class: ToolClass,
        tool_name: &str,
        args: &Value,
        scope: GrantScope,
        run_id: Option<&str>,
    ) -> GrantStoreResult<()> {
        let allowed_scope = self
            .policy
            .tools
            .max_grant_scope
            .as_ref()
            .and_then(|m| m.get(&tool_class))
            .map_or(scope, |&max| scope.min(max));

        let shape_hash = if allowed_scope == GrantScope::Session {
            Self::session_grant_shape(tool_class, tool_name, args)
        } else {
            Self::shape_hash(tool_name, args)
        };

        let grant = Grant {
            id: uuid::Uuid::now_v7().to_string(),
            tool_class: format!("{tool_class:?}"),
            tool_name: tool_name.to_string(),
            shape_hash,
            scope: allowed_scope,
            run_id: if allowed_scope == GrantScope::Session {
                None
            } else {
                run_id.map(str::to_string)
            },
            expires_at: None,
        };

        let mut store = self.grant_store.lock().unwrap_or_else(PoisonError::into_inner);
        store.add_grant(&grant)
    }
}
