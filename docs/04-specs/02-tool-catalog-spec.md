# Tool Catalog Specification

Spec-Version: 2.2.0

Every tool is a security boundary. This document is the normative contract for tool identity, classification, schemas, limits, results, and failure behavior. A tool that is not specified here **MUST NOT** be registered.

## Tool contract

```typescript
interface ToolDefinition {
  name: string;                  // ^[a-z][a-z0-9_]{2,63}$ — stable forever once shipped
  version: number;               // bumped on any schema or semantic change
  title: string;                 // human label for tool cards
  description: string;           // model-facing; <= 1024 chars, no marketing language
  inputSchema: JSONSchema;       // draft 2020-12, additionalProperties: false
  class: ToolClass;              // drives policy and approval
  risk: "low" | "medium" | "elevated" | "critical";
  modes: Mode[];                 // [] means all modes
  idempotent: boolean;           // safe to retry without side effects
  mutates: boolean;              // touches workspace state
  checkpointed: boolean;         // runtime snapshots before execution
  timeoutMs: number;
  concurrency: "parallel" | "serial_per_path" | "serial_per_workspace";
  resultCapTokens: number;
}

type ToolClass =
  | "read" | "write" | "execute" | "network" | "mcp" | "write_outside_workspace";
```

`class` is what the [policy engine](../03-security/03-permission-and-policy-engine.md) and [approval UX](../05-ui/05-approval-and-trust-ux.md) key on. `risk` only affects presentation and default grant scope. The two are never conflated.

## Registry rules

| # | Rule |
|---|---|
| R1 | Names are permanent. A behavior change means a new `version`; an incompatible change means a new name |
| R2 | The registry is filtered by mode **and** by effective policy; a tool the user cannot run is never advertised to the model |
| R3 | Schemas are the single source of truth; the Rust type, the TS type, and the model-facing schema are generated from one definition |
| R4 | Every tool validates its own input again at execution time; upstream validation is never trusted |
| R5 | Path arguments are normalized and re-checked immediately before the syscall, not only at validation ([TOCTOU](../03-security/02-sandbox-model.md)) |
| R6 | Results are structured, truncated deterministically, and never contain secrets |
| R7 | Every mutation is checkpointed before it happens ([checkpoints](12-checkpoint-and-rollback-spec.md)) |
| R8 | Failures are returned to the model as data; a tool failure never crashes the run |

## Path handling (applies to every path argument)

Normalization order, before any check:

```
1. reject NUL bytes, control characters, and empty strings
2. expand a leading "~" only when it resolves inside the workspace
3. resolve relative paths against the workspace root (never the process cwd)
4. lexically normalize "." and ".." without touching the filesystem
5. canonicalize with symlink resolution (parent-canonicalize when creating)
6. verify the canonical result is inside the workspace boundary
7. verify it is not in the always-deny list
8. on Windows: reject reserved device names (CON, PRN, AUX, NUL, COM1-9, LPT1-9),
   trailing dots/spaces, and alternate data streams
9. reject non-regular files (devices, FIFOs, sockets) for read and write
```

Always-deny list (denied even inside the workspace, and never listed):

```
**/.git/{config,hooks/**,index}    **/.ssh/**            **/.env*
**/.aws/**  **/.kube/**  **/.docker/config.json          **/*.pem  **/*.key
**/id_rsa*  **/id_ed25519*  **/.npmrc  **/.pypirc  **/.netrc
**/.eury/keys/**                   the Eury Agent data directory itself
```

Writes to `.gitignore`, `.euryignore`, `EURY.md`, and CI workflow files are allowed but always escalate to `elevated` risk with an explicit warning, because they change the rules the agent itself operates under.

## Filesystem tools

| Name | Class | Risk | Modes | Timeout | Concurrency |
|---|---|---|---|---|---|
| `read_file` | read | low | all but chat | 10 s | parallel |
| `list_dir` | read | low | all but chat | 10 s | parallel |
| `glob` | read | low | all but chat | 15 s | parallel |
| `grep` | read | low | all but chat | 30 s | parallel |
| `write_file` | write | medium | agent, build | 30 s | serial_per_path |
| `edit_file` | write | medium | agent, build | 30 s | serial_per_path |
| `multi_edit` | write | elevated | agent, build | 60 s | serial_per_path |
| `delete_file` | write | elevated | agent, build | 30 s | serial_per_path |
| `move_file` | write | elevated | agent, build | 30 s | serial_per_path |
| `mkdir` | write | low | agent, build | 10 s | serial_per_path |

### `read_file`

```json
{
  "path": "string, required, workspace-relative",
  "offset": "integer >= 1, optional, 1-based start line",
  "limit": "integer 1..5000, optional, default 2000"
}
```

| Rule | Value |
|---|---|
| Max file size | 10 MB; larger returns `EURY_TOOL_READ_TOO_LARGE` with size and a paging hint |
| Binary detection | NUL byte in the first 8 KB, or invalid UTF-8 → metadata only, never bytes |
| Encoding | UTF-8, UTF-16 with BOM, and Latin-1 fallback; the detected encoding is reported |
| Output | Line-numbered (`  12→content`) so the model can cite ranges precisely |
| Long lines | Truncated at 2000 chars per line with an explicit marker |
| Missing file | `EURY_TOOL_NOT_FOUND`, plus up to 3 near-miss suggestions from the index |
| Result cap | 25 000 tokens |

### `write_file`

```json
{ "path": "string, required", "content": "string, required, max 5 MB",
  "createDirs": "boolean, default false", "expectedHash": "string, optional sha256" }
```

| Rule | Detail |
|---|---|
| Atomicity | Write to a temp file in the same directory, `fsync`, then atomic rename |
| Overwrite | Requires `expectedHash` when the file changed since the agent last read it, otherwise `EURY_TOOL_STALE_WRITE` |
| Blind create | Writing a file the agent never read is allowed but escalates risk to `elevated` |
| Preservation | Existing file mode, EOL style, trailing-newline presence, and BOM are preserved |
| Approval payload | A full diff against the current content, never just the path |
| Result | `{ action: "created" \| "updated" \| "unchanged", linesAdded, linesRemoved, bytes }` |

### `edit_file`

```json
{ "path": "string, required", "oldString": "string, required",
  "newString": "string, required", "replaceAll": "boolean, default false" }
```

| Rule | Detail |
|---|---|
| Uniqueness | `oldString` must match exactly once unless `replaceAll`; zero matches → `EURY_TOOL_EDIT_NO_MATCH`, multiple → `EURY_TOOL_EDIT_AMBIGUOUS` with the match count and line numbers |
| Whitespace | Matching is byte-exact; no fuzzy or trimmed fallback (silent mis-edits are worse than a retry) |
| No-op | `oldString == newString` is rejected at validation |
| Atomicity | Same temp-file-and-rename path as `write_file` |

### `multi_edit`

Applies an ordered list of edits to one file as a single transaction. Every edit must apply, or none do. Later edits see the results of earlier ones. One checkpoint and one approval cover the whole batch.

### `delete_file` and `move_file`

`delete_file` moves the file into the checkpoint store rather than unlinking it, so revert is always possible. Directory deletion requires `recursive: true`, is capped at 1000 entries, and is always `critical` risk. `move_file` refuses to overwrite an existing destination unless `overwrite: true`.

### `glob` and `grep`

| Rule | Detail |
|---|---|
| Ignores | `.gitignore` + `.euryignore` + defaults, unless `includeIgnored: true` (which escalates risk) |
| `glob` caps | 1000 results, sorted by modification time descending |
| `grep` engine | Rust regex crate — linear time, no catastrophic backtracking possible |
| `grep` caps | 100 files, 50 matches per file, 500 total; results include `path:line:text` |
| `grep` modes | `content`, `files_with_matches`, `count`; context lines 0–20 |
| Timeout | Partial results are returned with `truncated: true`, never an empty error |

## Shell tools

| Name | Class | Risk | Modes | Timeout | Concurrency |
|---|---|---|---|---|---|
| `run_command` | execute | elevated | agent, build | 300 s | serial_per_workspace |

```json
{ "command": "string, required, max 8192 chars",
  "cwd": "string, optional, must be inside the workspace",
  "timeoutMs": "integer 1000..600000, optional",
  "background": "boolean, default false",
  "explanation": "string, required — why this command, shown in the approval card" }
```

| Rule | Detail |
|---|---|
| No shell interpolation by default | The command is parsed into argv; `&&`, `\|\|`, `;`, `\|`, backticks, `$()`, and redirection require the `shell: true` flag, which is always `critical` risk |
| Command guard | Deny-listed patterns are refused before any approval prompt is shown ([sandbox model](../03-security/02-sandbox-model.md)) |
| Environment | Allowlisted variables only; no provider keys, no tokens, no `EURY_*` secrets |
| `PATH` | Pinned to system defaults plus workspace-local tool directories |
| Output | Streamed to the UI; capped at 2 MB total and 30 000 tokens for the model, head-and-tail truncated with a middle marker |
| Exit code | Always reported; non-zero is data for the model, not a run failure |
| Timeout | Process group terminated (`SIGTERM`, then `SIGKILL` after 2 s); partial output is retained |
| Background | Returns a handle; output is readable via `read_command_output`; killed on run end |
| Interactive detection | Commands that block on stdin are detected by idle-with-no-output and terminated with a clear hint |
| Never allowed | Commands modifying `~`, package-manager global installs, `sudo`, `curl \| sh`, or anything writing outside the workspace without `write_outside_workspace` |

## Network tools

| Name | Class | Risk | Modes | Timeout |
|---|---|---|---|---|
| `web_search` | network | low | ask, plan, agent, build | 20 s |
| `web_fetch` | network | medium | ask, plan, agent, build | 30 s |

| Rule | Detail |
|---|---|
| Routing | Through the managed gateway when signed in, else the user's own configured provider |
| SSRF guard | Deny `localhost`, `127.0.0.0/8`, `::1`, RFC1918, link-local, `169.254.169.254`, and any non-`http(s)` scheme; re-checked after every redirect |
| Redirects | Max 3, each re-validated |
| Response cap | 2 MB downloaded, converted to markdown, capped at 20 000 tokens |
| Trust | Results are **always** `untrusted` and delimited as data |
| Policy | Domain allow/deny lists apply; a blocked domain returns `EURY_POLICY_NETWORK_DENIED` |
| Air-gapped | Both tools are unregistered entirely |

Interactive browser navigation, login, form filling, and DOM clicking are deliberately out of scope for v1. `web_fetch` retrieves public HTTP(S) content only; it never inherits browser cookies or a user session.

## Visual media tools

Visual input and image generation are first-class capabilities, not text pasted into a chat. The model receives an image only through a structured reference with declared media type, dimensions, and byte size. Image bytes are never embedded in tool-call JSON or written to the transcript.

| Name | Class | Risk | Modes | Timeout | Purpose |
|---|---|---|---|---|---|
| `read_image` | read | low | ask, plan, agent, build | 20 s | Inspect a workspace or user-attached image with a vision-capable model |
| `generate_image` | network | medium | ask, plan, agent, build | 90 s | Generate one or more images through an enabled image provider |

### `read_image`

```json
{ "source": "attachment-id or workspace-relative path, required",
  "detail": "low | high, default low",
  "question": "string, optional, max 2000 chars" }
```

| Rule | Detail |
|---|---|
| Accepted formats | PNG, JPEG, WebP, GIF (first frame only), and AVIF |
| Limits | 20 MB decoded input, 8 192 px maximum on either side, 40 megapixels maximum; larger images are downscaled locally and reported as such |
| Locality | Workspace paths use the ordinary path guard; pasted/dropped images enter the encrypted attachment store first and have no path privileges |
| Privacy | EXIF, ICC, and other metadata are stripped before a provider request unless the user explicitly exports the original |
| Provider capability | The registry omits this tool when the selected model has no vision capability; the UI explains why rather than silently OCRing |
| Result | Structured visual observations and optional OCR spans, each tagged with the source and coordinate region when available |

### `generate_image`

```json
{ "prompt": "string, required, max 4000 chars",
  "aspectRatio": "1:1 | 4:3 | 3:4 | 16:9 | 9:16, default 1:1",
  "count": "integer 1..4, default 1",
  "referenceAttachments": "attachment-id[], optional, max 4" }
```

| Rule | Detail |
|---|---|
| Cost and consent | The first generation for a run shows the provider, estimated credit cost, and count. An organization policy may require approval for every generation. |
| Safety | Provider safety settings and organization policy apply before the request. Rejected prompts return a structured policy result, never a fabricated image. |
| Result handling | Results enter the encrypted attachment store with provider, prompt hash, dimensions, and content hash. They are displayed as a gallery and are not written to the workspace automatically. |
| Saving | Saving a generated image is a separate, explicit `write_file` operation with its normal path, diff, and approval rules. |
| Trust | Generated images and vision-derived text are untrusted content for prompt-injection purposes and cannot trigger tools or follow-on instructions by themselves. |
| Offline | The tool is unregistered in air-gapped mode and when no image provider is configured. |

## Git tools

| Name | Class | Risk | Modes | Notes |
|---|---|---|---|---|
| `git_status` | read | low | all but chat | Porcelain v2, parsed |
| `git_diff` | read | low | all but chat | Staged, unstaged, or ref range |
| `git_log` | read | low | all but chat | Max 100 commits |
| `git_branch` | write | medium | agent, build | Create and switch only |
| `git_commit` | execute | elevated | agent, build | Never `--amend`, never `--no-verify` |
| `git_stash` | execute | medium | agent, build | Used by checkpoints |

Forbidden at the tool layer, not merely discouraged: `push`, `reset --hard`, `clean -fdx`, `rebase`, `filter-branch`, force operations, and any `git config` write. Those remain human actions.

## Runtime tools

| Name | Class | Risk | Modes | Purpose |
|---|---|---|---|---|
| `read_command_output` | read | low | agent, build | Poll a background command |
| `memory_store` | write | medium | agent, build | Propose a memory entry (always user-confirmed) — [memory](08-memory-spec.md) |
| `spawn_subagent` | execute | elevated | agent, build | Delegate to a role — [multi-agent](13-multi-agent-spec.md) |

Task-list updates and plan persistence are application-owned state operations,
not model-facing tools. A model emits structured task or plan content; the
runtime validates it and the trusted state or plan store persists it.
Consequently `todo_write` and `plan_write` are not registered tools and cannot
smuggle a workspace write into Ask or Plan mode.

## MCP tools

Namespaced `mcp__<server>__<tool>`, matching `^[a-z0-9_]+$` after sanitization so every provider's function-name rules are satisfied.

| Rule | Detail |
|---|---|
| Class | `mcp`, unless the server is explicitly marked read-only in its approval record |
| Risk | `elevated` by default; a server cannot lower its own risk |
| Collisions | A server may never shadow a built-in name; conflicts are suffixed `_2` and logged |
| Schema | Rejected if it is not valid JSON Schema, exceeds 32 KB, or declares more than 64 tools |
| Trust | All results are `untrusted` |
| Details | [MCP integration](10-mcp-integration-spec.md) |

## Result format

```typescript
interface ToolResult {
  ok: boolean;
  content: string;                    // model-facing, already truncated
  truncated: boolean;
  trust: "trusted" | "semi_trusted" | "untrusted";
  error?: { code: string; message: string; retryable: boolean; hint?: string };
  metadata?: {
    path?: string; paths?: string[];
    action?: "created" | "updated" | "unchanged" | "deleted" | "moved";
    linesAdded?: number; linesRemoved?: number; bytes?: number;
    exitCode?: number; matchCount?: number; fileCount?: number;
    durationMs: number;
    checkpointId?: string;
    encoding?: string; eol?: "lf" | "crlf";
  };
  uiPayload?: object;                 // richer view for the UI only, never sent to the model
}
```

### Truncation rules

Deterministic so replays and golden tests are stable:

| Content type | Strategy |
|---|---|
| File read | Head-first, page hint appended |
| Command output | Head 60% + tail 40% with an omission marker showing the byte count |
| Search results | Highest-scoring first, then a total count |
| Structured JSON | Truncated at element boundaries, never mid-token |

The marker text is fixed: `[... N bytes omitted ...]`.

### Secret redaction

Every result passes a redactor before reaching the model, the UI, or the logs. It masks provider key patterns, bearer tokens, PEM blocks, connection strings with credentials, AWS key ids, and high-entropy assignments to names containing `secret`, `token`, `password`, or `key`. Redaction failures fail closed — the value is dropped, not passed through ([privacy](../03-security/07-privacy-and-data-residency.md)).

## Mode matrix

| Mode | read | write | execute | network | mcp | write_outside_workspace |
|---|---|---|---|---|---|---|
| `chat` | — | — | — | — | — | — |
| `ask` | yes | — | — | policy | read-only servers | — |
| `plan` | yes | — | — | policy | read-only servers | — |
| `agent` | yes | policy | policy | policy | policy | explicit grant only |
| `build` | yes | policy | policy | policy | policy | explicit grant only |

"policy" means the tool is registered but every call is subject to the effective policy and the deny-by-default grant flow. Mode restrictions are enforced in the registry, so a disallowed tool is not merely refused — the model never sees it.

An MCP server approved as read-only has its exposed tools classified as `read`;
therefore Ask and Plan do not receive the broader `mcp` class. Plan artifacts
are persisted only through the trusted application sink described above.

## Adding a new tool

1. Write the spec entry here, including class, risk, limits, and error codes.
2. Add the schema definition; the Rust and TypeScript types are generated.
3. Implement with input re-validation and pre-syscall path re-checks.
4. Add unit tests, a golden schema snapshot, and adversarial cases (traversal, symlink, huge input, hostile unicode).
5. Add a policy entry and a default risk mapping.
6. Add the activity-row and approval-card rendering.
7. Add an [eval](../08-quality/02-agent-eval-harness.md) task that requires the tool, and an injection task that tries to abuse it.

Skipping step 7 is not "done" ([definition of done](../08-quality/05-definition-of-done.md)).

## Conformance tests

| ID | Test |
|---|---|
| T1 | Every path argument rejects `../` escape, absolute escape, symlink escape, and Windows device names |
| T2 | Deny-listed paths are invisible to `list_dir`, `glob`, and `grep`, not merely unreadable |
| T3 | Binary files never return raw bytes to the model |
| T4 | Concurrent writes to one path serialize; no interleaved corruption |
| T5 | Every mutating tool produces a restorable checkpoint |
| T6 | `run_command` refuses metacharacters without `shell: true` |
| T7 | Every deny-listed command pattern is refused before approval is offered |
| T8 | No tool result contains a value from a seeded secret corpus |
| T9 | Truncation is byte-identical across runs and platforms |
| T10 | Every registered tool's advertised schema matches its implementation (property test on generated types) |
| T11 | `web_fetch` blocks the SSRF corpus, including post-redirect targets |
| T12 | Mode filtering: in `ask` and `plan`, no write or execute tool appears in the advertised list |
| T13 | Visual-media tools obey attachment validation, metadata stripping, policy, and explicit-save requirements |

## Related documents

- [Sandbox model](../03-security/02-sandbox-model.md)
- [Permission and policy engine](../03-security/03-permission-and-policy-engine.md)
- [Agent runtime](01-agent-runtime-spec.md)
- [MCP integration](10-mcp-integration-spec.md)
- [Multimodal and attachments](17-multimodal-and-attachment-spec.md)
- [Error taxonomy](15-error-taxonomy.md)
