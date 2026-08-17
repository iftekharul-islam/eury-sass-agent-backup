# Glossary

Spec-Version: 1.1.0

Terms used consistently across Eury Agent documentation.

## Product terms

| Term | Definition |
|------|------------|
| **Eury Agent** | The Tauri desktop application (product name). |
| **Workspace** | A root directory on disk the user has opened; all tools are scoped here. |
| **Home** | Workspace-independent developer assistance, projects, artifacts, and customization; it has no implicit repository access. |
| **Mode** | Permission profile: Chat, Ask, Plan, Agent, or Build. Review is a workflow, not a mode. |
| **Run** | One user-initiated agent session from prompt to completion/cancel. |
| **Turn** | One model call + optional tool executions within a run. |
| **Step** | A single item in a Plan or Build workflow. |

## Architecture terms

| Term | Definition |
|------|------------|
| **Desktop core** | Rust code inside the Tauri process: agent, tools, sandbox, store. |
| **AgentEngine** | Our trait abstracting the agent SDK; Cersei is one implementation. |
| **Cersei** | Rust SDK for coding agents ([docs](https://cersei.tryatlas.cc/docs)). Embedded, not a separate service. |
| **Eury Cloud** | NestJS backend: auth, billing, gateway, policy, audit, sync. |
| **Managed gateway** | Cloud proxy for LLM calls when user does not BYOK. |
| **BYOK** | Bring Your Own Key — user stores provider API key in OS keychain; desktop calls provider directly. |

## Security terms

| Term | Definition |
|------|------------|
| **Sandbox** | Path guard + command allowlist + OS-level restrictions on tool execution. |
| **Policy engine** | Rules deciding whether a tool call is allowed, needs approval, or denied. |
| **Approval** | Explicit user consent for a single tool invocation or scoped grant. |
| **Audit event** | Immutable record of tool use, model call metadata, policy decision. |

## Data terms

| Term | Definition |
|------|------------|
| **Local store** | Encrypted SQLite in app data dir. |
| **Graph memory** | Cersei Grafeo-backed relationship memory per workspace. |
| **EURY.md** | Project instruction file (CLAUDE.md-compatible hierarchy). |
| **Plan file** | Markdown + `plan_steps` JSON in `<workspace>/.eury/plans/`. |
| **Checkpoint** | Snapshot of workspace + run journal for rollback. |

## Protocol terms

| Term | Definition |
|------|------------|
| **Agent event** | Streaming message from core to UI (`text_delta`, `tool_start`, etc.). |
| **IPC command** | Tauri invoke from UI to Rust (`agent_run_start`, etc.). |
| **NDJSON stream** | Newline-delimited JSON for model streaming via cloud gateway. |

## Enterprise terms

| Term | Definition |
|------|------------|
| **Organization** | Billing and policy boundary; maps to existing `Organization` in Prisma. |
| **Workspace policy** | Org-distributed rules: allowed tools, models, MCP servers. |
| **Seat** | Licensed org member slot (Business plan). |
| **SCIM** | Automated user provisioning from IdP. |

## Legacy terms (migration)

| Term | Definition |
|------|------------|
| **code-old** | Deprecated PySide6 Eury Code app. |
| **Eury Code** | Former product name; superseded by **Eury Agent**. |
| **`/code/*`** | Legacy Nest API namespace; parallel to `/agent/v1/*` during migration. |
