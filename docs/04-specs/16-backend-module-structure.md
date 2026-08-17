# Backend Module Structure (Self-Contained Agent Module)

Spec-Version: 1.0.0

## Decision

Eury Agent's cloud API is **one NestJS module inside the existing backend app** — not a separate service, not a new deployment. Everything the Agent needs lives inside `backend/src/modules/agent/`.

Rationale: a separate microservice adds a network hop, another deployment target, another auth boundary, and another on-call surface for no benefit ([ADR-0001](../02-architecture/adr/0001-embed-cersei-in-desktop.md) already removed the agent loop from the server, so the cloud side is thin).

## Hard rules

| # | Rule | Why |
|---|---|---|
| R1 | All Agent routes are under `/agent/v1/*`, registered by controllers inside the Agent module only | No route collision with `code`, `eury`, `auth`, `admin` |
| R2 | The Agent module MUST NOT be imported by any other module | Deletable/replaceable in one step |
| R3 | The Agent module MUST NOT inject a service owned by another feature module | No coupling, no cascading breakage |
| R4 | Shared infrastructure is consumed only through globally-provided primitives (`PrismaService`, `ConfigService`, `JwtService` instance created *by* the Agent module) | Infra, not features |
| R5 | Every Prisma model the Agent owns is prefixed `Agent` | No table name collision |
| R6 | Existing models are changed only by **adding** back-relation fields | No destructive migration |
| R7 | No shared-secret bypass (`CODE_API_TOKEN` equivalent) exists | Closes the legacy open-endpoint bug |
| R8 | Agent logic is never added to an existing controller/service/guard | Reviewable blast radius |

### On R3 — duplication is intentional

Where the Agent needs behavior that a legacy module already has (prompt assembly, model catalog validation, provider key lookup, Tavily search), the Agent module gets **its own implementation** under `agent/`. Copying ~200 lines is cheaper than coupling two product lifecycles. The legacy `code` module keeps its own copy until sunset.

## Directory layout

```
backend/src/modules/agent/
  agent.module.ts                 # only module registered in AppModule
  agent.constants.ts              # route prefix, limits, event names

  config/
    agent-config.service.ts       # reads env; no other module's config
    agent-env.schema.ts           # validation of AGENT_* + provider keys

  auth/
    agent-auth.controller.ts      # /agent/v1/auth/*
    agent-auth.service.ts         # device flow (PKCE), refresh rotation
    agent-auth.guard.ts           # bearer access token -> AgentPrincipal
    agent-admin.guard.ts          # admin-only agent routes
    agent-scim.guard.ts           # static SCIM bearer token
    agent-principal.ts            # { userId, orgId, plan, deviceId, scopes }
    dto/
      agent-device-start.dto.ts
      agent-device-exchange.dto.ts
      agent-device-poll.dto.ts
      agent-refresh.dto.ts

  chat/
    agent-chat.controller.ts      # /agent/v1/chat/stream
    agent-chat.service.ts         # upstream LLM proxy + NDJSON tee
    agent-prompt.builder.ts       # Agent-owned prompt assembly
    agent-model-catalog.ts        # Agent-owned catalog + validation
    agent-provider.resolver.ts    # Agent-owned provider/key resolution
    dto/agent-chat-stream.dto.ts

  tools/
    agent-tools.controller.ts     # /agent/v1/tools/web-search, /tavily
    agent-web-search.service.ts
    dto/agent-web-search.dto.ts

  policy/
    agent-policy.controller.ts    # /agent/v1/policies/*
    agent-policy.service.ts       # merge org -> team -> user caps
    agent-policy.schema.ts        # WorkspacePolicy zod/class-validator
    dto/agent-policy-upsert.dto.ts

  audit/
    agent-audit.controller.ts     # /agent/v1/audit/batch
    agent-audit.service.ts        # signature verify + append-only write
    agent-audit-export.service.ts # CSV/JSON export for admins

  devices/
    agent-device.controller.ts    # /agent/v1/devices/*
    agent-device.service.ts       # enrollment, public key, revoke

  releases/
    agent-release.controller.ts   # /agent/v1/releases/latest
    agent-release-admin.controller.ts # /agent/v1/admin/releases/*
    agent-release.service.ts      # manifest + upload to object storage
    dto/agent-release-upsert.dto.ts

  usage/
    agent-usage.service.ts        # quota counters (Redis + AgentUsageCounter)
    agent-usage.guard.ts          # applied to /chat/stream

  sync/
    agent-sync.controller.ts      # /agent/v1/sync/*
    agent-sync.service.ts

  scim/
    agent-scim.controller.ts      # /agent/v1/scim/v2/*
    agent-scim.service.ts

  common/
    agent-ndjson.util.ts
    agent-redaction.util.ts
    agent-errors.ts               # maps to EURY_* taxonomy
    agent-logger.ts               # writes backend/logs/agent/
```

## Module definition

```ts
@Module({
  imports: [
    // infrastructure only — never a feature module
    PrismaModule,
    JwtModule.registerAsync({ /* AGENT_* secrets, Agent-owned instance */ }),
  ],
  controllers: [
    AgentAuthController,
    AgentChatController,
    AgentToolsController,
    AgentPolicyController,
    AgentAuditController,
    AgentDeviceController,
    AgentReleaseController,
    AgentReleaseAdminController,
    AgentSyncController,
    AgentScimController,
  ],
  providers: [ /* Agent services + guards only */ ],
  exports: [], // R2: exports nothing
})
export class AgentModule {}
```

`AppModule` change is exactly one line: `imports: [ ..., AgentModule ]`.

## Route prefix

Controllers declare the full prefix so no global prefix change is required:

```ts
@Controller('agent/v1/chat')
export class AgentChatController { /* ... */ }
```

`agent.constants.ts` holds `AGENT_API_PREFIX = 'agent/v1'` for reference and tests.

## Authentication (Agent-owned)

The Agent module issues and validates its **own** tokens. It does not read `Session`, `IdeAuthSession`, or the auth module's guards.

| Token | Storage | TTL | Rotation |
|---|---|---|---|
| Agent access token (JWT) | desktop keychain | 15 min | via refresh |
| Agent refresh token (opaque, hashed) | `AgentRefreshToken` | 30 d | rotate-on-use, reuse detection |
| Device auth session | `AgentAuthSession` | 10 min | single-use, PKCE S256 |

The browser exchange step (`POST /agent/v1/auth/device/exchange`) is the only place the Agent module needs a signed-in web user. It accepts the platform's normal web session cookie/JWT and verifies it with its **own** `JwtService` instance configured from `JWT_SECRET`; it does not call `AuthService`. It then mints Agent-scoped tokens.

`AgentAuthGuard` behavior:

1. Read `Authorization: Bearer <token>`; reject if absent → `EURY_AUTH_UNAUTHORIZED`.
2. Verify signature and `exp`.
3. Load `AgentDevice` + `AgentRefreshToken` state; reject if device revoked.
4. Attach `AgentPrincipal` to the request.
5. **No fallback path.** Missing config causes startup failure, never an open endpoint.

## Data ownership

The Agent module reads/writes only:

`AgentRelease`, `AgentDevice`, `AgentRefreshToken`, `AgentAuthSession`, `AgentPolicy`, `AgentAuditEvent`, `AgentUsageCounter`, `AgentSyncCursor`

Read-only, by id, for entitlement and scoping: `User`, `Organization`, `OrganizationMember`, `Subscription`, `Plan`. These reads go through `PrismaService` directly — not through `EntitlementService` or `OrganizationsService` (R3).

Schema: [cloud data model](07-cloud-data-model.md).

## Configuration

`AgentConfigService` validates on boot and fails fast:

```ts
AGENT_JWT_SECRET            // required
AGENT_REFRESH_TOKEN_SECRET  // required
AGENT_UPSTREAM_LLM_URL      // required
AGENT_SCIM_TOKEN            // optional; SCIM routes 404 without it
AGENT_RELEASE_BUCKET        // required for admin uploads
AGENT_RELEASE_*             // env fallback manifest
```

Provider keys are read by `AgentProviderResolver` from process env with an Agent-specific precedence chain, documented in [environments and config](../07-ops/01-environments-and-config.md).

## Testing boundary

| Test | Scope |
|---|---|
| Unit | Each Agent service in isolation with mocked `PrismaService` |
| Contract | Every `/agent/v1/*` route against golden request/response fixtures |
| Guard | `AgentAuthGuard` denies with no token, bad signature, revoked device, missing config |
| Isolation | Static check: no import from `../auth`, `../code`, `../eury`, `../billing`, `../organizations` service files |

The isolation test is a CI grep/lint rule and is a Phase 25 exit criterion.

## Deletion test

A valid implementation satisfies: removing `AgentModule` from `AppModule` and deleting `backend/src/modules/agent/` leaves the rest of the backend compiling and its tests green (Agent Prisma models remain but are unreferenced).

## Conformance tests

| ID | Test |
|---|---|
| T1 | No file under `modules/agent/` imports from another feature module; only infrastructure imports are permitted (lint rule) |
| T2 | No file outside `modules/agent/` imports from `modules/agent/` |
| T3 | Every controller in the module resolves to a path under `/agent/v1` |
| T4 | The Agent module registers its own guards; removing a shared guard cannot silently open an Agent route |
| T5 | Deleting the module leaves the backend compiling with all other tests green (the deletion test, run in CI) |
| T6 | Agent configuration reads only `AGENT_*` variables plus shared infrastructure variables |
| T7 | Agent Prisma access is limited to `Agent*` models plus documented read-only lookups on shared models |
| T8 | Agent tests run standalone without booting other feature modules |

## Related documents

- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
- [Cloud API contract](06-cloud-api-contract.md)
- [Cloud architecture](../02-architecture/03-cloud-architecture.md)
