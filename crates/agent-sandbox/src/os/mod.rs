use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxProbe {
    pub id: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// Field set mirrors the `sandbox-capabilities` wire schema (docs/04-specs/schemas);
// splitting into a bitflag/enum would break the JSON contract consumers rely on.
#[allow(clippy::struct_excessive_bools)]
pub struct SandboxCapabilities {
    pub schema_version: u32,
    pub os: String,
    pub os_version: Option<String>,
    pub mechanism: Option<String>,
    pub mechanism_version: Option<String>,
    pub filesystem_guard: bool,
    pub process_isolation: bool,
    pub egress_enforcement: bool,
    pub outside_root_capabilities: bool,
    pub privileged_tools_enabled: bool,
    pub verified_at: String,
    pub probes: Vec<SandboxProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_codes: Option<Vec<String>>,
}

/// Runs the real, platform-specific sandbox capability probes and reports
/// what they actually found — never a claim of containment nobody verified.
///
/// - **macOS**: spawns real child processes under a `(deny default)`
///   Seatbelt profile (via `sandbox-exec`) and confirms the forbidden
///   filesystem/egress operation actually failed.
/// - **Linux / Windows**: not yet implemented (see `linux.rs`/`windows.rs`
///   for why); reports `passed: false` honestly rather than an unverified
///   `true`.
#[must_use]
pub fn probe_capabilities() -> SandboxCapabilities {
    #[cfg(target_os = "macos")]
    {
        macos::probe()
    }
    #[cfg(target_os = "linux")]
    {
        linux::probe()
    }
    #[cfg(target_os = "windows")]
    {
        windows::probe()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        unsupported_os_capabilities()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn unsupported_os_capabilities() -> SandboxCapabilities {
    let probe_ids = [
        "outside_root_read_denied",
        "outside_root_write_denied",
        "process_tree_contained",
        "privilege_escalation_denied",
        "egress_denied",
    ];
    let probes: Vec<SandboxProbe> = probe_ids
        .iter()
        .map(|id| SandboxProbe {
            id: (*id).to_string(),
            passed: false,
            reason_code: Some("unsupported_os".to_string()),
        })
        .collect();

    SandboxCapabilities {
        schema_version: 1,
        os: "unknown".to_string(),
        os_version: None,
        mechanism: None,
        mechanism_version: None,
        filesystem_guard: false,
        process_isolation: false,
        egress_enforcement: false,
        outside_root_capabilities: false,
        privileged_tools_enabled: false,
        verified_at: chrono::Utc::now().to_rfc3339(),
        probes,
        reason_codes: Some(vec!["unsupported_os".to_string()]),
    }
}
