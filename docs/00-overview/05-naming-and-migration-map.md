# Naming and Migration Map

Spec-Version: 1.0.0

Authoritative rename table. The deprecated product was **Eury Code** (`code-old/`, Nest module `code`). The new product is **Eury Agent**. Every identifier below MUST use the Agent column. No new code may introduce the legacy column.

## Rule

> **Nothing in Eury Agent shares a name, route, table, env var, directory, or file with the legacy `code` stack.**

This guarantees the legacy client keeps working during migration and that the Agent module can be deleted or replaced without touching shared services.

## Product and brand

| Legacy (`code`) | Eury Agent | Notes |
|---|---|---|
| Eury Code | Eury Agent | Product name |
| `code-old/` | `agent/` | Repo folder |
| `eury_code` (Python package) | `agent-*` (Rust crates) | See [crate split](../02-architecture/adr/0007-rust-workspace-crate-split.md) |
| `com.eury.code` | `com.eury.agent` | Bundle / app id |
| `Eury-Code-x.y.z.dmg` | `Eury-Agent-x.y.z.dmg` | Installer artifact |
| Git tag `vX.Y.Z` | Git tag `agent-vX.Y.Z` | Avoids clashing with backend/frontend tags |

## Backend HTTP surface

All Agent routes live under a single prefix. No Agent route is added to an existing controller.

| Legacy | Eury Agent | Auth |
|---|---|---|
| `GET /code/health` | `GET /agent/v1/health` | public |
| `GET /code/version` | `GET /agent/v1/releases/latest` | public |
| `GET /code/models` | `GET /agent/v1/models` | bearer |
| `POST /code/stream` | `POST /agent/v1/chat/stream` | bearer |
| `POST /code/web-search` | `POST /agent/v1/tools/web-search` | bearer |
| `POST /code/tavily` | `POST /agent/v1/tools/tavily` | bearer |
| `POST /auth/ide/device/start` | `POST /agent/v1/auth/device/start` | public |
| `POST /auth/ide/device/exchange` | `POST /agent/v1/auth/device/exchange` | user JWT (browser) |
| `POST /auth/ide/device/poll` | `POST /agent/v1/auth/device/poll` | public + PKCE |
| *(none — gap)* | `POST /agent/v1/auth/refresh` | refresh token |
| *(none)* | `POST /agent/v1/auth/logout` | bearer |
| *(none)* | `GET /agent/v1/me` | bearer |
| `/admin/desktop-releases/*` | `/agent/v1/admin/releases/*` | admin |
| *(none)* | `GET /agent/v1/policies/effective` | bearer |
| *(none)* | `POST /agent/v1/audit/batch` | bearer |
| *(none)* | `POST /agent/v1/devices/enroll` | bearer |
| *(none)* | `/agent/v1/sync/*` | bearer |
| *(none)* | `/agent/v1/scim/v2/*` | SCIM token |

Full contract: [cloud API contract](../04-specs/06-cloud-api-contract.md).

## Backend code layout

| Legacy | Eury Agent |
|---|---|
| `backend/src/modules/code/` | `backend/src/modules/agent/` |
| `CodeModule` | `AgentModule` |
| `CodeController` | `AgentChatController`, `AgentAuthController`, … |
| `CodeService` | `AgentChatService`, … |
| `CodeAuthGuard` | `AgentAuthGuard` (no unauthenticated fallback) |
| `CodeTokenGuard` | *(removed — no shared-secret bypass)* |
| `CodeChatDto` | `AgentChatStreamDto` |
| `CodePromptBuilderService` | `AgentPromptBuilderService` (inside module) |
| `CodeDebugLogger` | `AgentDebugLogger` (inside module) |
| `DesktopReleaseService` | `AgentReleaseService` (inside module) |

Layout rules: [backend module structure](../04-specs/16-backend-module-structure.md).

## Database (Prisma)

Every Agent table is prefixed `Agent`. No legacy table is altered; only additive back-relation fields are added to `User` / `Organization`.

| Legacy | Eury Agent |
|---|---|
| `DesktopRelease` | `AgentRelease` |
| *(none)* | `AgentDevice` |
| *(none)* | `AgentRefreshToken` |
| *(none)* | `AgentAuthSession` (replaces `IdeAuthSession` for Agent) |
| *(none)* | `AgentPolicy` |
| *(none)* | `AgentAuditEvent` |
| *(none)* | `AgentUsageCounter` |
| `IdeAuthSession` | *(kept for legacy client only; Agent does not read it)* |

Schema: [cloud data model](../04-specs/07-cloud-data-model.md).

## Environment variables

| Legacy | Eury Agent | Side |
|---|---|---|
| `BACKEND_URL` | `EURY_AGENT_CLOUD_URL` | desktop |
| `CODE_API_TOKEN` | *(removed — no shared secret)* | — |
| `EURY_CODE_DATA_DIR` | `EURY_AGENT_DATA_DIR` | desktop |
| `EURY_CODE_BACKEND_URL` (CI var) | `EURY_AGENT_CLOUD_URL` (CI var) | CI |
| `CODE_DESKTOP_VERSION` | `AGENT_RELEASE_FALLBACK_VERSION` | cloud |
| `CODE_DESKTOP_MIN_SUPPORTED` | `AGENT_RELEASE_MIN_SUPPORTED` | cloud |
| `CODE_DESKTOP_URL_DARWIN` | `AGENT_RELEASE_URL_DARWIN` | cloud |
| `CODE_DESKTOP_URL_WIN32` | `AGENT_RELEASE_URL_WIN32` | cloud |
| `CODE_DESKTOP_URL_LINUX` | `AGENT_RELEASE_URL_LINUX` | cloud |
| *(none)* | `AGENT_REFRESH_TOKEN_SECRET` | cloud |
| *(none)* | `AGENT_SCIM_TOKEN` | cloud |

Shared provider keys (`OPENAI_API_KEY`, `CLAUDE_API_KEY`, …) and `DATABASE_URL` stay as-is — the Agent module reads them through its own thin config provider, never by importing another module's service.

## Local filesystem paths

| Legacy | Eury Agent |
|---|---|
| `~/.eury-code/` | OS app-data dir for `com.eury.agent` |
| `~/.eury-code/sessions.json` | `db/agent.sqlite` (encrypted) |
| `~/.eury-code/auth.json` | OS keychain (`eury.agent.auth.*`) |
| `~/.eury-code/eury-preferences.json` | `settings` table in SQLite |
| `~/.eury-code/conversation-cache.json` | `conversations` table |
| `~/.eury-code/run-checkpoints/` | `checkpoints/<run_id>/` |
| `<workspace>/.eury/plans/` | `<workspace>/.eury/plans/` (unchanged) |
| `CLAUDE.md` compatibility | `EURY.md` (+ `.eury.local.md`) |

Paths: [desktop runtime](../02-architecture/02-desktop-runtime.md).

## Release and download paths

| Concern | Legacy | Eury Agent |
|---|---|---|
| Object storage prefix | `desktop-releases/` | `agent-releases/<version>/` |
| Admin upload route | `POST /admin/desktop-releases/upload` | `POST /agent/v1/admin/releases/upload` |
| Manifest route | `GET /code/version` | `GET /agent/v1/releases/latest` |
| Marketing page | `/bn/eury-code` | `/bn/eury-agent` |
| CI workflow | `desktop-release.yml` | `agent-release.yml` |
| CI workflow (tests) | *(none)* | `agent-ci.yml` |

Details: [packaging](../07-ops/03-packaging-signing-notarization.md), [auto-update](../07-ops/04-auto-update-and-rollback.md).

## Frontend (web) additions

| Purpose | Route |
|---|---|
| Device authorize page | `/{locale}/agent/authorize` |
| Download / marketing | `/{locale}/eury-agent` |
| Admin releases | `/admin/agent/releases` |
| Admin policies | `/admin/agent/policies` |
| Admin audit | `/admin/agent/audit` |

Legacy `/{locale}/ide/authorize` stays for the old client until sunset.

## Telemetry and log namespaces

| Legacy | Eury Agent |
|---|---|
| `backend/logs/code/` | `backend/logs/agent/` |
| metric `code_stream_*` | metric `agent_chat_stream_*` |
| error prefix (none) | `EURY_*` codes ([taxonomy](../04-specs/15-error-taxonomy.md)) |

## Sunset plan

| Step | Trigger |
|---|---|
| Agent GA ships | Phase 29 |
| Legacy `/code/*` marked deprecated in response headers | GA + 0 |
| Legacy desktop forced-update notice | GA + 1 month |
| `/code/*`, `code` module, `IdeAuthSession` removed | GA + 6 months (see [Q09](../09-roadmap/open-questions.md)) |

Until removal, both stacks run side by side with **zero shared code paths**.
