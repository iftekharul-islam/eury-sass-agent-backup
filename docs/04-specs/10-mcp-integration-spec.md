# MCP Integration Specification

Spec-Version: 2.0.0

Model Context Protocol support lets users extend the agent with third-party tools. Every MCP server is **untrusted third-party code running on the user's machine**, and this spec is written from that premise.

## Threat framing

| Assumption | Consequence |
|---|---|
| A server can execute arbitrary code | Adding a server is as consequential as installing software, and the consent flow says so |
| A server can lie about its tools | Tool descriptions are untrusted text and are delimited as data in the prompt |
| A server can return malicious content | Every result is `untrusted` and cannot escalate privileges |
| A server can change its tools after approval | Tool sets are fingerprinted; a change requires re-approval |
| A server can exfiltrate anything it is given | Arguments are shown in full at approval time, and secrets are never auto-injected |

## Supported transports

| Transport | Support | Notes |
|---|---|---|
| `stdio` | Full | Local child process; the primary and recommended mode |
| `http` (streamable HTTP) | Full | Remote server; requires HTTPS and an explicit URL approval |
| `sse` | Legacy | Accepted for compatibility, HTTPS only |
| `websocket` | Not supported | No compelling use case; smaller surface is better |

Protocol version: MCP `2025-06-18`, with negotiation. A server requiring a newer major version is refused with a clear message rather than being spoken to incorrectly.

## Configuration

Three sources, merged with the organization winning:

| Source | Path | Precedence |
|---|---|---|
| Organization registry | Delivered with policy | Highest — may allowlist, pin, or forbid |
| User | `$EURY_AGENT_DATA_DIR/mcp.json` | Middle |
| Workspace | `<workspace>/.eury/mcp.json` | Lowest, and **never auto-enabled** |

```json
{
  "servers": {
    "github": {
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github@1.2.3"],
      "env": { "GITHUB_TOKEN": "${keychain:mcp/github/token}" },
      "cwd": "${workspace}",
      "enabled": true,
      "readOnly": false,
      "timeoutMs": 30000,
      "toolAllowlist": ["search_issues", "get_issue"],
      "toolDenylist": [],
      "trustFingerprint": "sha256:9f2c…"
    },
    "docs": {
      "transport": "http",
      "url": "https://mcp.example.com/v1",
      "headers": { "Authorization": "Bearer ${keychain:mcp/docs/token}" },
      "enabled": false
    }
  }
}
```

| Field rule | Detail |
|---|---|
| `command` | Must resolve to an executable on the pinned `PATH`; shell interpolation is not performed |
| `args` | Array only; a single string is rejected, so quoting bugs cannot become injection |
| Version pinning | Unpinned `@latest` specifiers produce a warning at approval time, since the approved code could silently change |
| `env` | Only `${keychain:...}` references and literal non-secret values. `${env:...}` is **forbidden**, so the app's own secrets can never be forwarded |
| `cwd` | Must be inside the workspace |
| `url` | HTTPS only; SSRF rules from the tool catalog apply |
| Workspace config | Treated as untrusted repo content: discovered, listed, and never started until the user approves it in the UI |

## Approval and trust

Adding or enabling a server requires an explicit dialog showing: the resolved executable path or URL, the full argv, the environment variable names (never values), the working directory, the config source, whether the package is version-pinned, and the complete list of tools with their descriptions and schemas after connecting.

```
User adds server
  -> config validated
  -> "This runs code on your computer" consent dialog  [Cancel | Approve]
  -> handshake in a probe session
  -> tool list fetched and fingerprinted
  -> tool list shown for review                        [Reject | Approve tools]
  -> trustFingerprint stored, server enabled
```

The fingerprint is `sha256` over the sorted list of tool names, descriptions, and input schemas. On every subsequent connect, a fingerprint mismatch disables the server and requires re-approval, with a diff of what changed. This is the control that prevents a benign server from silently gaining a `delete_everything` tool after an update.

## Lifecycle

| Phase | Behavior |
|---|---|
| Connect | Lazy: on the first run in a workspace where the server is enabled, or eagerly if the user opens the MCP settings page |
| Handshake | `initialize` with our capabilities, then `tools/list`; must complete within 10 s |
| Registration | Tools registered as `mcp__<server>__<tool>` after sanitization to `^[a-z0-9_]+$` |
| Health | `ping` every 30 s; three consecutive failures trigger one reconnect |
| Reconnect | Exponential backoff (1 s, 4 s, 16 s), max 3 attempts, then disabled for the session |
| Restart | Manual restart is always available and clears the backoff |
| Shutdown | Graceful `shutdown` request, then `SIGTERM`, then `SIGKILL` after 2 s; always on workspace close and app quit |
| Orphans | A PID file per server; leftover processes from a crashed app are reaped on next launch |

## Process isolation (stdio servers)

| Control | Implementation |
|---|---|
| Environment | Cleared, then rebuilt from the allowlist: `PATH`, `HOME`, `LANG`, `TMPDIR`, plus the declared `env` entries |
| Secrets | The Eury Agent keychain, provider keys, and session tokens are never present in the child environment |
| Working directory | The workspace, or a temp directory when unspecified |
| Resource caps | 512 MB memory, 25% CPU sustained, 30 s per request; exceeding these terminates the server |
| Process group | Its own group, so termination is complete and reliable |
| OS sandbox | Where the platform allows, the same sandbox profile as `run_command`, minus workspace write access unless the server declares it needs it |
| Output | `stderr` is captured to the server's log pane, never merged into tool results |
| Network | Subject to the policy's network rules, like any other tool |

## Tool registration

| Rule | Detail |
|---|---|
| Naming | `mcp__<server>__<tool>`, sanitized and truncated to 64 chars with a hash suffix on collision |
| Class | `mcp`; a server marked `readOnly: true` in its **approval record** (not in its own self-description) maps to `read` |
| Risk | `elevated` by default, `medium` for read-only servers; a server cannot declare its own risk |
| Caps | 64 tools per server, 256 total across all servers, 32 KB per schema |
| Validation | Schemas must be valid JSON Schema draft 2020-12; invalid ones are skipped with a warning and the rest of the server still works |
| Filtering | `toolAllowlist` and `toolDenylist` apply, then the organization registry, then mode restrictions |
| Collisions | Built-in names are never shadowed |
| Descriptions | Truncated to 1024 chars and wrapped as untrusted data in the prompt, so a description cannot smuggle instructions |

## Invocation

```
model calls mcp__github__create_issue
  -> policy check (class = mcp, plus org allow/deny)
  -> approval: card shows server, tool, full arguments, and the destination
  -> arguments validated against the cached schema
  -> tools/call with a 30 s timeout
  -> result: truncated to 20 000 tokens, secret-redacted, marked trust=untrusted
  -> audit event with server, tool, and an args hash (never raw args)
```

Approval scope for MCP tools maxes out at `session`. `always` is deliberately unavailable, because a permanent unattended grant to third-party code is not something we are willing to make one click away.

## Resources and prompts

| MCP capability | Support |
|---|---|
| `tools` | Full |
| `resources` | Read-only, user-initiated only. Never auto-included in context, because auto-inclusion is a silent injection channel |
| `prompts` | Listed as slash commands after approval; the template body is shown before first use |
| `sampling` | **Not supported.** A server asking us to call the model on its behalf would let third-party code spend the user's tokens |
| `roots` | Supported: the workspace root is advertised so servers do not need to guess paths |
| `logging` | Captured into the server log pane at `info` and above |

## Failure handling

| Failure | Behavior | Code |
|---|---|---|
| Executable not found | Server disabled, actionable install hint shown | `EURY_MCP_SPAWN_FAILED` |
| Handshake timeout | Disabled for the session after retries | `EURY_MCP_HANDSHAKE_FAILED` |
| Protocol version unsupported | Disabled with an explanation | `EURY_MCP_VERSION_UNSUPPORTED` |
| Fingerprint changed | Disabled pending re-approval, with a diff | `EURY_MCP_FINGERPRINT_CHANGED` |
| Crash mid-run | Pending calls return a structured error to the model; the run continues | `EURY_MCP_SERVER_CRASHED` |
| Request timeout | Tool error returned; the server stays connected | `EURY_MCP_TIMEOUT` |
| Invalid response | Tool error; three occurrences disable the server | `EURY_MCP_PROTOCOL_ERROR` |
| Resource cap exceeded | Server terminated and disabled, with the reason shown | `EURY_MCP_RESOURCE_EXCEEDED` |
| Policy denies the server | Never started; listed as "blocked by your organization" | `EURY_MCP_POLICY_DENIED` |

Failures are always visible in the UI. A silently missing tool leads users to blame the model for a configuration problem.

## Enterprise controls

| Policy field | Effect |
|---|---|
| `mcp.enabled: false` | MCP is entirely unavailable; no server starts |
| `mcp.allowedServers: [...]` | Only these servers may run, matched by name **and** command fingerprint |
| `mcp.blockedServers: [...]` | Explicit denials, evaluated after the allowlist |
| `mcp.allowWorkspaceConfig: false` | `.eury/mcp.json` is ignored entirely |
| `mcp.requireApproval: "always"` | Disables `session` scope; every call is approved individually |
| `mcp.allowedTransports: ["stdio"]` | Restricts remote servers |
| `mcp.registryUrl` | An organization-curated catalog shown in the UI, with pinned versions |

Admins manage this from the [admin console](../06-enterprise/06-admin-console-spec.md); the desktop treats the registry as read-only.

## Observability

Per server: connection state, uptime, restart count, tool call counts, error rate, p95 latency, memory and CPU, and the last 500 stderr lines. Audit events: `mcp.server.added`, `mcp.server.approved`, `mcp.server.enabled`, `mcp.server.disabled`, `mcp.server.fingerprint_changed`, `mcp.tool.called`, `mcp.tool.denied`, `mcp.server.crashed`.

## Conformance tests

| ID | Test |
|---|---|
| T1 | No server starts without explicit approval, including from workspace config |
| T2 | The child environment contains no Eury Agent secret, provider key, or session token |
| T3 | `${env:...}` in config is rejected at validation |
| T4 | A changed tool fingerprint disables the server and requires re-approval |
| T5 | A malicious tool description cannot cause a privileged action without approval |
| T6 | A server declaring itself read-only is still treated as `mcp` class unless the approval record says otherwise |
| T7 | Results are marked untrusted and pass through the redactor |
| T8 | Crash, hang, flood, and malformed-JSON servers are all contained; the run survives |
| T9 | Resource caps terminate a runaway server within 5 s |
| T10 | `sampling` requests are refused |
| T11 | Resources are never auto-included in a prompt |
| T12 | `mcp.enabled: false` results in zero processes and zero registered tools |
| T13 | App quit and crash leave no orphan server processes |
| T14 | Name collisions with built-in tools are impossible |

## Related documents

- [Tool catalog](02-tool-catalog-spec.md)
- [Sandbox model](../03-security/02-sandbox-model.md)
- [Prompt injection defense](../03-security/05-prompt-injection-defense.md)
- [Workspace policies](../06-enterprise/03-workspace-policies.md)
- [Error taxonomy](15-error-taxonomy.md)
