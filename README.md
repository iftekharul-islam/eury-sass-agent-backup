# Eury Agent

Production-grade desktop coding agent for the Eury platform.

**Status:** Phase 0 foundation, Phase 1 product contract, Phase 2 security foundation, Phase 3 desktop shell, Phase 4 agent core, Phase 5 workspace and sandbox, Phase 6 tool layer v1, Phase 7 policy and approval system, Phase 8 chat experience, Phase 9 local persistence, and Phase 10 identity completed and automatically verified. The agent now owns its authentication stack with a PKCE device flow and secure keychain storage. Phase 11 network gateway is next.

## What this is

Eury Agent is a Tauri desktop application with an embedded [Cersei](https://cersei.tryatlas.cc/docs) agent runtime. It replaces the deprecated PySide6 app in `code-old/`. The agent loop, tool execution, indexing, and primary data store all run on the user's machine. The NestJS backend provides identity, billing, a managed model gateway, policy distribution, audit ingestion, and optional sync — it never runs the agent loop and never touches local files.

## Start here

**[docs/00-index.md](docs/00-index.md)** — the complete documentation map with a "what should I read for my task" table.

**[mockups/index.html](mockups/index.html)** — the clickable design mockup for all ten screens. Open it in a browser; it needs no build step and works offline.

| Order | Folder | Purpose |
|---|---|---|
| 1 | [docs/00-overview/](docs/00-overview/) | Vision, glossary, conventions, naming map |
| 2 | [docs/02-architecture/](docs/02-architecture/) | System design, 10 ADRs, latency budget, offline modes |
| 3 | [docs/03-security/](docs/03-security/) | Threat model, sandbox, permissions, privacy, compliance |
| 4 | [docs/01-product/](docs/01-product/) | Personas, feature catalog, modes, pricing, non-goals |
| 5 | [docs/04-specs/](docs/04-specs/) | 17 implementation-ready contracts |
| 6 | [docs/05-ui/](docs/05-ui/) + [mockups/](mockups/) | Visual language, design system, UX specs, and the clickable mockup |
| 7 | [docs/06-enterprise/](docs/06-enterprise/) | SSO, RBAC, policies, audit, quotas, admin, air-gapped |
| 8 | [docs/07-ops/](docs/07-ops/) + [docs/08-quality/](docs/08-quality/) | Config, CI/CD, packaging, updates, SLOs, runbooks, testing |
| 9 | [docs/09-roadmap/](docs/09-roadmap/) | 30-phase delivery plan with exit criteria |

Bangla executive summary: [docs/README.bn.md](docs/README.bn.md)

## Locked decisions

1. **The agent loop runs in the desktop Rust core** — tool execution, policy, sandboxing, and file access never leave the machine. Model inference goes through the managed gateway (`/agent/v1/chat/stream`), which is where org control and billing are enforced; this supersedes the original "Cersei embedded, no network hop" decision (see [phase-04](docs/09-roadmap/phase-04.md#architecture-as-shipped-managed-gateway-not-embedded-cersei)).
2. **`AgentEngine` trait boundary** — no engine- or provider-specific type appears outside the adapter crate, so swapping the engine stays a one-crate change.
3. **Tauri 2 + React 19 + TypeScript + Tailwind 4** for the shell.
4. **Encrypted SQLite** locally; secrets only in the OS keychain.
5. **BYOK by default**, managed gateway for org control and billing.
6. **Deny-by-default** for every write, execute, and network tool.
7. **One self-contained NestJS module** for all cloud endpoints, under `/agent/v1/*`.
8. **A desktop GUI, not a terminal clone** — Claude Code's capabilities in a focused two-region Code workspace with dedicated panes, with monospace reserved for code, paths, and diffs.
9. **Home and Code as persistent top-level areas** — Home for general chat, projects, artifacts, and customization; Code for repository-scoped agent work.

Full reasoning: [docs/02-architecture/adr/](docs/02-architecture/adr/).

## Isolation from the legacy `code` stack

Eury Agent shares **no** route, table, environment variable, artifact name, or local path with the deprecated Eury Code stack. The legacy client keeps working untouched until its sunset six months after GA.

| Concern | Legacy | Eury Agent |
|---|---|---|
| API namespace | `/code/*`, `/auth/ide/*` | `/agent/v1/*` (including its own auth) |
| Backend code | `backend/src/modules/code/` | `backend/src/modules/agent/` (imports no other feature module) |
| Database | `DesktopRelease`, `IdeAuthSession` | `AgentRelease`, `AgentAuthSession`, `Agent*` only |
| Local data | `~/.eury-code/` | OS app-data dir for `com.eury.agent` |
| Artifacts | `Eury-Code-*` | `Eury-Agent-*` |
| Env vars | `CODE_*`, `EURY_CODE_*` | `AGENT_*`, `EURY_AGENT_*` |

The authoritative table is [docs/00-overview/05-naming-and-migration-map.md](docs/00-overview/05-naming-and-migration-map.md), and the isolation rules (with the automated CI check and the deletion test) are in [docs/04-specs/16-backend-module-structure.md](docs/04-specs/16-backend-module-structure.md).

## Repository layout

```
agent/
  README.md           ← you are here
  docs/               ← all specifications (complete)
  mockups/            ← clickable design mockup, ten screens (complete)
  version.json        ← single source of truth for the app version
  apps/
    desktop/          ← Tauri shell + React UI (Phase 3+)
  crates/             ← Rust workspace (Phase 4+)
    agent-types/      ← shared I/O-free contract types
    agent-core/       ← AgentEngine trait, run lifecycle, isolated Cersei adapter
    agent-tools/      ← tool registry, MCP bridge
    agent-sandbox/    ← the only crate allowed filesystem and process access
    agent-policy/     ← policy merge, grants, approvals
    agent-store/      ← encrypted SQLite, migrations
    agent-index/      ← workspace indexing and retrieval
    agent-memory/     ← EURY.md hierarchy and memory graph
  bench/              ← benchmarks + REPORT.md (Phase 28)
  eval/               ← agent eval harness (Phase 9+)
  tests/fixtures/     ← shared test repositories and corpora
```

## Phase 0 development

Prerequisites are pinned in [`.node-version`](.node-version) and
[`rust-toolchain.toml`](rust-toolchain.toml). From `agent/`:

```bash
pnpm install --frozen-lockfile
pnpm check
cargo check --workspace
pnpm --filter @eury/desktop tauri dev
```

`version.json` is the version source of truth. `pnpm version:check`,
`pnpm tokens:check`, `pnpm docs:check`, and `pnpm boundaries:check` enforce
the Phase 0 repository contracts.

## Phase FF local dev (5-minute setup)

Phase FF wires Home cloud chat and Code workspace agent against the NestJS backend on port **3001**. From the repo root:

```bash
# 1. Backend (terminal 1)
cd backend && PORT=3001 pnpm start:dev

# 2. Web authorize page (terminal 2)
cd frontend && pnpm dev

# 3. Desktop app (terminal 3)
cd agent
export VITE_EURY_AGENT_API_URL=http://localhost:3001/agent/v1
export EURY_AGENT_GATEWAY_URL=http://localhost:3001/agent/v1/chat/stream
pnpm --filter @eury/desktop tauri dev
```

Sign in via device auth in the desktop app, send a Home chat message, then use **Open project** to pick a folder and run the Code agent. See [docs/09-roadmap/phase-ff.md](docs/09-roadmap/phase-ff.md) for exit criteria.

## Related packages

| Path | Role |
|---|---|
| [backend/](../backend/) | NestJS cloud API; gains one `AgentModule` |
| [frontend/](../frontend/) | Next.js web app; gains the authorize, download, and `/admin/agent/*` pages |
| [code-old/](../code-old/) | Deprecated PySide6 desktop — feature reference only |

## Contributing

1. Read [docs/00-overview/04-doc-conventions.md](docs/00-overview/04-doc-conventions.md) and [docs/08-quality/05-definition-of-done.md](docs/08-quality/05-definition-of-done.md).
2. Follow the [naming map](docs/00-overview/05-naming-and-migration-map.md) — no legacy `code` identifiers in new code.
3. When implementation shows a spec is wrong, fix the spec in the same PR and bump its `Spec-Version`.
4. Every phase has objective exit criteria. Do not skip gates, and do not weaken a security default to make a feature work.
