# Phase 13 — Editor and Explorer

Spec-Version: 1.1.0

**Track:** C — Product surfaces · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

A CodeMirror-based editor with live write preview, plus a file explorer and search, so users can review and steer the agent's edits in place.

## Why this phase exists here

Live write preview was the deprecated app's best idea. Reproducing it properly requires the diff engine from Phase 6 and the approval flow from Phase 7 to already exist.

## In scope

- CodeMirror 6 with 20 bundled languages and plain fallback
- Read-only mode for untrusted workspaces; edit and save through the sandbox
- Live write preview decorations for pending agent writes, with hunk navigation
- Hunk-level apply and skip, reporting skipped hunks back to the agent
- External and agent change detection with safe reload behavior
- File explorer with git status decorations (statuses land fully in Phase 14)
- Workspace search and replace with preview
- Encoding and EOL display and preservation; large and binary file handling

## Feature IDs

`F-050`, `F-051`, `F-052`, `F-056`

## Out of scope

- LSP features (deferred; see open questions)
- Debugging

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D13.1 | Editor with save through the sandbox path guard | [editor/terminal/preview](../05-ui/06-editor-terminal-preview.md) |
| D13.2 | Live write preview overlay with pending-hunk gutter markers | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D13.3 | Hunk apply/skip with structured feedback to the agent | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D13.4 | Changes pane with multi-file diff review for a whole run | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D13.5 | File explorer with lazy loading for large trees | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D13.6 | Workspace search and replace with per-match preview | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D13.7 | Encoding/EOL preservation with visible indicators | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D13.8 | Cross-surface actions: ask about selection, open at line, insert at cursor | [keyboard](../05-ui/08-keyboard-and-command-palette.md) |

## Key decisions and design notes

- CodeMirror over Monaco: roughly a seventh of the bundle size, faster mount, and a decoration API that suits diff overlays.
- Editing a file with a pending preview dismisses and invalidates that write rather than merging blindly.
- Partial hunk application is allowed and is reported to the agent, so it can react instead of assuming success.
- The editor is deliberately not an IDE; language intelligence stays out of scope for v1.

## Contracts touched

- Pending-write preview IPC commands and events
- Hunk apply/skip result reported into the tool result

## Dependencies

- Phase 6 (diffs and write tools)
- Phase 7 (approvals)
- Phase 3 (pane shell)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Preview desynchronizing from the file | Wrong edits applied | Content hash checked at apply time; mismatch invalidates the pending write |
| Large file performance | Freeze | Size thresholds, virtualization, highlight disabled beyond limits |
| Encoding mishandling | Corrupted files | Round-trip tests across encodings and EOL styles |
| Explorer on huge repos | Slow UI | Lazy loading, virtualized tree, throttled watchers |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Decoration mapping, hunk selection state, dirty-buffer logic |
| Integration | Agent write → preview → partial apply → save → verify on disk |
| Property | Apply/skip combinations produce the expected file content |
| Performance | 1k, 100k-line files; 50k-file explorer tree |
| A11y | Keyboard hunk navigation and apply |

## Metrics and targets

| Metric | Target |
|---|---|
| Open 1000-line file | < 120 ms p95 |
| Open 100k-line file | < 600 ms p95 |
| Preview decoration render | < 50 ms |
| Keystroke latency while streaming | < 16 ms p95 |

## Exit criteria

- [ ] Live write preview works with hunk-level apply and skip
- [ ] Skipped hunks are reported to the agent as not applied
- [ ] Encoding and EOL round trips are lossless
- [ ] Explorer and search perform acceptably on a 50k-file repo
- [ ] Untrusted workspaces are read-only in the editor

## Deferred from this phase

- LSP integration (open question Q07)
- Git status detail (Phase 14)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
