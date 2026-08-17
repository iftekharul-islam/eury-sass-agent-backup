# Cloud API Contract

Spec-Version: 1.5.0

Base URL: `{EURY_AGENT_CLOUD_URL}/agent/v1`

**Every** Agent endpoint is under `/agent/v1`. Nothing is added to `/auth`, `/code`, `/eury`, or `/admin` ([backend module structure](16-backend-module-structure.md)).

Auth header: `Authorization: Bearer <access_token>` unless marked **public**.

Common headers on all responses:

| Header | Meaning |
|---|---|
| `X-Agent-Request-Id` | Correlation id, echoed in logs and audit |
| `X-Agent-Api-Version` | `1` |
| `Retry-After` | Present on `429` and `503` |

## Endpoint index

| Method | Path | Auth |
|---|---|---|
| GET | `/health` | public |
| GET | `/releases/latest` | public |
| POST | `/auth/device/start` | public |
| POST | `/auth/device/poll` | public + PKCE |
| POST | `/auth/device/exchange` | web session |
| POST | `/auth/refresh` | refresh token |
| POST | `/auth/logout` | bearer |
| GET | `/me` | bearer |
| POST | `/devices/enroll` | bearer |
| DELETE | `/devices/:id` | bearer |
| GET | `/models` | bearer |
| POST | `/chat/stream` | bearer |
| POST | `/tools/web-search` | bearer |
| GET | `/policies/effective` | bearer |
| POST | `/audit/batch` | bearer |
| GET | `/usage/current` | bearer |
| POST | `/sync/conversations` | bearer |
| GET/POST | `/admin/releases` | admin |
| POST | `/admin/releases/:id/activate` | admin |
| POST | `/admin/releases/:id/rollback` | admin |
| GET/PUT | `/admin/policies` | admin |
| POST | `/admin/policies/:id/activate` | admin |
| GET | `/admin/audit` | admin |
| GET | `/admin/devices` | admin |
| * | `/scim/v2/*` | SCIM token |

---

## Public

### GET /health

```json
{ "ok": true, "version": "1.0.0", "time": "2026-08-16T03:55:00Z" }
```

### GET /releases/latest

Update manifest. Optional query: `?platform=darwin&arch=aarch64&channel=stable`.

```json
{
  "version": "1.0.0",
  "minSupported": "1.0.0",
  "channel": "stable",
  "publishedAt": "2026-08-16T00:00:00Z",
  "releaseNotes": "markdown",
  "mandatory": false,
  "downloads": {
    "darwin-aarch64": "https://cdn.example.com/agent-releases/1.0.0/Eury-Agent-1.0.0-aarch64.dmg",
    "darwin-x86_64":  "https://.../Eury-Agent-1.0.0-x64.dmg",
    "win32-x86_64":   "https://.../Eury-Agent-1.0.0-x64.msi",
    "linux-x86_64":   "https://.../Eury-Agent-1.0.0-x86_64.AppImage"
  },
  "sha256": {
    "darwin-aarch64": "e3b0c442...",
    "win32-x86_64": "…",
    "linux-x86_64": "…"
  },
  "signature": "base64-minisign-of-manifest"
}
```

The desktop verifies `sha256` **and** the manifest signature before applying an update ([auto-update](../07-ops/04-auto-update-and-rollback.md)).

---

## Auth

### POST /auth/device/start — public

```json
{ "code_challenge": "base64url", "code_challenge_method": "S256",
  "device_name": "Manna's MacBook", "platform": "darwin", "app_version": "1.0.0" }
```

```json
{ "device_code": "opaque-32b", "user_code": "BQDX-7T2M",
  "verification_uri": "https://eury.app/en/agent/authorize",
  "verification_uri_complete": "https://eury.app/en/agent/authorize?code=BQDX-7T2M",
  "expires_in": 600, "interval": 2 }
```

Errors: `400 EURY_AUTH_PKCE_INVALID` (missing/plain challenge), `429 EURY_RATE_LIMITED`.

### POST /auth/device/exchange — web session

Called by the browser page after the user approves.

```json
{ "user_code": "BQDX-7T2M", "approve": true }
```

```json
{ "ok": true, "device_name": "Manna's MacBook" }
```

Errors: `401 EURY_AUTH_UNAUTHORIZED`, `404 EURY_AUTH_CODE_NOT_FOUND`, `410 EURY_AUTH_CODE_EXPIRED`.

### POST /auth/device/poll — public

```json
{ "device_code": "opaque-32b", "code_verifier": "…" }
```

Pending: `202 { "status": "pending" }`. Approved:

```json
{
  "status": "approved",
  "access_token": "jwt",
  "refresh_token": "opaque-32b",
  "expires_in": 900,
  "device_id": "cuid",
  "user": { "id": "cuid", "email": "…", "name": "…", "plan": "pro",
            "organizationId": "cuid|null", "role": "member|admin|owner|null" }
}
```

Errors: `400 EURY_AUTH_PKCE_MISMATCH`, `403 EURY_AUTH_DENIED`, `410 EURY_AUTH_CODE_EXPIRED`, `429 EURY_AUTH_SLOW_DOWN`.

### POST /auth/refresh — refresh token

```json
{ "refresh_token": "opaque-32b", "device_id": "cuid" }
```

```json
{ "access_token": "jwt", "refresh_token": "new-opaque-32b", "expires_in": 900 }
```

Rotate-on-use. Presenting an already-rotated token returns `401 EURY_AUTH_REFRESH_REUSED` and revokes the entire token chain for that device.

### POST /auth/logout — bearer

```json
{ "device_id": "cuid", "all_devices": false }
```

Response `204`. Revokes refresh chain(s); access tokens expire naturally within 15 min.

### GET /me — bearer

```json
{
  "user": { "id": "cuid", "email": "…", "name": "…", "avatarUrl": null },
  "plan": { "code": "pro", "managedGateway": true, "byokAllowed": true },
  "entitlements": {
    "version": 12,
    "effectiveAt": "2026-08-16T00:00:00Z",
    "keys": ["agent.desktop", "modes.ask", "modes.plan", "modes.agent",
             "modes.build", "managed.inference", "byok"]
  },
  "organization": { "id": "cuid", "name": "Acme", "role": "member" },
  "limits": { "dailyManagedRuns": 1000, "monthlyManagedTokens": 5000000, "concurrentRuns": 3 },
  "policyVersion": 7
}
```

Entitlement keys and package semantics are defined in
[pricing and packaging](../01-product/04-pricing-and-packaging.md). The client
checks both an entitlement and the relevant runtime capability/policy; a key
never overrides policy.

---

## Devices

### POST /devices/enroll — bearer

```json
{ "device_id": "cuid", "public_key": "base64-ed25519", "platform": "darwin",
  "app_version": "1.0.0", "os_version": "15.5" }
```

```json
{ "ok": true, "audit_required": true }
```

The public key verifies audit batch signatures. Re-enrolling with a new key supersedes the old one and is itself an audit event.

### DELETE /devices/:id — bearer

`204`. Owner or org admin only. Revokes tokens and marks the device `revokedAt`.

---

## Models and gateway

### GET /models — bearer

```json
{
  "models": [
    { "provider": "openai", "id": "gpt-5", "label": "GPT-5",
      "contextWindow": 400000, "maxOutputTokens": 128000,
      "supportsTools": true, "supportsVision": true, "supportsImageGeneration": true,
      "maxImageBytes": 20971520, "maxImageCount": 4,
      "tier": "pro", "allowed": true, "disabledReason": null }
  ],
  "default": { "provider": "openai", "id": "gpt-5" },
  "etag": "m-14"
}
```

`allowed` reflects the caller's plan **and** org policy model allowlist. Served by the Agent module's own catalog — it does not call the legacy code/eury model service.

### POST /chat/stream — bearer

Managed gateway. Content-Type of response: `application/x-ndjson`.

```typescript
interface AgentChatStreamRequest {
  runId: string;                  // client-generated, used for abort + audit
  provider: string;
  model: string;
  mode: "chat" | "agent" | "plan" | "ask" | "build";
  messages: { role: "system" | "user" | "assistant" | "tool";
              content: MessagePart[]; name?: string; toolCallId?: string }[];
  tools?: ToolSchema[];           // JSON-schema tool declarations
  maxOutputTokens?: number;       // default 20000, capped by plan
  temperature?: number;           // default 0.2
  compressionRatio?: number;      // default 0.78
  workspace?: { name?: string; summary?: string; languages?: string[] };
  projectId?: string;
  mock?: boolean;                 // test-only, rejected in production
}

type MessagePart =
  | { type: "text"; text: string }
  | { type: "image"; attachmentId: string; mediaType: "image/png" | "image/jpeg" | "image/webp" | "image/gif" | "image/avif";
      sha256: string; width: number; height: number };
```

The desktop uploads or resolves `attachmentId` through the Agent module's encrypted attachment service; raw image data is never included in the stream request. Image parts are rejected when the model does not advertise `supportsVision`.

Stream events, one JSON object per line:

```typescript
{ type: "meta", runId, provider, model, requestId, upstreamLatencyMs }
{ type: "delta", text: string }
{ type: "reasoning", text: string }          // when provider exposes it
{ type: "tool_call", id, name, argumentsDelta? , arguments? }
{ type: "citations", sources: { title, url }[] }
{ type: "usage", promptTokens, completionTokens, costUsdMicros }
{ type: "done", finishReason: "stop"|"length"|"tool_calls"|"aborted" }
{ type: "error", code: "EURY_…", message: string, retryable: boolean }
```

| Rule | Value |
|---|---|
| First byte target | ≤ 400 ms p95 added over upstream |
| Abort | Client disconnect aborts upstream within 250 ms |
| Max request body | 1 MB |
| Quota exceeded | `429 EURY_QUOTA_EXCEEDED` before any upstream call |
| Model not allowed | `403 EURY_POLICY_MODEL_DENIED` |

### POST /tools/web-search — bearer

```json
{ "query": "tauri 2 ipc channel backpressure", "maxResults": 5,
  "includeDomains": [], "runId": "…" }
```

```json
{ "results": [ { "title": "…", "url": "…", "snippet": "…", "score": 0.82 } ],
  "provider": "tavily", "cached": false }
```

Agent-owned Tavily client. Denied with `403 EURY_POLICY_NETWORK_DENIED` when org policy blocks network tools.

---

## Policy, audit, usage

### GET /policies/effective — bearer

Send `If-None-Match`; `304` when unchanged.

```json
{
  "version": 7,
  "etag": "p-7-org-cuid",
  "computedAt": "2026-08-16T03:00:00Z",
  "sources": ["org:acme", "team:platform", "user:caps"],
  "policy": { /* WorkspacePolicy — see workspace-policies doc */ }
}
```

### POST /audit/batch — bearer

```json
{
  "device_id": "cuid",
  "batch_id": "uuid",
  "events": [ { "id": "uuid", "runId": "…", "eventType": "tool_end",
                "occurredAt": "…", "payload": { } } ],
  "signature": "base64-ed25519-over-canonical-json"
}
```

```json
{ "accepted": 42, "rejected": 0, "duplicates": 3, "nextBackoffMs": 0 }
```

Idempotent on `events[].id`. Max 500 events / 2 MB per batch. Bad signature → `401 EURY_AUDIT_SIGNATURE_INVALID`.

### GET /usage/current — bearer

```json
{
  "limits": {
    "dailyManagedRuns": {
      "used": 37, "limit": 1000, "period": "2026-08-16",
      "resetsAt": "2026-08-17T00:00:00Z"
    },
    "monthlyManagedTokens": {
      "used": 812344, "limit": 5000000, "period": "2026-08",
      "resetsAt": "2026-09-01T00:00:00Z"
    },
    "monthlyBudgetUsdMicros": {
      "used": 12400000, "limit": 200000000, "period": "2026-08",
      "resetsAt": "2026-09-01T00:00:00Z"
    }
  },
  "concurrentRuns": { "used": 1, "limit": 3 },
  "throttled": false
}
```

### POST /sync/conversations — bearer

Optional, off by default. Last-write-wins per message id with vector clock on conversation.

```json
{ "since": "cursor-token",
  "upserts": [ { "conversationId": "…", "updatedAt": "…", "messages": [ ] } ],
  "deletes": ["conversationId"] }
```

```json
{ "cursor": "next-cursor", "applied": 12, "conflicts": [] }
```

---

## Admin

All under `/agent/v1/admin/*`, `AgentAdminGuard`.

| Method | Path | Body / Result |
|---|---|---|
| GET | `/admin/releases` | list with channel, active flag, download counts |
| POST | `/admin/releases` | `{ version, channel, notes, artifacts[], sha256{}, minSupported, mandatory }` |
| POST | `/admin/releases/:id/activate` | promote to active for its channel |
| POST | `/admin/releases/:id/rollback` | reactivate previous active release |
| GET | `/admin/policies` | list org policies with versions |
| PUT | `/admin/policies/:id` | upsert `rulesJson`, bumps `version` |
| POST | `/admin/policies/:id/activate` | make active; invalidates policy ETags |
| GET | `/admin/audit` | filter by `orgId`, `userId`, `runId`, `eventType`, date range; `format=json\|csv` |
| GET | `/admin/devices` | org device inventory |

## SCIM

`/agent/v1/scim/v2/Users`, `/agent/v1/scim/v2/Groups` — RFC 7644 subset ([identity, SSO, SCIM](../06-enterprise/01-identity-sso-scim.md)). Returns `404` when `AGENT_SCIM_TOKEN` is unset so the surface does not exist for non-enterprise deployments.

## Errors

Uniform body; `code` values from [error taxonomy](15-error-taxonomy.md).

```json
{
  "error": {
    "code": "EURY_POLICY_MODEL_DENIED",
    "message": "Model gpt-5 is not permitted by organization policy.",
    "requestId": "req_01J…",
    "retryable": false,
    "details": { "model": "gpt-5", "policyVersion": 7 }
  }
}
```

| HTTP | Typical codes |
|---|---|
| 400 | `EURY_REQUEST_INVALID`, `EURY_AUTH_PKCE_INVALID` |
| 401 | `EURY_AUTH_UNAUTHORIZED`, `EURY_AUTH_REFRESH_REUSED`, `EURY_AUDIT_SIGNATURE_INVALID` |
| 403 | `EURY_POLICY_*`, `EURY_AUTH_DENIED` |
| 404 | `EURY_NOT_FOUND` |
| 409 | `EURY_SYNC_CONFLICT` |
| 410 | `EURY_AUTH_CODE_EXPIRED` |
| 429 | `EURY_RATE_LIMITED`, `EURY_QUOTA_EXCEEDED`, `EURY_AUTH_SLOW_DOWN` |
| 5xx | `EURY_UPSTREAM_FAILED`, `EURY_INTERNAL` |

## Versioning

`v1` is stable from GA. Additive fields only; a breaking change means `/agent/v2` served in parallel for ≥ 6 months. Desktop sends `X-Agent-Client-Version`; cloud may return `426 EURY_CLIENT_TOO_OLD` when below `minSupported`.

## Conformance tests

| ID | Test |
|---|---|
| T1 | Every route in this document exists, and every registered Agent route appears here (generated test, fails on drift) |
| T2 | No Agent route is served outside the `/agent/v1` prefix |
| T3 | Every endpoint rejects an absent, malformed, expired, and wrong-audience token with the documented code |
| T4 | Every request DTO rejects unknown fields and returns `EURY_REQUEST_INVALID` with the offending field names |
| T5 | Authorization: a member token is refused on every admin route (IDOR corpus across org boundaries) |
| T6 | Error responses always carry `code`, `message`, and `requestId`, and never carry a stack trace or upstream body |
| T7 | The HTTP status for every error code matches the [error taxonomy](15-error-taxonomy.md) mapping |
| T8 | Rate-limit and quota responses set `Retry-After`; 402 is used only for budgets |
| T9 | The gateway stream emits well-formed NDJSON frames in the documented order and always terminates with `done` or `error` |
| T10 | Idempotency keys deduplicate replayed mutations |
| T11 | Pagination cursors are opaque, stable, and reject tampering |
| T12 | OpenAPI output matches the implementation and is committed as a golden file |

## Related documents

- [Backend module structure](16-backend-module-structure.md)
- [Cloud data model](07-cloud-data-model.md)
- [Cloud architecture](../02-architecture/03-cloud-architecture.md)
- [Provider and model governance](../02-architecture/07-provider-and-model-governance.md)
- [Multimodal and attachments](17-multimodal-and-attachment-spec.md)
- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
