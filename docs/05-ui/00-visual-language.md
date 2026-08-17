# Visual Language

Spec-Version: 2.2.0

Eury Agent is a **native-feeling desktop application** that has Claude Code's agent capabilities. We clone the *capabilities and interaction guarantees*, not the terminal. Nothing in this product looks like a CLI.

This document is the north star: it states what we take from Claude Code, how a desktop GUI improves on it, and it holds the wireframe for every screen so the mockup and the implementation cannot drift apart.

The clickable mockup that implements this document lives in [`agent/mockups/`](../../mockups/README.md) and is a **Phase 0, day-one deliverable**. No UI code is written before it exists.

## What we clone: the capabilities

| Capability | Our desktop equivalent |
|---|---|
| Agentic loop over real tools | Same loop, embedded in the Rust core ([agent runtime](../04-specs/01-agent-runtime-spec.md)) |
| Read, write, edit, search, shell, git, web | Full tool catalog ([tool catalog](../04-specs/02-tool-catalog-spec.md)) |
| Visible tool activity | Tool cards in the conversation, expandable to full output |
| Permission prompts before risky actions | Approval cards with real buttons, and a modal only for critical risk |
| Diffs before writes | A proper diff viewer with syntax highlighting, unified and split views, per-hunk apply |
| Plan then execute | A plan panel backed by a real markdown file in the repo ([plan format](../04-specs/11-plan-format-spec.md)) |
| Todo tracking during long tasks | A task checklist in the active plan card and the Runs pane |
| Sub-agents for large work | Sub-agent runs with their own collapsible sections ([multi-agent](../04-specs/13-multi-agent-spec.md)) |
| Project instructions files | `EURY.md` hierarchy with a memory manager UI ([memory](../04-specs/08-memory-spec.md)) |
| MCP servers | Managed in Settings with per-server trust ([MCP](../04-specs/10-mcp-integration-spec.md)) |
| Undo what the agent did | Checkpoints with a visual restore preview ([checkpoints](../04-specs/12-checkpoint-and-rollback-spec.md)) |
| Slash commands and file mentions | `/` and `@` in the composer, plus a command palette |

## What a desktop GUI does better

This is the reason the product exists. A terminal cannot do any of the following, and each one is a feature we ship rather than an afterthought.

| Capability | Why it matters |
|---|---|
| Real diff viewer — syntax highlighting, unified or split, word-level marks, per-hunk apply and revert | Reviewing agent edits is the single most common task. Doing it well is our biggest advantage |
| Persistent conversation sidebar with search across all history | Terminal scrollback is lossy; ours is SQLite-backed and searchable ([local data model](../04-specs/05-local-data-model.md)) |
| Inspectable run details without a dashboard rail | Tool cards provide immediate state; Changes, Runs, Plans, Memory, and Approvals provide depth in the center pane |
| Clickable file paths that open a real editor pane | Navigate from a tool result to the code in one click |
| Images in and out — paste a screenshot, view generated charts | Multimodal work is first-class |
| Rich approval cards with the diff or command inline, and buttons | Nothing to memorize, nothing to type; the safe choice is the visually obvious one |
| Multiple workspaces in multiple windows | Real parallel work instead of tabs of terminals |
| Native notifications, deep links, menu bar, and auto-update | Desktop citizenship |
| Full accessibility: focus rings, ARIA semantics, screen reader support, zoom | Terminal UIs are typically poor at all of these ([accessibility](07-accessibility-and-i18n.md)) |

## What we never copy

Another product's brand, wordmark, icon, or exact palette. The valuable part is the interaction model, which is freely learnable; trade dress is not. Our accent is a warm ember tone chosen for this UI, and it is one token any organization can re-theme.

We also never copy permissive defaults. Deny-by-default, typed confirmation for critical operations, and a hard workspace boundary stay exactly as specified in [approval and trust UX](05-approval-and-trust-ux.md).

## Design principles

1. **A desktop app, not a terminal in a window.** Proportional UI font for chrome and prose, monospace only for code, paths, commands, and diffs.
2. **Home and Code are separate product areas.** Home handles workspace-independent developer assistance, projects, artifacts, and customization; Code handles repository-scoped agent work. The switch is always visible.
3. **Every agent action is a first-class object.** A tool call is a card you can expand, re-run, copy, or roll back — not a line of log text.
4. **The safe choice is the obvious choice.** Approval cards put the payload in full view and never make the destructive option the prettiest button.
5. **Show the work.** Tool activity, retrieved context, token cost, and changed files are always inspectable, never hidden behind a spinner.
6. **Dense, not cramped.** 13px base, 4px grid, generous hit targets. More signal per screen than a chat app, calmer than an IDE.
7. **Keyboard parity.** Everything reachable by mouse is reachable by keyboard, but nothing *requires* memorizing a keystroke.

## Screen wireframes

These are normative layouts. The mockup implements each one; a UI pull request that changes a layout updates the wireframe here in the same commit.

### 1. Home — general assistant and launch surface

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● ● ●   [⌂ Home] [</> Code]                                      ⌘K  ⚙     │
├───────────────────┬──────────────────────────────────────────────────────────┤
│ ⊕ New             │                  What can I help you with?               │
│ ▣ Projects        │                                                          │
│ ◇ Artifacts       │      ┌────────────────────────────────────────────┐      │
│ ⚙ Customize       │      │ Ask Eury anything…                 Send →  │      │
│                   │      └────────────────────────────────────────────┘      │
│ CHAT              │                                                          │
│ • Usage limits    │      [ </> Open Code ] [ Open project ] [ Artifact ]     │
│ • Claude API      │                                                          │
│ • Email delivery  │      RECENT PROJECTS                                     │
│                   │      ▪ acme-api      main       2h ago       ● indexed   │
│                   │      ▪ eury-saas     feat/…     1d ago       ● indexed   │
│ ────────────────  │                                                          │
│ Manna · Free      │                                                          │
└───────────────────┴──────────────────────────────────────────────────────────┘
```

Home conversations are account-level and may work without a repository. Opening a recent project or selecting **Code** moves to the last Code workspace without losing the Home draft.

### 2. Code — agent run

Two regions: a compact project sidebar and one centered, light conversation canvas. This follows the supplied Claude desktop reference: agent work reads as a spacious transcript, activity is presented as compact lines, and only expanded command output or a diff receives a container.

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│ ● ● ● [Home|Code] acme-api ▾ ⑂ main •       ⌘  Changes  Preview  Files  Agent ▾ │
├────────────────────────────┬─────────────────────────────────────────────────────┤
│ ⊕ New                      │     You                                    09:41  │
│ ◇ Artifacts                │     Fix the flaky auth session test.               │
│ ⚙ Customize                │                                                     │
│ ▾ More                     │     ⬢ Eury · Sonnet 4 · Agent                       │
│                            │     I'll reproduce the failure first…              │
│ acme-api              + ⚙ │     ▾  ⌘ Terminal  pnpm test session   3.1s  ✗       │
│ ● Fix flaky auth test      │       ┌─────────────────────────────────────────┐   │
│ ○ Add OAuth login          │       │ FAIL … refreshes before expiry           │   │
│ ○ Billing refactor         │       └─────────────────────────────────────────┘   │
│   Runs · Plans · Memory    │     ▸  ▢ Read  session.test.ts       12ms   ✓        │
│   Approvals                │     The timer advances before the promise…          │
│ ────────────────────────── │     ▸  ✎ Edit  session.test.ts       +4 −2  ✓       │
│ ◐ Sonnet 4 · BYOK          │     ▸  ⌘ Terminal  pnpm test session     Running    │
│ ⚙ Settings                 │                                                     │
│                            │     ⟳ Verifying the fix…                            │
│                            ├─────────────────────────────────────────────────────┤
│                            │  Reply to Eury…                  Agent ▾  Send →   │
├────────────────────────────┴─────────────────────────────────────────────────────┤
│ ● Connected · Sonnet 4 · 12.4k/200k · $0.04 · 1 changed · index ready            │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 3. Approval card — command

Inline in the conversation, with real buttons. The run pauses; the rest of the app stays usable.

```
┌ ⚠ Approval required ─────────────────────────── elevated risk ──┐
│ ⌘ Run command                                                   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ pnpm migrate:reset                                          │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ Directory   ~/dev/acme-api                                      │
│ Reason      Step 3 of plan: reset local DB before re-seeding     │
│ Risk        Matches destructive pattern: migrate:reset           │
│                                                                 │
│ ┌───────────┐ ┌───────────────┐ ┌──────────────────────────┐    │
│ │   Deny    │ │  Allow once   │ │ Allow for this session ▾  │    │
│ └───────────┘ └───────────────┘ └──────────────────────────┘    │
│                                          Esc to deny            │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Approval card — diff

```
┌ ⚠ Approval required ───────────────────────────── medium risk ──┐
│ ✎ Edit file    src/auth/session.test.ts              +4 −2      │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 41    it('refreshes before expiry', async () => {           │ │
│ │ 42 -    vi.advanceTimersByTime(1000)                        │ │
│ │ 43 -    await session.refresh()                             │ │
│ │ 42 +    const pending = session.refresh()                   │ │
│ │ 43 +    await vi.advanceTimersByTimeAsync(1000)             │ │
│ │ 44 +    await pending                                       │ │
│ │ 45      expect(store.token).toBe('new')                     │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ ⌄ Open in editor   ⧉ Copy diff                                  │
│                                                                 │
│ [ Deny ]  [ Apply once ]  [ Allow edits in this file ▾ ]         │
└─────────────────────────────────────────────────────────────────┘
```

### 5. Diff review — full pane

Opened from a tool card, the changed-files list, or `⌘⇧G`. Replaces the conversation area rather than covering it, so the conversation is one click away.

```
┌───────────────────┬──────────────────────────────────────────────────────────────┐
│ CHANGES  3 files  │  src/auth/session.test.ts            +4 −2   [Unified|Split] │
│ +48 −12           ├──────────────────────────────────────────────────────────────┤
│ ▪ session.test.ts │  41    it('refreshes before expiry', async () => {           │
│      +4 −2   ✓    │  42 -    vi.advanceTimersByTime(1000)                        │
│ ▪ oauth.ts        │  43 -    await session.refresh()                             │
│      +41     ✓    │  42 +    const pending = session.refresh()                   │
│ ▪ schema.prisma   │  43 +    await vi.advanceTimersByTimeAsync(1000)             │
│      +3 −10  ⧗    │  44 +    await pending                                       │
│                   │  45      expect(store.token).toBe('new')                     │
│                   ├──────────────────────────────────────────────────────────────┤
│ [ Revert all ]    │  Hunk 1 of 1   [ Revert hunk ]  [ Open in editor ]           │
└───────────────────┴──────────────────────────────────────────────────────────────┘
```

### 6. Plan

```
┌ ⬗ Plan · Add OAuth login ──────────────── draft · 5 steps · ~90m ──┐
│ .eury/plans/2026-08-16-add-oauth-login-a1b2c3.md            ⧉ ✎    │
│                                                                    │
│ 1 ▢ Add OAuthAccount model and migration      medium   ~10m   2 f. │
│ 2 ▢ Implement the provider client             low      ~25m   3 f. │
│ 3 ▢ Wire callback routes                      medium   ~20m   2 f. │
│ 4 ▢ Add session linking                       low      ~15m   1 f. │
│ 5 ▢ Tests and docs                            low      ~20m   4 f. │
│                                                                    │
│ [ Build step by step ]  [ Build all ]  [ Edit plan ]  [ Discard ]  │
└────────────────────────────────────────────────────────────────────┘
```

### 7. Command palette

```
┌──────────────────────────────────────────────────────────────┐
│ ⌕ compact                                                    │
├──────────────────────────────────────────────────────────────┤
│ COMMANDS                                                     │
│ ▸ /compact        Summarize older turns to free context       │
│   /clear          Start a fresh conversation           ⌘⇧K    │
│ SETTINGS                                                     │
│   Context         Configure the compaction threshold          │
└──────────────────────────────────────────────────────────────┘
```

### 8. Workspace trust — modal

The one place a modal is correct: nothing else can happen until the question is answered.

```
┌ Trust this project? ─────────────────────────────────────────┐
│ ~/dev/unknown-repo                                           │
│ 1,284 files · github.com/acme/unknown-repo                   │
│                                                              │
│ Found in this project:                                       │
│   EURY.md · .eury/policy.json · 3 MCP server configs         │
│                                                              │
│ Until you trust it, Eury runs read-only: no shell, no        │
│ network, no MCP, and the project's own config is shown for   │
│ review rather than applied.                                  │
│                                                              │
│              [ Open read-only ]  [ Trust project ]           │
└──────────────────────────────────────────────────────────────┘
```

### 9. Runs

```
┌───────────────────┬──────────────────────────────────────────────────────────────┐
│ RUNS              │  Fix the flaky auth session test                             │
│ ▸ Today           │  ✓ Completed · Agent · Sonnet 4 · 11.4s · $0.04              │
│   ✓ Flaky auth    ├──────────────────────────────────────────────────────────────┤
│   ✓ Add oauth     │  TIMELINE                                                    │
│   ⚠ Billing       │  0.0s  ▸ Turn 1                                              │
│   ⊘ Update deps   │  0.4s  ▸ ⌘ pnpm test session              3.1s  ✗            │
│   Yesterday       │  3.6s  ▸ ▢ Read session.test.ts           12ms ✓            │
│   ✓ Session flow  │  4.1s  ▸ ✎ Edit session.test.ts           +4 −2 ✓  ⟲ restore │
│   ⊗ Vitest        │  8.0s  ▸ ⌘ pnpm test session              3.2s  ✓            │
│                   │                                                              │
│                   │  [ Open conversation ]  [ View changes ]  [ Revert run ]      │
└───────────────────┴──────────────────────────────────────────────────────────────┘
```

### 10. Settings modal

```
            ┌──────────────────────────────────────────────────────┐
            │ ⌕ Search settings                                ×  │
            ├──────────────────┬───────────────────────────────────┤
            │ PROFILE          │ Profile                           │
            │ ▸ General        │                                   │
            │   Account        │ Avatar                         M │
            │   Privacy        │ Full name       [ Manna         ] │
            │   Billing        │ Call me         [ Manna         ] │
            │                  │ Work            [ Select       ▾ ] │
            │ DESKTOP APP      │                                   │
            │   General        │ Instructions for Eury             │
            │   Extensions     │ [                                ] │
            │   Developer      │                                   │
            │                  │ PREFERENCES                       │
            │ CODE             │ Appearance   [ system ][light][dark]│
            │   Permissions    │ Chat font                  System ▾│
            │   Memory         │ Motion       Reduced when system… ▾│
            │   MCP servers    │                                   │
            └──────────────────┴───────────────────────────────────┘
```

Settings is a searchable, 960px modal over the active Home or Code area. It preserves the underlying location and draft. Categories are grouped as Profile, Desktop app, and Code. `Permissions`, `Memory`, and `MCP servers` stay under Code; organization-locked controls explain the applicable policy rather than appearing editable. `Esc`, the close button, and `⌘,` close the modal and restore focus to the invoking control.

## Layout rules

| Rule | Value |
|---|---|
| Code sidebar | 260px default, 220–320 resizable, collapsible to icons at 56px |
| Conversation | Flex, content column capped at 720px and centered within it |
| Run detail | Tool and plan cards by default; Changes, Runs, Plans, Memory, Approvals, Editor, and Terminal open as center panes |
| Below 900px | Sidebar collapses to icons |
| Grid | 4px spacing base; card padding 12–16px; card gap 8px |
| Radius | Cards and inputs 10px, buttons 8px, chips 6px, avatars full |
| Elevation | Flat by default; one shadow level for popovers, one for modals |

## Anti-patterns

| Do not | Reason |
|---|---|
| Make the UI look like a terminal | This is a desktop app. Monospace belongs in code, paths, and diffs only |
| Require keystrokes to answer prompts | Buttons first; shortcuts are an accelerator, never the only route |
| Use chat bubbles with avatars on both sides | Wastes horizontal space; the assistant turn needs full width for diffs and cards |
| Hide tool activity behind a single spinner | The work is the product; show it |
| Put the destructive action in the primary button style | The safe option is always the visually dominant one |
| Animate per token while streaming | Costs frames, hurts readability |
| Use color as the only carrier of risk or status | Fails accessibility ([accessibility](07-accessibility-and-i18n.md)) |
| Add decorative gradients or glass effects | Nothing decorative survives the first week of real use |

## Related documents

- [Design system](01-design-system.md)
- [App shell and navigation](02-app-shell-and-navigation.md)
- [Chat and streaming UX](03-chat-and-streaming-ux.md)
- [Tool activity and diff UX](04-tool-activity-and-diff-ux.md)
- [Approval and trust UX](05-approval-and-trust-ux.md)
- [Mockups](../../mockups/README.md)
