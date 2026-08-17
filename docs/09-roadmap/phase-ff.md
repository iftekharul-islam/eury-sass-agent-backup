# Phase FF — Fast Forward MVP

Spec-Version: 1.0.0

**Track:** Acceleration · **Estimated size:** 2 weeks · **Milestone:** Usable desktop alpha

## Goal

Deliver a **usable desktop app** with two surfaces before completing the full formal roadmap:

1. **Home** — cloud-backed chat history (parity with web / `code-old`) and streaming general Q&A (no file tools).
2. **Code** — native folder picker, real workspace, and a basic agent loop with `read_file` / `write_file` plus approval.

Mock UI chrome (demo projects, tool cards, plan cards) **stays visible** as design scaffolding; real behavior wires in alongside it.

## Why this phase exists

The formal phases (0–29) build production-grade depth. Phase FF unblocks daily dogfooding: sign in → chat at Home → pick a folder → run an agent on real files — without waiting for SQLite persistence, terminal, editor, or full policy engine.

## Two-surface model

| Surface | Chat type | History | Agent tools | Data source |
|---------|-----------|---------|-------------|-------------|
| **Home** | General Q&A | `GET /conversations?variant=chat&globalOnly=true` | None | Cloud API when signed in; mock list when offline |
| **Code** | Workspace agent | In-memory session (FF only) | `read_file`, `write_file` + approval | Real folder after picker; mock projects remain in sidebar |

## Deliverables

| ID | Deliverable | Key files |
|----|-------------|-----------|
| DFF.1 | Unified dev URL config (port 3001) | `cloud.ts`, `config.ts`, `gateway.rs` |
| DFF.2 | Home conversations API client | `lib/conversations.ts`, `lib/useHomeChatHistory.ts` |
| DFF.3 | Home chat canvas + sidebar wiring | `HomeChatCanvas.tsx`, `Sidebar.tsx`, `App.tsx` |
| DFF.4 | Folder picker + project registry | `tauri-plugin-dialog`, `workspace_pick_folder`, `lib/projects.ts` |
| DFF.5 | Live streaming (Home + Code) | `HomeChatCanvas.tsx`, `ConversationCanvas.tsx`, `stream-events.ts` |
| DFF.6 | Agent loop (Code only) | `agent-core/src/agent_loop.rs`, `commands.rs` |
| DFF.7 | Approval IPC | `run_approve`, `ApprovalCard.tsx` |
| DFF.8 | Auth + model bootstrap | `agent.module.ts`, `bootstrap.ts`, `PlatformOrAgentAuthGuard` |
| DFF.9 | Phase doc + dev quickstart | `phase-ff.md`, `agent/README.md` |

## Out of scope (later phases)

- SQLite local persistence (Phase 9) — Home uses cloud; Code uses in-memory session
- Full Code sidebar pagination / local conversation DB
- Terminal, editor, git (Phases 12–14)
- Full policy engine, standing grants (Phase 7)
- Project cloud sync / add-to-project mutations

## Exit criteria

### Home

- [x] Signed-in user sees **real chat history** in Home sidebar (same API as web); mock titles shown only when offline/unauthenticated
- [x] Home launcher composer creates/opens a conversation and shows **streaming reply**
- [x] Selecting a history item loads messages and continues the thread
- [x] New messages sync to backend (debounced PUT)

### Code

- [x] "Open project" opens **folder picker**; picked path becomes active workspace
- [x] Real project appears in UI alongside mock demo entries
- [x] In Code Agent mode: user can ask to read a file → tool card → content in reply
- [x] User can ask to write a file → approval card → file created in picked folder
- [x] Cancel stops in-flight Code run

### Shared

- [x] Device auth works; free-tier default model auto-selected from `GET /agent/v1/models`
- [x] Errors visible in UI (not only console)
- [x] 5-minute dev setup documented in agent README

## Local dev checklist

1. Backend `PORT=3001` + agent JWT secrets + Prisma migrations
2. Web app running for device auth authorize page (`http://localhost:3000`)
3. Desktop env (optional overrides):
   - `VITE_EURY_AGENT_API_URL=http://localhost:3001/agent/v1`
   - `EURY_AGENT_GATEWAY_URL=http://localhost:3001/agent/v1/chat/stream`
4. Sign in → Home chat works → pick folder → Code agent works

## Relationship to code-old

| code-old behavior | FF implementation |
|-------------------|-------------------|
| `Session(kind="home")` + `ConversationSync` | `useHomeChatHistory` + `/conversations` API |
| `Session(kind="code")` local only | In-memory Code session; SQLite in Phase 9 |
| Home/Code pill in sidebar | `area` home vs code in `App.tsx` |
| Project picker | Tauri folder dialog (`workspace_pick_folder`) |

## Related

- [Roadmap overview](00-roadmap-overview.md)
- [Phase 11 — Model Routing and Cost](phase-11.md)
