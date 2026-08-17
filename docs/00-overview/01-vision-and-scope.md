# Vision and Scope

Spec-Version: 1.1.0

## Vision

Eury Agent is the desktop coding agent for the Eury platform: a fast, trustworthy, locally-executed AI pair programmer that enterprises can govern. Users open a workspace, describe intent in natural language, and the agent reads, plans, edits, tests, and explains — with every filesystem and shell action visible, approvable, and auditable.

## Mission

Deliver a **production + enterprise-grade** coding agent that:

1. Matches or exceeds incumbent UX (Cursor, Claude Code, Windsurf) on core workflows.
2. Keeps latency low by embedding the agent runtime in the desktop process.
3. Keeps source code local by default; only policy-authorized prompt context transits a selected model route, and cloud persistence occurs only for explicitly enabled sync or required audit metadata.
4. Scales from solo developers (BYOK) to regulated teams (SSO, policy, audit).

## Product boundaries

### In scope

- Desktop app: macOS, Windows, Linux.
- Workspace-scoped agent with Cersei embedded in Rust core.
- Modes: Chat, Ask, Plan, Agent, Build. Review is a multi-agent workflow, not a sixth mode.
- Local tools: filesystem, shell, git, web fetch/search, MCP.
- Cloud: identity, billing, managed models, policy distribution, optional sync, release manifests.
- Enterprise: SSO, SCIM, RBAC, workspace policies, tamper-evident audit.

### Out of scope (v1 GA)

- Mobile clients.
- Cloud-hosted IDE (browser-only editing).
- Running user code on Eury servers.
- Replacing the Next.js web app or Collab editor.
- Training custom foundation models.

See [../01-product/05-non-goals.md](../01-product/05-non-goals.md) for the full non-goals list.

## Success metrics (GA)

| Metric | Target |
|--------|--------|
| Cold start to interactive UI | < 2 s (p95) on M1 / equivalent |
| First token after send (managed model) | < 1.5 s (p95), network permitting |
| Tool dispatch (local read_file) | < 50 ms (p95) |
| Agent task success rate (eval harness) | ≥ 85% on internal v1 suite |
| Crash-free sessions | ≥ 99.5% |
| Enterprise audit coverage | 100% of write/execute/network tools |

## Stakeholders

| Stakeholder | Primary need |
|-------------|--------------|
| Individual developer | Fast agent, BYOK, low friction |
| Team lead | Shared projects, usage visibility |
| Security / IT | Policy, SSO, audit, no silent exfiltration |
| Platform engineering | Observable cloud, clear SLOs, safe releases |

## Relationship to existing repos

| Component | Fate |
|-----------|------|
| `code-old/` | Deprecated; feature inventory only |
| `backend/` | Gains one self-contained `AgentModule` serving `/agent/v1/*`; the legacy `code` module and `/code/*` are untouched until sunset |
| `frontend/` | New pages only: `/{locale}/agent/authorize`, `/{locale}/eury-agent`, `/admin/agent/*` |

Every identifier that the legacy stack already uses is renamed for Agent — routes, tables, env vars, artifacts, local paths. The authoritative table is [naming and migration map](05-naming-and-migration-map.md); isolation rules are in [backend module structure](../04-specs/16-backend-module-structure.md).

## Document map

This file is the north star. Implementation order follows [../09-roadmap/00-roadmap-overview.md](../09-roadmap/00-roadmap-overview.md).
