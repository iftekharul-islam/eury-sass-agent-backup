# Cloud Architecture

Spec-Version: 1.1.0

## Role

Eury Cloud (existing NestJS `backend/`) is the **control plane**. It never executes agent tools, never reads user repositories, and never runs the agent loop — that all lives in the desktop Rust core ([ADR-0001](adr/0001-embed-cersei-in-desktop.md)).

## Deployment shape

One module inside the existing Nest app. **Not** a separate service.

```
backend/ (single Nest app, single deploy)
├── AppModule
│   ├── ...existing feature modules (untouched)
│   └── AgentModule            ← one line added to imports
│       └── /agent/v1/*        ← every Agent route
```

Why not a separate service: extra network hop on the auth and gateway path, second deploy target, duplicate secret distribution, and a second on-call surface — for a control plane that is mostly reads. Revisit only if Agent traffic requires independent scaling (tracked as [Q10](../09-roadmap/open-questions.md)).

Internal structure, hard isolation rules, and the deletion test: [backend module structure](../04-specs/16-backend-module-structure.md).

## Sub-areas inside `AgentModule`

| Area | Path prefix | Purpose |
|---|---|---|
| `auth/` | `/agent/v1/auth` | Agent-owned PKCE device flow, refresh rotation, logout |
| `chat/` | `/agent/v1/chat`, `/agent/v1/models` | Managed LLM gateway (Enterprise/managed plans) |
| `tools/` | `/agent/v1/tools` | Server-side tool proxies (web search) |
| `policy/` | `/agent/v1/policies` | Org → team → user policy merge and distribution |
| `audit/` | `/agent/v1/audit` | Signed batch audit ingest and export |
| `devices/` | `/agent/v1/devices` | Device enrollment, key registry, revocation |
| `releases/` | `/agent/v1/releases`, `/agent/v1/admin/releases` | Update manifest + admin publish |
| `usage/` | (guard, no routes) | Quota counters and budget enforcement |
| `sync/` | `/agent/v1/sync` | Optional conversation/memory sync |
| `scim/` | `/agent/v1/scim/v2` | SCIM 2.0 user/group provisioning |

Legacy `/code/*` keeps running unchanged until sunset ([naming and migration map](../00-overview/05-naming-and-migration-map.md)). No Agent code path touches it.

## What the Agent module does *not* reuse

| Legacy capability | Agent approach |
|---|---|
| `AuthService`, `IdeAuthService`, `/auth/ide/*` | Own controller + service under `/agent/v1/auth` |
| `IdeAuthSession`, `Session` tables | Own `AgentAuthSession`, `AgentRefreshToken` |
| `CodeAuthGuard` (with `CODE_API_TOKEN` bypass) | `AgentAuthGuard`, no fallback, fails closed |
| `DesktopRelease` + `/admin/desktop-releases` | `AgentRelease` + `/agent/v1/admin/releases` |
| `CodePromptBuilderService`, code model catalog | Agent-owned prompt builder and catalog |
| `EntitlementService`, `OrganizationsService` | Direct read-only Prisma queries on `Subscription`, `Plan`, `OrganizationMember` |

Shared **infrastructure** is still shared: `PrismaService`, Redis client, object storage client, `ConfigService`, logger transport. Shared **features** are not.

## Authentication

### Device login (desktop → cloud)

```
Desktop                     Browser                    Nest AgentModule
  │ 1. POST /agent/v1/auth/device/start                    │
  │    { code_challenge, device_name, platform }           │
  │───────────────────────────────────────────────────────>│  create AgentAuthSession
  │<─── { device_code, user_code, verify_url, interval } ───│
  │                                                        │
  │ 2. open verify_url ──────>│                            │
  │                           │ 3. POST /agent/v1/auth/device/exchange
  │                           │    (web session, user_code)│
  │                           │───────────────────────────>│  bind userId to session
  │ 4. POST /agent/v1/auth/device/poll                      │
  │    { device_code, code_verifier }                       │
  │───────────────────────────────────────────────────────>│  verify S256, single-use
  │<─── { access_token, refresh_token, expires_in } ────────│
```

| Property | Value |
|---|---|
| PKCE method | S256 only (`plain` rejected) |
| `AgentAuthSession` TTL | 10 min, single-use, deleted on success |
| Poll interval | 2 s; `EURY_AUTH_SLOW_DOWN` after 3 fast polls |
| Access token | JWT, 15 min, `aud: "eury-agent"` |
| Refresh token | 32-byte opaque, stored hashed (`AgentRefreshToken`), 30 d |
| Rotation | Rotate on every use; replay of a rotated token revokes the whole chain |
| Storage on device | OS keychain only ([secrets](../03-security/04-secrets-and-key-management.md)) |

### Guarding

All `/agent/v1/*` routes require `Authorization: Bearer <access_token>` except:

- `GET /agent/v1/health`
- `GET /agent/v1/releases/latest`
- `POST /agent/v1/auth/device/start`
- `POST /agent/v1/auth/device/poll`

`/agent/v1/admin/*` additionally requires `AgentAdminGuard` (platform admin or org owner). `/agent/v1/scim/v2/*` uses `AgentScimGuard` with a static provisioning token and 404s when unconfigured.

The legacy "no token configured → allow request" fallback is **not** reproduced. Missing required config fails app boot.

## Managed model gateway

Only used when the workspace is on a managed plan. BYOK calls go direct from desktop to provider ([ADR-0005](adr/0005-byok-vs-managed-gateway.md)).

```
Desktop                      Nest AgentModule                Upstream LLM
  │ POST /agent/v1/chat/stream    │                                │
  │──────────────────────────────>│ AgentAuthGuard                 │
  │                               │ AgentUsageGuard (quota/budget) │
  │                               │ model allowlist from policy    │
  │                               │ Redis rate limit + abort handle │
  │                               │ audit: metadata only            │
  │                               │───────────────────────────────>│
  │<────── NDJSON passthrough ────│<───────────────────────────────│
```

| Guarantee | Detail |
|---|---|
| Streaming | NDJSON tee'd through with no buffering beyond one line |
| Abort | Client disconnect → upstream abort via Redis-tracked handle |
| Timeout | 300 s wall clock per request |
| Logging | Never log full prompt at `info` in production; `debug` samples with redaction |
| Failure mode | Gateway 5xx → desktop may fall back to BYOK if policy allows |

Request/response shapes: [cloud API contract](../04-specs/06-cloud-api-contract.md).

## Policy distribution

```
GET /agent/v1/policies/effective
  → merge: org defaults → team overrides → user caps (most restrictive wins)
  → ETag; desktop refreshes on login, every 15 min, and on push hint
```

Desktop enforces locally so policy works offline; cloud is the source of truth for enterprise. Merge semantics: [permission and policy engine](../03-security/03-permission-and-policy-engine.md).

## Audit ingest

```
POST /agent/v1/audit/batch
  { device_id, events: AgentAuditEvent[], signature }
```

Events are signed with the device key registered at enrollment; a bad signature rejects the batch (`EURY_AUDIT_SIGNATURE_INVALID`). Storage is append-only Postgres with optional object-storage archive for long retention ([audit and retention](../06-enterprise/04-audit-and-retention.md)).

## Data stores

| Store | Agent data |
|---|---|
| PostgreSQL | `Agent*` tables only; read-only joins to `User`/`Organization`/`Subscription` |
| Redis | Rate limits, quota counters, stream abort handles, device-code throttle |
| Object storage | `agent-releases/<version>/…`, audit archives |

Schema: [cloud data model](../04-specs/07-cloud-data-model.md).

## Environments

| Env | Purpose | Notes |
|---|---|---|
| `local` | Developer machine, `PORT=4001` | `AGENT_*` secrets from `.env` |
| `staging` | Pre-prod integration | Separate release bucket prefix |
| `production` | Customer-facing | Secrets from platform secret manager |

Variables: [environments and config](../07-ops/01-environments-and-config.md).

## Related documents

- [System architecture](01-system-architecture.md)
- [Backend module structure](../04-specs/16-backend-module-structure.md)
- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Cloud data model](../04-specs/07-cloud-data-model.md)
- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
