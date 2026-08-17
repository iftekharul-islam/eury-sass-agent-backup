# IPC Command Specification

Spec-Version: 2.1.0

Every Tauri `invoke` command exposed by the Rust core. The IPC surface is an attack surface: it is enumerated here, and anything not listed **MUST NOT** be registered.

## General rules

| # | Rule |
|---|---|
| G1 | All commands return `Result<T, IpcError>`; the UI never receives a bare string error |
| G2 | Inputs are typed structs with `#[serde(deny_unknown_fields)]`; unknown fields are a hard error |
| G3 | Path arguments follow the full normalization and boundary checks from the [tool catalog](02-tool-catalog-spec.md) — UI-originated paths are not more trusted than model-originated paths |
| G4 | No command returns a secret. Keys are write-only; existence is queryable, values are not |
| G5 | Commands are non-blocking; anything above 50 ms is `async` and reports progress via events |
| G6 | Commands that mutate state are idempotent or take an idempotency key |
| G7 | Every command is rate-limited per window; abuse produces `EURY_IPC_RATE_LIMITED` |
| G8 | Commands requiring a workspace fail with `EURY_WORKSPACE_NOT_OPEN` rather than falling back to the process cwd |
| G9 | Commands requiring trust fail with `EURY_WORKSPACE_UNTRUSTED` ([workspace trust](../05-ui/05-approval-and-trust-ux.md)) |
| G10 | Every command emits an audit event when it changes security-relevant state |

```typescript
interface IpcError {
  code: string;                    // error taxonomy code
  message: string;
  retryable: boolean;
  details?: Record<string, unknown>;
}
```

## Capability annotations

| Symbol | Meaning |
|---|---|
| `W` | Requires an open workspace |
| `T` | Requires the workspace to be trusted |
| `A` | Requires an authenticated session |
| `!` | Security-relevant: always audited |
| `~` | Long-running: reports progress via events |

## Workspace

| Command | Input | Output | Flags |
|---|---|---|---|
| `workspace_open` | `{ path, window?: "current" \| "new" }` | `WorkspaceInfo` | `!` `~` |
| `workspace_close` | `{}` | `void` | |
| `workspace_info` | `{}` | `WorkspaceInfo \| null` | |
| `workspace_recent` | `{ limit?: number }` | `RecentWorkspace[]` | |
| `workspace_trust_set` | `{ path, trusted: boolean }` | `WorkspaceInfo` | `!` |
| `workspace_reindex` | `{ full?: boolean }` | `{ jobId: string }` | `W` `~` |
| `workspace_index_status` | `{}` | `IndexStatus` | `W` |

```typescript
interface WorkspaceInfo {
  id: string; path: string; name: string;
  trusted: boolean; isGitRepo: boolean; gitBranch?: string;
  languages: string[]; fileCount: number;
  policyDigest: string; policySource: "org" | "workspace" | "user" | "default";
  indexState: "none" | "building" | "ready" | "stale" | "error";
  openedAt: string;
}
```

`workspace_open` refuses paths that are a filesystem root, the user's home directory, a system directory, or larger than 200 000 files without explicit confirmation, because those are almost always mistakes with expensive consequences.

## Agent runs

| Command | Input | Output | Flags |
|---|---|---|---|
| `agent_run_start` | `RunRequest` + `Channel<AgentEvent>` | `{ runId, queuePosition }` | `W` `T` `!` `~` |
| `agent_run_cancel` | `{ runId, reason? }` | `void` | |
| `agent_run_snapshot` | `{ runId }` | `RunSnapshot` | |
| `agent_run_list` | `{ conversationId?, status?, limit?, cursor? }` | `Page<RunSummary>` | |
| `agent_run_resume` | `{ runId }` | `{ runId }` | `W` `T` `!` |
| `agent_steer` | `{ runId, text }` | `void` | |
| `agent_compact` | `{ runId }` | `void` | |
| `agent_estimate` | `{ prompt, mode, model }` | `{ promptTokens, estimatedCostUsdMicros }` | `W` |

`agent_run_start` returns as soon as the run is registered and persisted — it does not wait for the model. The channel is the only path for run output. Cancellation is idempotent: cancelling a finished run succeeds silently.

```typescript
interface RunSnapshot {
  runId: string; seq: number;               // resume point for the event stream
  phase: RunPhase; mode: Mode; model: string;
  messages: Message[];                      // full turn history for this run
  toolCalls: ToolCallRecord[];
  pendingApprovals: ApprovalRequest[];
  usage: TokenUsage; costUsdMicros: number;
  contextUsage: { usedTokens: number; windowTokens: number };
  filesChanged: FileChange[];
  startedAt: string; updatedAt: string;
}
```

## Approvals and grants

| Command | Input | Output | Flags |
|---|---|---|---|
| `approval_respond` | `{ approvalId, decision: "allow" \| "deny", scope?: GrantScope, note? }` | `void` | `!` |
| `approval_list_pending` | `{}` | `ApprovalRequest[]` | |
| `grant_list` | `{ workspaceId? }` | `Grant[]` | |
| `grant_revoke` | `{ grantId }` | `void` | `!` |
| `grant_revoke_all` | `{ workspaceId? }` | `{ revoked: number }` | `!` |

Rules: an unknown or already-resolved `approvalId` returns `EURY_APPROVAL_NOT_FOUND` and never silently allows. `scope` is required for `allow` and forbidden for `deny`. Requesting a scope the policy forbids returns `EURY_POLICY_SCOPE_DENIED` with the maximum permitted scope in `details`. Grants are stored with the normalized tool, argument shape, and workspace, never as a blanket "allow everything".

## Policy

| Command | Input | Output | Flags |
|---|---|---|---|
| `policy_effective` | `{ workspaceId? }` | `EffectivePolicy` | `W` |
| `policy_explain` | `{ tool, args }` | `PolicyExplanation` | `W` |
| `policy_refresh` | `{}` | `{ digest, source, fetchedAt }` | `A` `!` |
| `policy_set_user` | `UserPolicyPatch` | `EffectivePolicy` | `!` |

`policy_explain` is the debugging primitive that makes the system trustworthy: it returns the decision, the winning rule, the rule's source level, and the full merge trace ([policy engine](../03-security/03-permission-and-policy-engine.md)).

## Files

| Command | Input | Output | Flags |
|---|---|---|---|
| `file_read` | `{ path, encoding? }` | `{ content, encoding, eol, sizeBytes, sha256, truncated }` | `W` |
| `file_write` | `{ path, content, expectedSha256? }` | `{ sha256 }` | `W` `T` `!` |
| `file_list` | `{ path?, includeIgnored? }` | `FileEntry[]` | `W` |
| `file_search` | `{ query, kind: "name" \| "content", limit? }` | `SearchHit[]` | `W` |
| `file_stat` | `{ path }` | `FileStat` | `W` |
| `file_reveal` | `{ path }` | `void` | `W` |
| `file_open_external` | `{ path }` | `void` | `W` `T` `!` |

`file_write` from the UI is the user's own edit in the built-in editor. It still passes the path guards, still respects `expectedSha256` for conflict detection, and is still audited — the UI is not a policy bypass. `file_open_external` refuses executable extensions.

## Diffs and checkpoints

| Command | Input | Output | Flags |
|---|---|---|---|
| `diff_for_file` | `{ path, against: "checkpoint" \| "disk" \| "git" }` | `FileDiff` | `W` |
| `diff_for_run` | `{ runId }` | `FileDiff[]` | `W` |
| `checkpoint_list` | `{ runId?, workspaceId? }` | `Checkpoint[]` | `W` |
| `checkpoint_preview_restore` | `{ checkpointId }` | `RestorePlan` | `W` |
| `checkpoint_restore` | `{ checkpointId, paths?, confirmToken }` | `RestoreResult` | `W` `T` `!` |
| `checkpoint_delete` | `{ checkpointId }` | `void` | `!` |

Restore is two-phase: `checkpoint_preview_restore` returns the exact file list, conflicts, and a `confirmToken`; `checkpoint_restore` requires that token. This makes accidental destructive restores structurally impossible ([checkpoints](12-checkpoint-and-rollback-spec.md)).

## Terminal

| Command | Input | Output | Flags |
|---|---|---|---|
| `terminal_create` | `{ cwd?, shell?, cols, rows }` | `{ terminalId }` | `W` `T` `!` |
| `terminal_write` | `{ terminalId, data }` | `void` | |
| `terminal_resize` | `{ terminalId, cols, rows }` | `void` | |
| `terminal_close` | `{ terminalId }` | `void` | |
| `terminal_list` | `{}` | `TerminalInfo[]` | |
| `terminal_capture` | `{ terminalId, lines? }` | `{ text: string }` | |

Terminal output flows over its own dedicated `Channel<TerminalFrame>` per session, created by `terminal_create` and torn down on session close — never the run channel or a global event topic ([event protocol](03-event-protocol-spec.md)). `terminal_capture` is how the user attaches output to a prompt; captured text is marked `untrusted`. Terminals are capped at 4 per workspace.

## Conversations and history

| Command | Input | Output | Flags |
|---|---|---|---|
| `conversation_list` | `{ workspaceId?, limit?, cursor?, query? }` | `Page<ConversationSummary>` | |
| `conversation_create` | `{ workspaceId?, title?, mode? }` | `Conversation` | |
| `conversation_get` | `{ id, messageLimit?, cursor? }` | `Conversation` | |
| `conversation_update` | `{ id, title?, pinned?, archived? }` | `Conversation` | |
| `conversation_delete` | `{ id }` | `void` | `!` |
| `conversation_export` | `{ id, format: "md" \| "json" }` | `{ path }` | `!` |
| `message_delete` | `{ id }` | `void` | `!` |
| `message_edit_and_rerun` | `{ id, text }` | `{ runId }` | `W` `T` |

## Memory

| Command | Input | Output | Flags |
|---|---|---|---|
| `memory_list` | `{ workspaceId?, kind?, query?, limit? }` | `MemoryEntry[]` | |
| `memory_add` | `{ kind, text, scope }` | `MemoryEntry` | `!` |
| `memory_update` | `{ id, text?, pinned? }` | `MemoryEntry` | `!` |
| `memory_delete` | `{ id }` | `void` | `!` |
| `memory_proposal_respond` | `{ proposalId, accept: boolean, editedText? }` | `void` | `!` |
| `memory_export` | `{ format: "json" \| "md" }` | `{ path }` | |
| `memory_purge` | `{ workspaceId?, confirmToken }` | `{ deleted: number }` | `!` |

## Plans

| Command | Input | Output | Flags |
|---|---|---|---|
| `plan_list` | `{ workspaceId }` | `PlanSummary[]` | `W` |
| `plan_get` | `{ planId }` | `Plan` | `W` |
| `plan_save` | `{ planId?, markdown }` | `Plan` | `W` `T` |
| `plan_approve` | `{ planId }` | `Plan` | `W` `T` `!` |
| `plan_build_step` | `{ planId, stepId }` | `{ runId }` | `W` `T` `!` |
| `plan_delete` | `{ planId }` | `void` | `!` |

## Model configuration

| Command | Input | Output | Flags |
|---|---|---|---|
| `models_list` | `{ refresh?: boolean }` | `ModelInfo[]` | |
| `models_test` | `{ provider, modelId }` | `{ ok, latencyMs, error? }` | `!` |
| `provider_key_set` | `{ provider, key }` | `{ ok, verified }` | `!` |
| `provider_key_status` | `{}` | `{ provider, present, lastVerifiedAt }[]` | |
| `provider_key_delete` | `{ provider }` | `void` | `!` |

Keys go straight to the OS keychain and are never returned, logged, echoed in errors, or written to the SQLite store (G4). `provider_key_set` optionally performs a minimal validation call and reports only success or a sanitized failure.

## Authentication

| Command | Input | Output | Flags |
|---|---|---|---|
| `auth_device_start` | `{}` | `{ userCode, verificationUri, expiresAt, interval }` | `!` |
| `auth_device_poll` | `{}` | `AuthPollResult` | `!` |
| `auth_device_cancel` | `{}` | `void` | |
| `auth_session` | `{}` | `SessionInfo \| null` | |
| `auth_refresh` | `{}` | `SessionInfo` | `!` |
| `auth_logout` | `{ everywhere?: boolean }` | `void` | `!` |
| `auth_pat_set` | `{ token }` | `SessionInfo` | `!` |

The PKCE `code_verifier` never crosses the IPC boundary: it is generated in Rust, held in memory, and discarded after the exchange. The UI only ever sees the user code and the verification URI ([identity](../06-enterprise/01-identity-sso-scim.md)).

## Settings

| Command | Input | Output | Flags |
|---|---|---|---|
| `settings_get` | `{}` | `Settings` | |
| `settings_set` | `SettingsPatch` | `Settings` | `!` |
| `settings_reset` | `{ section? }` | `Settings` | `!` |
| `capabilities_get` | `{}` | `Capabilities` | |

```typescript
interface Capabilities {
  ipcApiVersion: number;            // integer, bumped on any breaking IPC change
  eventSpecVersion: string;         // event protocol semver
  appVersion: string; buildSha: string; channel: string;
  platform: { os: string; arch: string; osVersion: string };
  features: Record<string, boolean>;   // flags: mcp, index, subagents, sync, terminal
  sandbox: { available: boolean; kind: "seatbelt" | "landlock" | "job_object" | "none" };
  limits: { maxFileSizeMb: number; maxTerminals: number; maxParallelTools: number };
  offline: boolean; airGapped: boolean;
}
```

The UI calls `capabilities_get` first on every launch and refuses to render features the core does not advertise. A missing sandbox surfaces a persistent security banner rather than being silently ignored.

## MCP

| Command | Input | Output | Flags |
|---|---|---|---|
| `mcp_server_list` | `{}` | `McpServerInfo[]` | |
| `mcp_server_add` | `McpServerConfig` | `McpServerInfo` | `!` |
| `mcp_server_approve` | `{ serverId, approved: boolean }` | `McpServerInfo` | `!` |
| `mcp_server_remove` | `{ serverId }` | `void` | `!` |
| `mcp_server_restart` | `{ serverId }` | `McpServerInfo` | `!` |
| `mcp_tool_list` | `{ serverId }` | `McpToolInfo[]` | |

## Updates

| Command | Input | Output | Flags |
|---|---|---|---|
| `update_check` | `{ force?: boolean }` | `UpdateStatus` | `~` |
| `update_download` | `{ version }` | `{ jobId }` | `!` `~` |
| `update_install` | `{ version }` | `void` | `!` |
| `update_defer` | `{ version, until }` | `void` | |

`update_download` accepts a version, not a URL. The URL comes from the signed manifest inside the core, so a compromised UI cannot redirect the download (G4 in spirit: never let the renderer choose what gets executed).

## Diagnostics

| Command | Input | Output | Flags |
|---|---|---|---|
| `diagnostics_bundle` | `{ includeLogs?: boolean }` | `{ path }` | `!` |
| `logs_tail` | `{ lines? }` | `{ lines: string[] }` | |
| `audit_local_list` | `{ limit?, cursor? }` | `Page<AuditEvent>` | |
| `audit_flush` | `{}` | `{ uploaded: number, pending: number }` | `A` |
| `telemetry_set` | `{ enabled: boolean }` | `Settings` | `!` |

The diagnostics bundle is redacted, shows the user a manifest of exactly what it contains before writing, and is never uploaded automatically.

## Rate limits

| Group | Limit |
|---|---|
| `agent_run_start` | 10 per minute per window |
| `file_read`, `file_stat` | 300 per minute |
| `file_write` | 60 per minute |
| `file_search`, `diff_*` | 120 per minute |
| `auth_*` | 10 per minute |
| `provider_key_*` | 10 per minute |
| `terminal_create` | 20 per minute |
| Everything else | 600 per minute |

Exceeding a limit returns `EURY_IPC_RATE_LIMITED` with `retryAfterMs`. Limits exist to bound damage from a compromised renderer, not to constrain normal users — normal use is orders of magnitude below them.

## Versioning

`ipcApiVersion` is a single integer for the whole surface. Additive commands and optional fields do not bump it. Removing a command, renaming a field, or changing semantics does. On startup the UI compares its required version with `capabilities_get`; a mismatch shows a blocking update state instead of failing at a random click.

## Conformance tests

| ID | Test |
|---|---|
| T1 | The registered command list exactly equals this document (generated test, fails on drift) |
| T2 | Every command rejects unknown fields, wrong types, and null where non-null is required |
| T3 | Every path-taking command rejects the traversal and symlink corpus |
| T4 | No command's success or error output contains a seeded secret |
| T5 | `W`/`T`/`A` flags are enforced: calling without the precondition returns the specified code |
| T6 | `approval_respond` with an unknown, expired, or duplicate id never allows |
| T7 | Rate limits trigger at the documented thresholds and reset correctly |
| T8 | `checkpoint_restore` without a valid `confirmToken` is refused |
| T9 | All `!` commands produce exactly one audit row |
| T10 | Concurrent identical mutating calls with the same idempotency key produce one effect |

## Related documents

- [Event protocol](03-event-protocol-spec.md)
- [Agent runtime](01-agent-runtime-spec.md)
- [Threat model](../03-security/01-threat-model.md)
- [Error taxonomy](15-error-taxonomy.md)
