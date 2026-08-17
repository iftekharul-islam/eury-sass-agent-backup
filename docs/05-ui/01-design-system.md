# Design System

Spec-Version: 3.4.0

Tailwind 4 + CSS custom properties for a dense desktop application: 13px base rather than 16px, 4px grid, light-first. Proportional UI font for chrome and prose, monospace **only** for code, paths, commands, and diffs, per [visual language](00-visual-language.md).

The token values below are implemented literally in [`agent/mockups/index.html`](../../mockups/README.md), which is the visual source of truth. If a token changes, it changes in the mockup and here in the same commit.

## Token architecture

Three layers. UI code references **semantic** tokens only; never a raw hex, never a palette token.

```
palette   --ember-500: #e08a4c          (fixed hues)
   ↓
semantic  --color-accent: var(--ember-500)
   ↓
component --btn-primary-bg: var(--color-accent)
```

Tokens are declared on `:root` and overridden under `[data-theme="light"]` and `[data-accent="<name>"]`, so theme and accent switch by changing one attribute on `<html>` with no React re-render.

## Semantic tokens

Light is the default reference surface. The neutrals are warm rather than blue-grey, which keeps a dense interface from feeling clinical; dark remains a complete alternate theme.

| Token | Dark | Light (default) | Use |
|---|---|---|---|
| `--color-bg` | `#1a1918` | `#ffffff` | Conversation and pane background |
| `--color-bg-sunken` | `#141312` | `#f7f6f4` | Sidebar, title bar, secondary pane list |
| `--color-bg-elevated` | `#232120` | `#ffffff` | Cards, popovers, modals |
| `--color-bg-inset` | `#100f0f` | `#f2f0ed` | Code blocks, inputs, terminal output |
| `--color-bg-hover` | `#2a2826` | `#f0eeeb` | Row and button hover |
| `--color-bg-active` | `#332f2c` | `#e6e3df` | Pressed, selected row |
| `--color-fg` | `#f0ede8` | `#1c1a19` | Primary text |
| `--color-fg-muted` | `#a5a09a` | `#5f5b56` | Secondary text, metadata |
| `--color-fg-subtle` | `#78736d` | `#8b867f` | Timestamps, hints, placeholders |
| `--color-fg-faint` | `#514c47` | `#b5b0a9` | Disabled text, empty glyphs |
| `--color-border` | `#302d2a` | `#e5e2dd` | Card and pane hairlines |
| `--color-border-strong` | `#454039` | `#cbc7c0` | Focused and active edges |
| `--color-accent` | `#e08a4c` | `#c06a26` | Primary button, focus ring, active nav |
| `--color-accent-hover` | `#e89a60` | `#a95a1c` | Primary button hover |
| `--color-accent-fg` | `#1a1918` | `#ffffff` | Text on accent |
| `--color-accent-subtle` | `#3a2a1c` | `#fbf0e6` | Accent-tinted row background |
| `--color-success` | `#6fb56f` | `#2f7a35` | Applied, passed, ready |
| `--color-warning` | `#d9a441` | `#9a6b0f` | Needs approval, degraded |
| `--color-danger` | `#dc6d6d` | `#b3302f` | Denied, failed, destructive |
| `--color-info` | `#6f9bcd` | `#2f6191` | Neutral status, read-only |
| `--color-diff-add-bg` | `#16261a` | `#e7f6e9` | Diff added line |
| `--color-diff-del-bg` | `#2a1717` | `#fdeaea` | Diff removed line |
| `--color-diff-add-fg` | `#8ed48e` | `#1f5c25` | Diff added text and gutter |
| `--color-diff-del-fg` | `#e79191` | `#8c1f1f` | Diff removed text and gutter |
| `--color-diff-word-add` | `#2c5233` | `#bfe8c4` | Word-level added mark |
| `--color-diff-word-del` | `#5a2626` | `#f6bdbd` | Word-level removed mark |

## Accent palette

Five selectable accents. Ember is the default; the rest exist because accent is the one thing users reliably want to personalize.

| Name | Value |
|---|---|
| Ember (default) | `#e08a4c` |
| Teal | `#3fb3a2` |
| Blue | `#5f9de8` |
| Violet | `#a78bda` |
| Rose | `#dd7a92` |

Stored in `settings` as `ui.accent`, applied as `data-accent` on `<html>`.

## Status colors

Security and status state must never be carried by accent, because accent is user-configurable, and never by color alone, because of colorblind users and forced-colors mode. Every status carries an icon and a text label.

| State | Token | Icon | Label |
|---|---|---|---|
| Read-only, safe | `--color-info` | eye | Read |
| Running | `--color-accent` | spinner | Running |
| Pending approval | `--color-warning` | alert-triangle | Needs approval |
| Applied, succeeded | `--color-success` | check | Applied |
| Denied by policy | `--color-danger` | shield-x | Denied |
| Failed | `--color-danger` | x | Failed |
| Cancelled | `--color-fg-subtle` | circle-slash | Cancelled |
| Queued | `--color-fg-faint` | circle | Queued |

## Typography

Two families, with a strict division of labor. Using monospace for prose is the mistake that makes an app look like a terminal.

| Role | Family | Size / line-height | Weight |
|---|---|---|---|
| Body, assistant prose | UI | 13 / 20 | 400 |
| Body large (empty states) | UI | 14 / 22 | 400 |
| Small, metadata | UI | 11.5 / 16 | 400 |
| Label, section header | UI | 11 / 14 | 600, uppercase, tracking 0.06em |
| Button | UI | 12.5 / 16 | 500 |
| Sidebar item | UI | 12.5 / 18 | 400 |
| H1 (launcher) | UI | 22 / 28 | 600 |
| H2 (pane title) | UI | 15 / 22 | 600 |
| H3 (card title) | UI | 13 / 18 | 600 |
| Code, diff, terminal | Mono | 12.5 / 19 | 400 |
| Inline code, path, command | Mono | 12 / 18 | 400 |

Font stacks, bundled as WOFF2 inside the app:

- **UI**: `Inter`, then `-apple-system`, `Segoe UI`, `system-ui`, `sans-serif`.
- **Mono**: `JetBrains Mono`, then `ui-monospace`, `SFMono-Regular`, `Menlo`, `Consolas`, `monospace`.

CSP forbids remote font origins, so there is **no runtime font fetch** ([threat model](../03-security/01-threat-model.md)).

## Spacing, radius, elevation

- Spacing: 4px base — `1=4, 2=8, 3=12, 4=16, 5=20, 6=24, 8=32, 10=40`.
- Card padding: 12px compact, 16px standard. Card gap in the conversation: 8px. Turn gap: 20px.
- Radius: `sm=6` chips and badges, `md=8` buttons and inputs, `lg=10` cards and panels, `xl=14` modals, `full` avatars.
- Elevation: three levels only.
  - `flat` — hairline border, no shadow. Cards and panels use this.
  - `popover` — `0 6px 20px rgb(0 0 0 / 0.28)` plus border. Dropdowns, tooltips, palette.
  - `modal` — `0 20px 60px rgb(0 0 0 / 0.45)` plus a 40% scrim. Trust and critical-risk dialogs only.

In light theme, shadows drop to a third of their opacity and borders carry the separation instead.

## Layout constants

| Constant | Value |
|---|---|
| Title bar | 40px |
| Home/Code switch | 72px minimum per segment, always visible |
| Home sidebar | 220px default, 200–300 resizable |
| Code sidebar | 260px default, 220–320 resizable, 56px collapsed |
| Conversation content column | max 720px, centered |
| Run inspection | Tool cards plus dedicated center panes; no persistent right rail |
| Composer | 56px min, grows to 200px then scrolls |
| Status bar | 24px |
| Window min | 900 × 600 |
| Window default | 1360 × 860 |

## Component inventory

Shared with `frontend/` where the semantics match: `Button`, `IconButton`, `Input`, `Textarea`, `Select`, `Switch`, `Checkbox`, `Dialog`, `Popover`, `DropdownMenu`, `Tooltip`, `Tabs`, `Toast`, `Badge`, `Spinner`, `Skeleton`, `ScrollArea`, `Kbd`, `ResizablePanel`.

Desktop-specific:

| Component | Purpose | Key props |
|---|---|---|
| `AreaSwitcher` | Persistent Home/Code segmented switch | `value`, `homeStatus`, `codeStatus` |
| `TitleBar` | Custom decorations, area, workspace and mode controls | `area`, `workspace`, `branch`, `mode` |
| `HomeSidebar` | New, Projects, Artifacts, Customize, developer-assistance history | `items`, `activeId`, `account` |
| `SettingsModal` | Searchable 960px settings modal over the active surface | `section`, `query`, `onClose` |
| `SettingsNav` | Profile, Desktop app, and Code setting categories | `groups`, `selectedSection` |
| `Sidebar` | Code primary nav, project header, project conversation list | `project`, `items`, `collapsed` |
| `WorkspaceSwitcher` | Open, switch, recent projects | `workspaces`, `value` |
| `ConversationList` | Grouped by day, searchable | `items`, `activeId` |
| `UserTurn` | User message with attachments | `text`, `attachments`, `onEdit` |
| `AssistantTurn` | Streaming prose, model badge | `markdown`, `model`, `streaming` |
| `ReasoningDisclosure` | Collapsed thinking summary | `text`, `durationMs` |
| `ToolCard` | One tool invocation, expandable | `tool`, `status`, `durationMs`, `result` |
| `ToolOutput` | Virtualized stdout/stderr | `toolCallId`, `stream` |
| `DiffCard` | Diff inside the conversation | `patch`, `view`, `onApply` |
| `DiffViewer` | Full pane: file list, unified or split, per-hunk | `files`, `activeFile`, `view` |
| `ApprovalCard` | Permission request with buttons | `request`, `risk`, `scopes`, `onDecide` |
| `PlanCard` | Plan steps and build actions | `plan`, `activeStepId` |
| `RunHeader` | Status, elapsed, progress, stop | `run`, `onStop` |
| `Composer` | Input, attach, mode, model, send | `mode`, `model`, `draft`, `onSend` |
| `ModeSelect` | Chat, Ask, Plan, Agent, Build | `value`, `allowedModes` |
| `ModelSelect` | Provider and model with cost hint | `models`, `value`, `disabledReason` |
| `StatusBar` | Connection, plan, cost, index | `state` |
| `CommandPalette` | Fuzzy commands, files, runs | `commands`, `query` |
| `TrustDialog` | Workspace trust decision | `workspace`, `onDecide` |
| `CheckpointChip` | Restore affordance on a turn | `checkpointId`, `onRestore` |
| `TokenMeter` | Context usage bar | `used`, `window` |
| `EmptyState` | Illustration, explanation, one action | `title`, `body`, `action` |

Every component ships with a fixture used by visual regression tests, and each fixture corresponds to a screen or state in the mockup ([test strategy](../08-quality/01-test-strategy.md)).

## Buttons

| Variant | Use | Style |
|---|---|---|
| `primary` | The safe, expected action | Accent fill |
| `secondary` | Alternative actions | Border, transparent fill |
| `ghost` | Toolbar and row actions | No border until hover |
| `danger` | Destructive confirmation only | Border and danger-colored text; filled danger is used only inside a critical-risk modal |

A destructive action is never `primary`. In an approval card the primary button is the narrowest allow, deny is `secondary` and always leftmost, and scope broadening is a `secondary` split button so widening is a deliberate, separate act ([approval UX](05-approval-and-trust-ux.md)).

## States

Every interactive component defines all seven: `default`, `hover`, `active`, `focus-visible`, `disabled`, `loading`, `error`. Focus is a 2px `--color-accent` ring at 2px offset and is never removed.

## Motion

| Interaction | Duration | Easing |
|---|---|---|
| Panel collapse and resize | 140 ms | `ease-out` |
| Popover and dropdown enter | 110 ms | `ease-out` |
| Modal enter | 160 ms | `ease-out`, scale 0.98 → 1 |
| Card expand | 130 ms | `ease-out` |
| Toast enter and exit | 160 ms | `ease-out` |
| Spinner | 900 ms loop | linear |
| Theme change | 0 ms | — |
| Streaming text | none | — |

Streaming text has no per-token animation: it costs frames and hurts readability. All motion is disabled under `prefers-reduced-motion` ([accessibility](07-accessibility-and-i18n.md)).

## Density

`comfortable` (default, 13px) and `compact` (12px, row heights reduced 4px, card padding 12px). Stored as `ui.density`. Font size is adjustable from 11px to 18px with `⌘+` / `⌘-`; layout constants scale with it.

## Icons

`lucide-react`, stroke 1.5, `currentColor`. 14px in dense rows and cards, 16px in the sidebar and toolbars, 18px in pane headers. Tool icons are fixed per tool class so users learn shapes rather than reading labels: terminal for execute, file-text for read, pencil for edit, plus for create, globe for network, git-branch for git, plug for MCP.

## Do not

- Do not introduce a color outside the token table.
- Do not use monospace for prose, labels, or buttons.
- Do not use accent color for risk or status.
- Do not style a destructive action as `primary`.
- Do not animate per token during streaming.
- Do not add a shadow outside the three defined levels, or any gradient.
- Do not use `dangerouslySetInnerHTML` outside the sanitized markdown renderer.

## Related documents

- [Visual language](00-visual-language.md)
- [App shell and navigation](02-app-shell-and-navigation.md)
- [Approval and trust UX](05-approval-and-trust-ux.md)
- [Accessibility and i18n](07-accessibility-and-i18n.md)
- [Mockups](../../mockups/README.md)
