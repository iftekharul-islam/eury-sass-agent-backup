# App Shell and Navigation

Spec-Version: 3.4.0

The desktop application has two persistent top-level areas: **Home** for general assistant work and account-level resources, and **Code** for repository-scoped coding-agent work. Code follows a focused two-region layout: compact project sidebar plus one conversation canvas. Run details are revealed in tool cards and dedicated panes, never held in a permanent dashboard rail. Implemented in [`agent/mockups/`](../../mockups/README.md).

## Top-level areas

| Area | Owns | Repository required |
|---|---|---|
| Home | New developer-assistance chat, Projects, Artifacts, Customize, account-level conversation history | No |
| Code | Workspaces, coding conversations, runs, plans, files, diffs, editor, terminal, memory, approvals | Yes for tools; the empty Code state opens or clones one |

The segmented **Home / Code** switch is the first control after native window controls and remains visible in both areas. Switching preserves each area's navigation stack, selected conversation, scroll position, and composer draft. It never cancels a run. A running Code task continues in the background and adds a status badge to Code while Home is active.

## Window layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● ● ● [Home|Code]  acme-api ▾  ⑂ main •       ⌘ Commands  Changes  Preview  Files  Agent ▾  ⌘K  ⚙ │ 40
├───────────────────────────┬──────────────────────────────────────────────────┤
│ ⊕ New                     │                                                  │
│ ◇ Artifacts               │        Conversation canvas ≤ 720px               │
│ ⚙ Customize               │        (turns, tool cards, approvals)            │
│ ▾ More                    │                                                  │
│                           │                                                  │
│ acme-api             + ⚙ │                                                  │
│ ● Fix flaky auth test     │                                                  │
│ ○ Add OAuth login         │                                                  │
│ ○ Billing refactor        │                                                  │
│   Runs · Plans · Memory   │                                                  │
│                           ├──────────────────────────────────────────────────┤
│                           │ Composer                                         │
├───────────────────────────┴──────────────────────────────────────────────────┤
│ ● Connected · BYOK · Sonnet 4 · 12.4k/200k · $0.04 · 1 changed · index ready  │ 24
└──────────────────────────────────────────────────────────────────────────────┘
```

| Region | Min | Default | Max | Notes |
|---|---|---|---|---|
| Window | 900 × 600 | 1360 × 860 | — | Size and position persisted per display |
| Title bar | 40 | 40 | 40 | Custom decorations, drag region |
| Code sidebar | 220 | 260 | 320 | Project switcher, project chats, primary navigation; collapses to a 56px icon rail |
| Conversation canvas | 600 | flex | — | Never collapsible; content is capped at 720px and centered |
| Composer | 56 | 56 | 200 | Grows with content, then scrolls |
| Status bar | 24 | 24 | 24 | Always visible |

Responsive rules: below 900px the Code sidebar auto-collapses to icons. Home has its own 220px sidebar. Both remember the user's explicit choice.

## Title bar

Left: traffic lights (macOS inset), Home/Code switch, then — in Code — workspace switcher and git branch with a dirty dot. Right in Code: a compact workspace-tools group for **Commands** (`⌘K`), **Changes** (`⌘⇧G`), **Browser preview**, and **Files**, followed by mode select, command palette button, and settings. Each tool opens its existing center pane (or a browser-preview pane when enabled); none adds a permanent rail. Home hides repository controls and keeps its own Chat mode. Double-click zooms; the drag region excludes every interactive child.

## Home sidebar

Fixed navigation order: **New**, **Projects**, **Artifacts**, **Customize**, then account-level conversations grouped by recency. The footer shows account, plan, sync state, and update status. Home chat supports workspace-independent developer assistance; it may attach files explicitly but has no implicit workspace, shell, git, or repository write access.

## Code sidebar

| Section | Content |
|---|---|
| Primary navigation | New, Artifacts, Customize, More |
| Project history | Every recently opened workspace has its own compact conversation group, headed by the project name and total history count |
| Conversation pagination | Show the newest 5 conversations per project initially. **Show more** reveals the next 5 for that project only; it repeats until that project has no remaining history. |
| Secondary workspace surfaces | Runs, Plans, Memory, Approvals, Files, and Browser preview open from the workspace-tools group, palette, or context actions — they do not dilute the project-history sidebar. |
| Footer | Active model with a click-to-change, account and sync state, Settings |

A conversation row shows its title, relative time, and — when relevant — a badge for pending approvals or an active run. The active conversation is highlighted even when it is in a non-active project group. Right-click gives rename, pin, export, and delete. Titles are generated from the first user message and are editable.

## Panes

The center region shows exactly one pane at a time. The conversation is the default; the others replace it and keep it one click away, so nothing important is ever hidden behind a floating window.

| Pane | Opened by | Purpose |
|---|---|---|
| Conversation | default, or clicking a conversation | Turns, tool cards, approvals, plans |
| Changes | `⌘⇧G`, a diff card, the changed-files list, the branch chip | File list plus unified or split diff, per-hunk apply and revert |
| Editor | clicking a file path or line reference | Read and edit one file with diff decorations |
| Terminal | `` ⌘` `` | A user-owned PTY, separate from agent commands |
| Browser preview | workspace-tools group or a local URL detected from a run | Sandboxed local preview; no browser login/session automation |
| Runs | sidebar or palette | History, timeline, checkpoints, replay |
| Plans | sidebar or palette | Plans in this workspace with status |
| Memory | sidebar | `EURY.md` hierarchy, entries, pending proposals |
| Approvals | sidebar, `⌘⇧A` | Pending requests and decision history |

Back and forward (`⌘⌥←` / `⌘⌥→`) move through pane history, so opening a diff and returning to the conversation is one keystroke ([keyboard](08-keyboard-and-command-palette.md)).

### Overlays and modals

Only three things float, and each is justified:

| Floating surface | Why |
|---|---|
| Command palette (`⌘K`) | Transient input over any pane |
| Popovers and dropdowns | Model select, mode select, row menus |
| Modal dialogs | Settings, workspace trust, critical-risk confirmation, quit with an active run, destructive settings confirmations |

Everything else is a pane. Settings is the intentional exception: a searchable modal over the active surface, so a user can alter a preference without losing their work context. Approvals in particular are **cards in the conversation**, not modals, so the surrounding context stays visible ([approval UX](05-approval-and-trust-ux.md)). The one automatic behavior allowed: when an approval arrives while the user is scrolled away or on another pane, the status bar and sidebar badge update and the conversation scrolls to the card when reopened. Focus is never stolen mid-typing.

## Run details without a permanent rail

The Code canvas stays focused. Status and progress therefore live where they are useful:

1. **Tool cards** show current state, duration, command or file target, output, and failure.
2. **Plan cards** show steps and active work.
3. **The status bar** shows connection, model, context, cost, changed-file count, indexing, and pending approvals.
4. **Dedicated panes** provide deep inspection: Changes, Runs, Plans, Memory, Approvals, Editor, and Terminal.

This avoids shrinking the conversation into a dashboard while retaining all run information one click away.

## Workspace model

A workspace is a root directory with a trust decision attached. Opening one records a `workspaces` row ([local data model](../04-specs/05-local-data-model.md)).

| Rule | Behavior |
|---|---|
| Untrusted project | Read-only tools only: no shell, no network, no MCP. Its `EURY.md` and MCP config are shown for review rather than applied ([trust](05-approval-and-trust-ux.md)) |
| Multiple windows | One workspace per window, separate run queues, one shared SQLite via WAL |
| Missing path | Row kept and marked unavailable, offered for removal |
| Switching workspace | Cancels nothing; a running run keeps streaming and stays reachable from Runs |
| No workspace | Code empty state offers Open folder and Clone repository; Home remains fully usable |

## Status bar

Left to right: connection state, plan mode (`BYOK` or `Managed`), active model, context usage, session cost, changed-file count, index state, pending-approval count, background job spinner. Every segment is clickable and opens the matching pane or popover. Nominal segments stay visible but muted; abnormal states (`offline`, `degraded`, `index failed`, `N pending`) take a status color and an icon ([offline modes](../02-architecture/06-offline-and-degraded-modes.md)).

## Composer

A rounded input at the bottom of the conversation. Toolbar row underneath: attach, `@` mention, `/` command, mode select, model select, send. `Enter` sends, `Shift+Enter` newlines, `⌘Enter` queues a steering message during a run. Drafts persist per conversation. Full behavior: [chat and streaming UX](03-chat-and-streaming-ux.md).

## Window and instance rules

| Concern | Behavior |
|---|---|
| Multiple windows | Allowed, `⌘⇧N` |
| Single instance | Enforced; a second launch focuses or opens a window in the running process |
| Close last window | macOS: the app stays alive; Windows and Linux: it quits after flushing SQLite and the audit queue |
| Quit with an active run | Modal listing running tools, then cancel and flush |
| Crash recovery | On next launch, offer to resume or revert interrupted runs ([checkpoints](../04-specs/12-checkpoint-and-rollback-spec.md)) |
| Deep link | `eury-agent://run?workspace=…&prompt=…` focuses a window and pre-fills the composer, never auto-sends |

## Empty and error states

| Situation | State |
|---|---|
| No workspaces yet | Code empty state with Open folder and Clone repo; Home still offers workspace-independent developer assistance |
| Empty conversation | Mode explainer and three suggested prompts derived from the repository |
| No runs | One line explaining what a run is |
| No pending approvals | "Nothing waiting", with a link to Permissions |
| Index failed | Reason, affected paths, Retry; the conversation stays usable |
| Offline | Status bar shows `offline`; local features continue and cloud-dependent ones name what is unavailable |

Every error state names the failure, its `EURY_*` code, and exactly one recovery action ([error taxonomy](../04-specs/15-error-taxonomy.md)).

## Startup

Cold start renders the shell within 400 ms and never blocks on the index or the network. Opening a workspace shows progressive index state (`queued → scanning → ready`) in the status bar and workspace menu without gating the composer; retrieval quality degrades gracefully until ready ([latency budget](../02-architecture/05-latency-budget.md)).

## Related documents

- [Visual language](00-visual-language.md)
- [Design system](01-design-system.md)
- [Chat and streaming UX](03-chat-and-streaming-ux.md)
- [Editor, terminal, preview](06-editor-terminal-preview.md)
- [Keyboard and command palette](08-keyboard-and-command-palette.md)
- [Mockups](../../mockups/README.md)
