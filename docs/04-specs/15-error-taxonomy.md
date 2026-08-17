# Error Taxonomy

Spec-Version: 2.1.0

The complete, authoritative registry of error codes across the desktop core, the IPC surface, and the cloud Agent module. A code that is not in this table **MUST NOT** be emitted.

## Format

```
EURY_<DOMAIN>_<DETAIL>
```

| Rule | Detail |
|---|---|
| Stability | Codes are permanent API. Once shipped, a code is never renamed or repurposed |
| Domain | One of the domains listed below; no ad-hoc domains |
| Uniqueness | One code per distinct, actionable condition. Two conditions with the same fix share a code; two conditions with different fixes never do |
| Namespace hygiene | Error codes use domains like `RUN`, `TOOL`, `AUTH`. The `EURY_AGENT_` prefix is reserved for **environment variables** (`EURY_AGENT_DATA_DIR`) so a grep for either never returns the other |
| Deprecation | A retired code stays documented as deprecated with its replacement, for one major version |

## Error shape

```typescript
interface EuryError {
  code: string;              // from this registry
  message: string;           // English fallback; the UI renders from messageKey
  messageKey: string;        // i18n key, e.g. "errors.tool.pathOutside"
  params?: Record<string, string | number>;   // interpolation values, already redacted
  retryable: boolean;        // a retry could plausibly succeed
  recoverable: boolean;      // the session/run can continue
  httpStatus?: number;       // cloud errors only
  requestId?: string;        // cloud errors only, X-Agent-Request-Id
  details?: Record<string, unknown>;          // structured, never free-form user content
  docsUrl?: string;
  action?: {                 // the one thing the user should do
    kind: "retry" | "sign_in" | "grant" | "open_settings" | "contact_admin"
        | "upgrade_plan" | "update_app" | "reindex" | "restore" | "report";
    label: string;
  };
}
```

`retryable` and `recoverable` are distinct: a model rate limit is retryable and recoverable, a corrupt database is neither, and a cancelled run is recoverable but not retryable in place.

## Domains

| Domain | Layer | Meaning |
|---|---|---|
| `AUTH` | Both | Identity, sessions, tokens, device flow |
| `RUN` | Desktop | Run lifecycle and the model loop |
| `TOOL` | Desktop | Tool execution and sandbox enforcement |
| `POLICY` | Both | Policy and permission decisions |
| `APPROVAL` | Desktop | The approval flow |
| `PLAN` | Desktop | Plan parsing and building |
| `MCP` | Desktop | MCP servers |
| `MEMORY` | Desktop | Memory subsystem |
| `INDEX` | Desktop | Indexing and retrieval |
| `STORE` | Desktop | Local database and filesystem state |
| `WORKSPACE` | Desktop | Workspace state and trust |
| `IPC` | Desktop | The command surface itself |
| `UPDATE` | Desktop | Updates and integrity |
| `MODEL` | Both | Upstream provider failures |
| `GATEWAY` | Cloud | The managed model gateway |
| `QUOTA` | Cloud | Entitlement limits |
| `BUDGET` | Both | Spend caps |
| `ENTITLEMENT` | Cloud | Plan and seat rights |
| `AUDIT` | Both | Audit pipeline and integrity |
| `SYNC` | Both | Optional cloud sync |
| `DEVICE` | Cloud | Device registration and revocation |
| `SCIM` | Cloud | SCIM provisioning |
| `REQUEST` | Cloud | Request-level validation and routing |
| `INTERNAL` | Both | Unclassified failure; always a bug |

## AUTH

| Code | HTTP | Retry | Recover | Condition and action |
|---|---|---|---|---|
| `EURY_AUTH_UNAUTHORIZED` | 401 | no | yes | No valid session → sign in |
| `EURY_AUTH_SESSION_EXPIRED` | 401 | yes | yes | Access token expired → refresh, transparent to the user |
| `EURY_AUTH_REFRESH_FAILED` | 401 | no | yes | Refresh rejected → sign in again |
| `EURY_AUTH_REFRESH_REUSED` | 401 | no | no | A rotated refresh token was replayed. **The whole token family is revoked** and a security event is logged → sign in again |
| `EURY_AUTH_CODE_NOT_FOUND` | 404 | no | yes | Unknown device code → restart the device flow |
| `EURY_AUTH_CODE_EXPIRED` | 400 | no | yes | Device code expired → restart |
| `EURY_AUTH_PENDING` | 202 | yes | yes | Device flow not yet approved → keep polling (not an error to display) |
| `EURY_AUTH_SLOW_DOWN` | 429 | yes | yes | Polling too fast → increase the interval as instructed |
| `EURY_AUTH_DENIED` | 403 | no | yes | The user denied the device request |
| `EURY_AUTH_PKCE_INVALID` | 400 | no | yes | Malformed PKCE parameters → restart |
| `EURY_AUTH_PKCE_MISMATCH` | 400 | no | no | Verifier does not match the challenge. Treated as an attack signal and logged |
| `EURY_AUTH_DEVICE_REVOKED` | 401 | no | yes | An admin revoked this device → contact admin |
| `EURY_AUTH_SSO_REQUIRED` | 403 | no | yes | The organization requires SSO → use the SSO flow |
| `EURY_AUTH_ORG_SUSPENDED` | 403 | no | no | The organization is suspended → contact admin |
| `EURY_AUTH_PAT_INVALID` | 401 | no | yes | Personal access token invalid or expired |

## RUN

| Code | Retry | Recover | Condition and action |
|---|---|---|---|
| `EURY_RUN_ALREADY_ACTIVE` | yes | yes | A foreground run is active in this conversation → wait or cancel |
| `EURY_RUN_NOT_FOUND` | no | yes | Unknown run id |
| `EURY_RUN_CANCELLED` | no | yes | Cancelled by the user; informational, not shown as a failure |
| `EURY_RUN_TURN_LIMIT` | yes | yes | `maxTurns` reached → continue with a new turn or raise the limit |
| `EURY_RUN_TIME_LIMIT` | yes | yes | `maxWallMs` reached |
| `EURY_RUN_TOOL_CALL_LIMIT` | yes | yes | `maxToolCalls` reached |
| `EURY_RUN_CONTEXT_OVERFLOW` | no | yes | Compaction could not free enough context → start a new conversation |
| `EURY_RUN_COMPACTION_FAILED` | yes | yes | The summarization call failed → retry or compact manually |
| `EURY_RUN_QUEUE_FULL` | yes | yes | More than 10 queued runs |
| `EURY_RUN_SUBAGENT_DEPTH` | no | yes | Nesting depth above 2 |
| `EURY_RUN_SUBAGENT_FAILED` | yes | yes | A sub-agent failed; the parent decides how to proceed |
| `EURY_RUN_INVALID_TRANSITION` | no | no | Illegal state transition. Always a bug → report |
| `EURY_RUN_INTERRUPTED` | yes | yes | The app crashed mid-run → resume or revert |
| `EURY_RUN_ENGINE_FAILED` | yes | maybe | The agent engine returned an unusable state |

## TOOL

| Code | Retry | Recover | Condition and action |
|---|---|---|---|
| `EURY_TOOL_DENIED` | no | yes | The user or policy denied this call; returned to the model as data |
| `EURY_TOOL_NOT_FOUND` | no | yes | The path does not exist; suggestions included |
| `EURY_TOOL_PATH_OUTSIDE` | no | yes | Resolved outside the workspace → request a grant |
| `EURY_TOOL_PATH_DENIED` | no | yes | Matches the always-deny list; no grant can override it |
| `EURY_TOOL_SYMLINK_ESCAPE` | no | yes | A symlink resolved outside the boundary. Logged as a security event |
| `EURY_TOOL_COMMAND_FORBIDDEN` | no | yes | Matches a command deny pattern; no approval is offered |
| `EURY_TOOL_SHELL_REQUIRED` | no | yes | Metacharacters used without `shell: true` |
| `EURY_TOOL_TIMEOUT` | yes | yes | Exceeded its timeout; partial output retained |
| `EURY_TOOL_READ_TOO_LARGE` | no | yes | Above the 10 MB read limit → read a range |
| `EURY_TOOL_WRITE_TOO_LARGE` | no | yes | Above the 5 MB write limit |
| `EURY_TOOL_BINARY_FILE` | no | yes | Binary content cannot be read as text |
| `EURY_TOOL_STALE_WRITE` | yes | yes | The file changed since it was read → re-read and retry |
| `EURY_TOOL_EDIT_NO_MATCH` | no | yes | `oldString` not found |
| `EURY_TOOL_EDIT_AMBIGUOUS` | no | yes | Multiple matches; count and lines included |
| `EURY_TOOL_INVALID_INPUT` | no | yes | Schema validation failed; the field is named |
| `EURY_TOOL_SANDBOX_UNAVAILABLE` | no | no | The OS sandbox could not be established. Privileged tools refuse to run (fail closed) |
| `EURY_TOOL_EXECUTION_FAILED` | yes | yes | The tool crashed unexpectedly |
| `EURY_TOOL_NETWORK_BLOCKED` | no | yes | SSRF guard or an offline state blocked the request |
| `EURY_TOOL_UNAVAILABLE_IN_MODE` | no | yes | Not available in this mode; a bug if the model ever sees it |

## ATTACHMENT

| Code | Retry | Recover | Condition and action |
|---|---|---|---|
| `EURY_ATTACHMENT_MEDIA_UNSUPPORTED` | no | yes | The attachment MIME type is not supported → choose a supported image format |
| `EURY_ATTACHMENT_INVALID` | no | yes | The input is malformed or unsafe to decode → remove or replace it |
| `EURY_ATTACHMENT_TOO_LARGE` | no | yes | The image exceeds byte, dimension, or megapixel limits → resize it |
| `EURY_ATTACHMENT_SCAN_FAILED` | yes | yes | Required malware scan did not complete → retry or remove |
| `EURY_ATTACHMENT_EXPIRED` | no | yes | Managed attachment reference expired → reattach the image |
| `EURY_ATTACHMENT_ACCESS_DENIED` | no | yes | Attachment ownership, policy, or residency check failed |

## POLICY

| Code | HTTP | Retry | Recover | Condition and action |
|---|---|---|---|---|
| `EURY_POLICY_TOOL_DENIED` | — | no | yes | An organization rule forbids this tool → contact admin |
| `EURY_POLICY_MODEL_DENIED` | 403 | no | yes | The model is not allowed → choose an allowed model |
| `EURY_POLICY_NETWORK_DENIED` | 403 | no | yes | The domain is not allowed |
| `EURY_POLICY_SCOPE_DENIED` | — | no | yes | The requested grant scope exceeds the maximum; the maximum is included |
| `EURY_POLICY_STALE` | — | yes | yes | The cached policy is past its grace window → reconnect. Privileged tools are refused until refreshed |
| `EURY_POLICY_UNAVAILABLE` | — | yes | yes | No policy could be loaded; the strictest defaults apply |
| `EURY_POLICY_INVALID` | 400 | no | no | The policy document failed validation. The previous valid policy stays in force |
| `EURY_POLICY_SIGNATURE_INVALID` | — | no | no | Signature verification failed. The document is rejected outright |
| `EURY_POLICY_WORKSPACE_UNTRUSTED` | — | no | yes | The workspace is not trusted → trust it explicitly |

## APPROVAL

| Code | Retry | Recover | Condition |
|---|---|---|---|
| `EURY_APPROVAL_NOT_FOUND` | no | yes | Unknown id; never resolves as an allow |
| `EURY_APPROVAL_ALREADY_RESOLVED` | no | yes | Duplicate response; the first decision stands |
| `EURY_APPROVAL_TIMEOUT` | yes | yes | No response within the window; the run pauses rather than proceeding |
| `EURY_APPROVAL_EXPIRED` | no | yes | Responded after expiry; treated as a denial |
| `EURY_APPROVAL_SCOPE_INVALID` | no | yes | A scope was sent with a denial, or omitted from an allow |

## PLAN, MCP, MEMORY, INDEX

| Code | Retry | Recover | Condition |
|---|---|---|---|
| `EURY_PLAN_INVALID_FRONTMATTER` | no | yes | Front matter missing or unparseable |
| `EURY_PLAN_INVALID_STEPS` | no | yes | The `plan_steps` block is not a valid 1–30 element array |
| `EURY_PLAN_DUPLICATE_STEP` | no | yes | Duplicate step id |
| `EURY_PLAN_CYCLIC_DEPENDENCY` | no | yes | `dependsOn` forms a cycle |
| `EURY_PLAN_MULTIPLE_STEP_BLOCKS` | no | yes | More than one `plan_steps` fence |
| `EURY_PLAN_MISSING_SECTION` | no | yes | A required body section is absent |
| `EURY_PLAN_PATH_OUTSIDE` | no | yes | A `files` entry points outside the workspace |
| `EURY_PLAN_NOT_APPROVED` | no | yes | Build attempted before approval |
| `EURY_PLAN_EDITED` | no | yes | `stepsHash` mismatch → re-approve |
| `EURY_PLAN_STEP_FAILED` | yes | yes | A step or its verification failed → retry, skip, or revert |
| `EURY_PLAN_VERSION_UNSUPPORTED` | no | yes | Unknown major `specVersion` |
| `EURY_MCP_SPAWN_FAILED` | yes | yes | The executable could not be started |
| `EURY_MCP_HANDSHAKE_FAILED` | yes | yes | `initialize` failed or timed out |
| `EURY_MCP_VERSION_UNSUPPORTED` | no | yes | Incompatible protocol version |
| `EURY_MCP_FINGERPRINT_CHANGED` | no | yes | The tool set changed → re-approve |
| `EURY_MCP_NOT_APPROVED` | no | yes | The server is configured but not approved |
| `EURY_MCP_POLICY_DENIED` | no | yes | Blocked by the organization |
| `EURY_MCP_TIMEOUT` | yes | yes | The request exceeded its timeout |
| `EURY_MCP_PROTOCOL_ERROR` | yes | yes | Malformed response |
| `EURY_MCP_SERVER_CRASHED` | yes | yes | The server exited unexpectedly |
| `EURY_MCP_RESOURCE_EXCEEDED` | no | yes | Memory or CPU cap exceeded; the server was terminated |
| `EURY_MCP_SAMPLING_REFUSED` | no | yes | The server requested sampling, which is unsupported by design |
| `EURY_MEMORY_DISABLED` | no | yes | Memory is off by policy |
| `EURY_MEMORY_LIMIT_REACHED` | no | yes | Entry cap reached → prune |
| `EURY_MEMORY_PROPOSAL_NOT_FOUND` | no | yes | Unknown or expired proposal |
| `EURY_MEMORY_REJECTED_SECRET` | no | yes | The candidate contained a secret and was refused |
| `EURY_INDEX_UNAVAILABLE` | yes | yes | No index yet; live search is used instead |
| `EURY_INDEX_CORRUPT` | yes | yes | Quarantined and rebuilding |
| `EURY_INDEX_TOO_LARGE` | no | yes | Above the workspace size limit → scope the index |
| `EURY_INDEX_EMBEDDING_FAILED` | yes | yes | Embeddings unavailable; lexical retrieval continues |

## STORE, WORKSPACE, IPC, UPDATE

| Code | Retry | Recover | Condition and action |
|---|---|---|---|
| `EURY_STORE_KEY_UNAVAILABLE` | no | no | The keychain key is missing → repair or reset local data |
| `EURY_STORE_MIGRATION_FAILED` | no | no | Migration failed; the pre-migration backup was restored |
| `EURY_STORE_MIGRATION_TAMPERED` | no | no | A migration checksum changed → reinstall |
| `EURY_STORE_SCHEMA_TOO_NEW` | no | no | The database is newer than the app → update the app |
| `EURY_STORE_CORRUPT` | no | no | Integrity check failed; recovery attempted, the file is quarantined |
| `EURY_STORE_DISK_FULL` | yes | yes | Insufficient space. Mutating tools stop rather than writing unrecoverably |
| `EURY_STORE_LOCKED` | yes | yes | Another instance holds the write lock |
| `EURY_STORE_READ_ONLY` | no | yes | Degraded mode after a failed migration |
| `EURY_CHECKPOINT_WRITE_FAILED` | no | yes | The checkpoint could not be written, so the tool did not run |
| `EURY_CHECKPOINT_NOT_FOUND` | no | yes | Unknown checkpoint |
| `EURY_CHECKPOINT_OBJECT_MISSING` | no | yes | Stored content is missing; the affected paths are named as unrestorable |
| `EURY_CHECKPOINT_CONFIRM_INVALID` | no | yes | Missing, expired, or mismatched confirm token |
| `EURY_CHECKPOINT_CONFLICT` | no | yes | Files changed since the checkpoint → resolve per file |
| `EURY_WORKSPACE_NOT_OPEN` | no | yes | The command requires an open workspace |
| `EURY_WORKSPACE_UNTRUSTED` | no | yes | The command requires trust |
| `EURY_WORKSPACE_NOT_FOUND` | no | yes | The path no longer exists → relocate or remove |
| `EURY_WORKSPACE_TOO_LARGE` | no | yes | Above the file-count threshold → confirm or scope |
| `EURY_IPC_UNKNOWN_COMMAND` | no | yes | Not in the registry; indicates a version mismatch |
| `EURY_IPC_INVALID_ARGS` | no | yes | Deserialization failed; the field is named |
| `EURY_IPC_RATE_LIMITED` | yes | yes | Command rate limit exceeded; `retryAfterMs` included |
| `EURY_IPC_VERSION_MISMATCH` | no | no | `ipcApiVersion` incompatible → restart to finish updating |
| `EURY_UPDATE_UNAVAILABLE` | yes | yes | The update service is unreachable |
| `EURY_UPDATE_INTEGRITY_FAILED` | no | yes | Signature or hash verification failed. The artifact is deleted and a security event is logged |
| `EURY_UPDATE_DOWNLOAD_FAILED` | yes | yes | Download interrupted; resumable |
| `EURY_UPDATE_INSTALL_FAILED` | yes | yes | The installer failed; the current version stays intact |
| `EURY_UPDATE_REQUIRED` | no | no | Below `minSupported` → update to continue |
| `EURY_CLIENT_TOO_OLD` | no | no | The cloud refuses this client version |

## MODEL, GATEWAY, QUOTA, BUDGET, ENTITLEMENT

| Code | HTTP | Retry | Recover | Condition and action |
|---|---|---|---|---|
| `EURY_MODEL_KEY_MISSING` | — | no | yes | No BYOK key for this provider → open settings |
| `EURY_MODEL_KEY_INVALID` | — | no | yes | The provider rejected the key |
| `EURY_MODEL_RATE_LIMITED` | 429 | yes | yes | Upstream provider rate limit; backoff applied automatically |
| `EURY_MODEL_UNAVAILABLE` | 503 | yes | yes | Provider outage → retry or switch model |
| `EURY_MODEL_CONTEXT_EXCEEDED` | 400 | no | yes | The request exceeded the model's window → compact |
| `EURY_MODEL_CONTENT_FILTERED` | 400 | no | yes | The provider refused the content |
| `EURY_MODEL_NOT_FOUND` | 404 | no | yes | Unknown model id |
| `EURY_MODEL_RESPONSE_INVALID` | — | yes | yes | Malformed stream or tool call; retried once |
| `EURY_MODEL_VISION_UNSUPPORTED` | 400 | no | yes | Selected model cannot accept image input → choose a vision-capable model |
| `EURY_GATEWAY_UNAVAILABLE` | 503 | yes | yes | The managed gateway is down → retry, or fall back to BYOK where policy allows |
| `EURY_GATEWAY_TIMEOUT` | 504 | yes | yes | Upstream timeout |
| `EURY_QUOTA_EXCEEDED` | 429 | yes | yes | An entitlement limit (daily messages, monthly tokens) was hit → wait for reset or upgrade |
| `EURY_QUOTA_CONCURRENCY` | 429 | yes | yes | Too many concurrent runs |
| `EURY_BUDGET_EXCEEDED` | 402 | no | yes | An org or user spend budget was hit → contact admin |
| `EURY_COST_CAP_EXCEEDED` | — | no | yes | The per-run cost cap aborted the run |
| `EURY_ENTITLEMENT_NO_SEAT` | 403 | no | yes | No Agent seat assigned → contact admin |
| `EURY_ENTITLEMENT_PLAN_REQUIRED` | 403 | no | yes | The feature needs a higher plan → upgrade |

## AUDIT, SYNC, DEVICE, SCIM, REQUEST, INTERNAL

| Code | HTTP | Retry | Recover | Condition |
|---|---|---|---|---|
| `EURY_AUDIT_QUEUE_FULL` | — | no | yes | The local queue is full. Privileged tools stop rather than dropping records |
| `EURY_AUDIT_UPLOAD_FAILED` | — | yes | yes | Upload failed; events stay queued |
| `EURY_AUDIT_SIGNATURE_INVALID` | 400 | no | no | Server-side signature verification failed. Logged as a security event |
| `EURY_AUDIT_SEQUENCE_GAP` | 409 | no | yes | A gap in the device chain was detected; investigated as tampering |
| `EURY_SYNC_DISABLED` | — | no | yes | Sync is off by policy or preference |
| `EURY_SYNC_CONFLICT` | 409 | no | yes | Concurrent edits; last-write-wins with the loser preserved |
| `EURY_SYNC_CURSOR_INVALID` | 400 | no | yes | Stale cursor → full resync |
| `EURY_DEVICE_LIMIT_REACHED` | 403 | no | yes | Device cap for the plan → revoke a device |
| `EURY_DEVICE_NOT_FOUND` | 404 | no | yes | Unknown device |
| `EURY_SCIM_SCHEMA_INVALID` | 400 | no | yes | The SCIM payload failed validation |
| `EURY_SCIM_CONFLICT` | 409 | no | yes | Duplicate `externalId` or `userName` |
| `EURY_REQUEST_INVALID` | 400 | no | yes | Validation failed; the offending fields are listed |
| `EURY_REQUEST_NOT_FOUND` | 404 | no | yes | Unknown resource |
| `EURY_REQUEST_FORBIDDEN` | 403 | no | yes | Authenticated but not authorized |
| `EURY_REQUEST_CONFLICT` | 409 | no | yes | Concurrent modification |
| `EURY_REQUEST_PAYLOAD_TOO_LARGE` | 413 | no | yes | Above the body limit |
| `EURY_RATE_LIMITED` | 429 | yes | yes | An Agent API rate limit; `Retry-After` is set |
| `EURY_OFFLINE` | — | yes | yes | No network. Local features continue working |
| `EURY_UPSTREAM_FAILED` | 502 | yes | yes | A dependency failed |
| `EURY_INTERNAL` | 500 | yes | maybe | Unclassified failure. Always a bug; `requestId` is shown for reporting |

## HTTP status mapping

| Status | Codes |
|---|---|
| 400 | `REQUEST_INVALID`, `AUTH_PKCE_*`, `AUTH_CODE_EXPIRED`, `POLICY_INVALID`, `SCIM_SCHEMA_INVALID`, `MODEL_CONTEXT_EXCEEDED` |
| 401 | All `AUTH_*` session and token failures |
| 402 | `BUDGET_EXCEEDED` only |
| 403 | `REQUEST_FORBIDDEN`, `POLICY_*_DENIED`, `ENTITLEMENT_*`, `AUTH_DENIED`, `AUTH_SSO_REQUIRED`, `DEVICE_LIMIT_REACHED` |
| 404 | `*_NOT_FOUND` |
| 409 | `REQUEST_CONFLICT`, `SYNC_CONFLICT`, `SCIM_CONFLICT`, `AUDIT_SEQUENCE_GAP` |
| 413 | `REQUEST_PAYLOAD_TOO_LARGE` |
| 429 | `RATE_LIMITED`, `QUOTA_*`, `MODEL_RATE_LIMITED`, `AUTH_SLOW_DOWN` |
| 500 | `INTERNAL` |
| 502 | `UPSTREAM_FAILED` |
| 503 | `GATEWAY_UNAVAILABLE`, `MODEL_UNAVAILABLE` |
| 504 | `GATEWAY_TIMEOUT` |

402 is reserved exclusively for spend budgets, and 429 exclusively for rate and entitlement limits. Conflating them makes client retry logic wrong: one should never be retried, the other always should.

## Implementation mapping

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("path resolves outside the workspace")]
    ToolPathOutside { path_hash: String },
    // …one variant per code
}

impl AgentError {
    pub fn code(&self) -> &'static str { /* generated */ }
    pub fn retryable(&self) -> bool { /* generated */ }
    pub fn recoverable(&self) -> bool { /* generated */ }
    pub fn message_key(&self) -> &'static str { /* generated */ }
}
```

| Rule | Detail |
|---|---|
| One variant per code | A generated exhaustiveness test fails the build on drift between the enum and this table |
| No stringly-typed errors | Constructing an error from a raw string is banned by a Semgrep rule ([security testing](../08-quality/04-security-testing.md)) |
| Redaction | Error `params` and `details` pass the redactor. Paths appear as hashes in logs and as relative paths in the UI only |
| Cloud | A NestJS exception filter maps every `EuryException` to this shape and attaches `requestId` |
| i18n | Every code has a `messageKey` present in every shipped locale, verified in CI ([i18n](../05-ui/07-accessibility-and-i18n.md)) |
| Logging | `warn` for user-recoverable, `error` for unrecoverable, `error` plus a security tag for the attack-signal codes |

## Security-signal codes

These indicate a possible attack, not a user mistake. They are logged at `error` with a security tag, audited, and alerted on when the rate rises ([observability](../07-ops/05-observability-and-slos.md)):

`EURY_AUTH_REFRESH_REUSED`, `EURY_AUTH_PKCE_MISMATCH`, `EURY_TOOL_SYMLINK_ESCAPE`, `EURY_TOOL_PATH_DENIED`, `EURY_POLICY_SIGNATURE_INVALID`, `EURY_UPDATE_INTEGRITY_FAILED`, `EURY_AUDIT_SIGNATURE_INVALID`, `EURY_AUDIT_SEQUENCE_GAP`, `EURY_MCP_FINGERPRINT_CHANGED`, `EURY_MEMORY_REJECTED_SECRET`.

## UI presentation

| Class | Presentation |
|---|---|
| Recoverable, retryable | Inline notice with a retry button; the run stays usable |
| Recoverable, not retryable | Inline notice with the specific `action` |
| Policy denial | Non-dismissible inline notice naming the rule and its scope, with a contact-admin action |
| Tool error | A tool card in the error state; expandable for detail. Not a modal |
| Unrecoverable | Full-surface error state with the request id, a copy-details action, and a report action |
| Security signal | Persistent banner until acknowledged |

Never shown to the user: a raw Rust `Debug` string, a stack trace, a provider error body, or an absolute filesystem path.

## Conformance tests

| ID | Test |
|---|---|
| T1 | Every enum variant appears in this document and vice versa (generated, fails on drift) |
| T2 | Every code has a `messageKey` present in all shipped locales |
| T3 | HTTP mapping matches this table for every cloud code |
| T4 | No error message or `details` payload contains a seeded secret or an absolute path |
| T5 | `retryable: false` codes are never automatically retried by any client path |
| T6 | Every security-signal code produces an audit event and a tagged log line |
| T7 | Fuzzing every IPC command yields only registry codes, never `EURY_INTERNAL` |
| T8 | A deprecated code still resolves, with its replacement documented |

T7 is the useful one in practice: any `EURY_INTERNAL` reached by fuzzing is an unclassified failure path, which means an unhandled case somewhere.

## Related documents

- [Agent runtime](01-agent-runtime-spec.md)
- [IPC commands](04-ipc-command-spec.md)
- [Cloud API contract](06-cloud-api-contract.md)
- [Permission and policy engine](../03-security/03-permission-and-policy-engine.md)
- [Accessibility and i18n](../05-ui/07-accessibility-and-i18n.md)
