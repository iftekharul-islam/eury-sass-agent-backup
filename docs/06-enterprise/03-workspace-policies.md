# Workspace Policies

Spec-Version: 1.4.0

Distributed via `GET /agent/v1/policies/effective`, enforced locally in `agent-policy` so policy still applies offline.

## Policy document

```typescript
interface WorkspacePolicy {
  schemaVersion: 1;
  version: number;                     // monotonically increasing per org
  label?: string;                      // "Standard", "Regulated"
  policyMaxAgeHours: number;           // default 168

  tools: {
    allow?: string[];                  // allowlist; omit = all not denied
    deny?: string[];                   // always wins over allow
    requireApproval?: ToolClass[];     // ["execute","network","write_outside_workspace"]
    defaultDecision: Partial<Record<ToolClass, "allow" | "needsApproval" | "deny">>;
    maxGrantScope?: Record<ToolClass, GrantScope>;  // caps "Always" grants
  };

  models: {
    allow?: string[];                  // "openai/gpt-5", "anthropic/claude-*"
    deny?: string[];
    requireManagedGateway?: boolean;    // forbid BYOK
    allowByok?: boolean;                // default true
  };

  filesystem: {
    denyGlobs: string[];                // [".env*", "**/*.pem", "~/.ssh/**"]
    allowOutsideWorkspace: boolean;     // default false
    maxFileWriteBytes?: number;         // default 10_485_760
    redactSecretsInContext: boolean;    // default true
  };

  commands: {
    denyPatterns: string[];             // regex, e.g. "\\brm\\s+-rf\\b", "\\bsudo\\b"
    allowPatterns?: string[];           // if set, argv[0] must match one
    maxRuntimeSeconds: number;          // default 300
    networkDuringExecute: boolean;      // default false; true only permits separate per-operation approval
  };

  network: {
    blockWebFetch: boolean;
    allowImageGeneration?: boolean;    // default false for Enterprise
    allowedHosts?: string[];            // exact or "*.internal.acme.com"
    blockedHosts?: string[];
  };

  mcp: {
    enabled: boolean;
    allowedServers?: string[];          // registry ids or signed manifest hashes
    blockedServers?: string[];
    allowLocalServers: boolean;         // default false for Enterprise
    allowWorkspaceConfig: boolean;
    requireSignedManifest: boolean;
    requireApproval: "once" | "run" | "session";
    allowedTransports: ("stdio" | "https")[];
    registryUrl?: string;
  };

  cost: {
    maxCostPerRunUsdMicros?: number;
    maxCostPerDayUsdMicros?: number;
    maxTokensPerRun?: number;
  };

  data: {
    auditUploadRequired: boolean;
    auditIncludePaths: boolean;         // paths, never contents
    auditPayloadMode: "metadata_only" | "full_payload";
    allowCloudSync: boolean;            // conversation sync
    allowScreenshots: boolean;
    residency?: "us" | "eu" | "in";
  };

  telemetry: {
    mode: "off" | "user_choice" | "required";
    crashReports: boolean;
    endpoint?: string;                  // HTTPS only
  };

  agent: {
    allowedModes: Mode[];               // subset of chat|agent|plan|ask|build
    allowSubAgents: boolean;
    maxParallelTools: number;           // default 4
    maxTurnsPerRun: number;             // default 50
    requirePlanBeforeWrite: boolean;    // "plan-gated" orgs
  };

  update: {
    minVersion?: string;
    channel?: "stable" | "beta";
    autoUpdate: boolean;
  };
}
```

The normative machine-readable definition is
[`workspace-policy.schema.json`](../04-specs/schemas/workspace-policy.schema.json).
Examples and consumers MUST validate against it; prose does not introduce
aliases or additional fields.

`ToolClass` = `read | write | execute | network | mcp | write_outside_workspace`.
`GrantScope` = `once | run | session | always`.

## Sources and merge order

```
1. Product defaults (shipped in the binary)
2. Organization policy      (AgentPolicy scope="org", isActive)
3. Team policy              (scope="team", user's teams — most restrictive)
4. User caps                (scope="user"; a user may only restrict further)
5. Workspace file           (<workspace>/.eury/policy.json — may only restrict)
6. Local settings           (may only restrict)
```

Merge rules, exactly:

| Field kind | Rule |
|---|---|
| `deny*` lists | Union |
| `allow*` lists | Intersection (an absent list means "unconstrained" and does not narrow) |
| Booleans | Restrictive value wins (`false` for allow-flags, `true` for require-flags) |
| Numeric limits | `min()` |
| Enum sets (`allowedModes`) | Intersection |
| `residency` | Org value is authoritative; lower levels cannot change it |
| `maxGrantScope` | Narrowest scope wins |
| `defaultDecision` | Most restrictive wins: `deny` > `needsApproval` > `allow` |

**Nothing below level 2 can widen anything.** A workspace file that tries to add a tool to `allow` or raise a limit is rejected and logged as `policy.workspace_override_rejected`. This is what makes it safe to open an untrusted repository that ships its own `.eury/policy.json`.

## Presets

| Preset | Character |
|---|---|
| `Permissive` | Product/org profile allows read/write in a trusted workspace; execute needs approval once per session; BYOK allowed; sync on |
| `Standard` (default) | Read allowed; write/execute/network default to `needsApproval`; no writes outside workspace |
| `Strict` | Execute allowlist only; no network tools; no local MCP; per-run cost cap; audit upload required |
| `Regulated` | Strict + managed gateway required, plan-gated writes, no sync, no screenshots, residency pinned, paths in audit |

Presets instantiate the product or organization source; they are not lower-level
overrides. The stored document is always fully expanded so behavior never
changes when a preset definition is edited later.

## Reference organization policies

| Organization profile | Required policy posture |
|---|---|
| Regulated engineering | `Regulated` preset; managed gateway only; residency pinned; no BYOK, sync, screenshots, web fetch, or image generation; audit upload and plan-gated writes required |
| Private-code company | Approved model allowlist; source code classified Internal; web fetch limited to approved hosts; image generation disabled; audit upload required |
| Air-gapped deployment | No managed gateway, web fetch, image generation, cloud sync, telemetry export, or remote MCP; signed offline model/catalog bundle only |
| Developer sandbox | Standard preset; BYOK allowed; bounded cost cap; web fetch and local MCP require per-run approval; no production credentials in workspace |
| Design-enabled product team | Standard preset plus approved image provider/model; image generation requires per-run approval and explicit workspace save; generated asset retention follows conversation retention |

Policy examples are reviewed by Security before an organization can use them as a starting preset. They are examples of a full expanded document, not a substitute for the merge rules above.

## Distribution and caching

| Concern | Rule |
|---|---|
| Endpoint | `GET /agent/v1/policies/effective` with `If-None-Match` |
| Refresh | On login, every 15 min, on workspace open, on manual refresh |
| Cache | Stored in SQLite with `etag`, `version`, `fetchedAt` |
| Offline | Cached policy applies indefinitely for enforcement |
| Stale limit | If `auditUploadRequired` and the cache is older than `policyMaxAgeHours` (default 168), write/execute tools fail closed |
| Downgrade protection | A policy with a lower `version` than the cached one is rejected (`EURY_POLICY_STALE`) |
| Signature | Enterprise policies are signed; the desktop verifies before applying |
| Hot apply | New policy applies at the next tool-call boundary, never mid-tool; standing grants that violate it are revoked immediately |

## Enforcement points

| Layer | Enforces |
|---|---|
| `agent-policy` (pre-flight) | Tool allow/deny, approval requirement, grant scope caps, mode restrictions |
| `agent-sandbox` | Path globs, workspace boundary, command patterns, egress during execute |
| Cost guard hook | Per-run and per-day cost/token caps |
| Cloud gateway | Model allowlist, managed-gateway requirement, org quota |
| Sync/telemetry client | `data.allowCloudSync`, `telemetry.mode`, `telemetry.crashReports`, residency routing |

Two-sided enforcement is deliberate: the desktop enforces for offline correctness and speed, the cloud enforces because a modified client must not be able to escape org rules.

## Admin experience

Web: `/admin/agent/policies` — preset picker, form editor for common fields, JSON editor with schema validation, diff against the active version, dry-run ("which of the last 100 runs would this have blocked?"), activate, and version history with rollback.

Desktop: Settings → Policy shows the effective policy read-only, annotated with the source level for each field, plus a "Request exception" action.

## Exceptions

A denied operation can generate an exception request (`policy.exception_requested`) containing tool, argument hash, justification, run link, and policy version. An admin with `agent:approve_exceptions` grants a **scoped, expiring** exception (single tool shape, single user, max 7 days) — never a permanent policy edit by accident.

## Validation and testing

| Check | Where |
|---|---|
| JSON schema validation | Server on upsert, client on load |
| Merge property tests | "Merge never widens" as a property, fuzzed over random policy pairs |
| Golden merges | Fixture set of org/team/user/workspace combinations with expected output |
| Enforcement matrix | Every `ToolClass` × preset asserted in integration tests |
| Downgrade/stale | Tests for version rollback and expiry fail-closed |

## Delivery

The local policy engine, merge semantics, grants, and the four presets ship in Phase 7. Cloud distribution, signing, version monotonicity, and the admin editor ship in Phase 24. Dry-run analysis ships in Phase 25.

## Related documents

- [Permission and policy engine](../03-security/03-permission-and-policy-engine.md)
- [Sandbox model](../03-security/02-sandbox-model.md)
- [Approval and trust UX](../05-ui/05-approval-and-trust-ux.md)
- [Admin console](06-admin-console-spec.md)
