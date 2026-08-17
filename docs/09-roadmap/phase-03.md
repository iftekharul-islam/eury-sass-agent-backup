# Phase 03 — Desktop Shell

Spec-Version: 1.1.0

**Track:** B — Core runtime · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

A production-quality application shell: window management, navigation, theming, settings, and the IPC plumbing that every later feature rides on.

## Why this phase exists here

The shell defines the performance ceiling and the IPC discipline. Building it before the agent means streaming, approvals, and diffs land in a structure that already handles focus, persistence, and window lifecycle correctly.

## In scope

- Tauri window configuration, custom titlebar, single-instance enforcement, multi-window
- Persistent Home/Code top-level switch with independent navigation and draft state
- Conversation sidebar, center pane, and collapsible live context panel
- Design tokens, light/dark themes, five accents, density modes
- Settings surface with persisted preferences (file-backed until Phase 9)
- Command palette with the command registry and `when`-clause evaluation
- IPC command and event scaffolding with typed contracts and golden fixtures
- Error boundary, toast system, empty/loading states, deep-link handler

## Feature IDs

`F-001`, `F-002`, `F-003`, `F-004`, `F-005`, `F-007`, `F-008`

## Out of scope

- Agent runs
- Editor and terminal
- SQLite persistence

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D3.1 | Token system and theme switching with no re-render | [design system](../05-ui/01-design-system.md) |
| D3.1a | Home area and persistent Home/Code switch with independent state restoration | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D3.2 | App shell: sidebar, center pane, collapsible context panel, status bar | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D3.3 | Window state persistence per display; single-instance focus behavior | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D3.4 | Command registry, palette with prefixes, rebinding UI | [keyboard](../05-ui/08-keyboard-and-command-palette.md) |
| D3.5 | Typed IPC command layer with validation and golden fixtures | [IPC spec](../04-specs/04-ipc-command-spec.md) |
| D3.6 | Event channel scaffolding with a frame-batched consumer hook | [event protocol](../04-specs/03-event-protocol-spec.md) |
| D3.7 | Component library: buttons, inputs, cards, dialogs, panes, toasts, skeletons | [design system](../05-ui/01-design-system.md) |
| D3.8 | `eury-agent://` deep-link handler that never auto-sends a prompt | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D3.9 | Strict CSP with no remote script, font, or frame origins | [threat model](../03-security/01-threat-model.md) |

## Key decisions and design notes

- Theme and accent switch via CSS variables and data attributes, not React state, so switching costs no re-render.
- The UI never touches the filesystem or network directly; everything is an IPC command ([ADR-0008](../02-architecture/adr/0008-event-protocol-over-tauri-channels.md)).
- Every command lives in one registry with a `when` clause, so disabled states are explainable rather than missing.
- Fonts are bundled; CSP forbids remote origins from day one so no feature can quietly add one.

## Contracts touched

- IPC command envelope and error shape
- Event channel envelope
- Command registry interface
- Design token names

## Dependencies

- Phase 0 (scaffold)
- Phase 1 (surfaces to build)
- Phase 2 (CSP and IPC discipline)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| WebView differences across platforms | Visual and behavioral bugs | Three-OS visual smoke in CI; avoid bleeding-edge CSS; feature-detect |
| IPC contract churn | Rework in later phases | Golden fixtures make changes visible; envelope is versioned |
| Shell performance regressions | Misses the latency budget | Cold-start and IPC benchmarks in CI from this phase forward |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Component behavior, command registry `when` evaluation, keybinding conflicts |
| Contract | IPC and event golden fixtures with TypeScript type parity |
| Integration | Window lifecycle, single instance, deep link, state persistence |
| A11y | `jest-axe` clean; keyboard-only navigation of every surface |
| Visual | Theme × accent × density snapshots on three platforms |

## Metrics and targets

| Metric | Target |
|---|---|
| Cold start to interactive shell | < 400 ms p95 |
| Cold start to fully ready | < 2 s p95 |
| IPC round-trip | < 1 ms p95 |
| Palette first results | < 50 ms |

## Exit criteria

- [ ] Shell launches and navigates on macOS, Windows, and Linux
- [ ] Themes, accents, and density switch without flicker or re-render
- [ ] Command palette works with all prefixes and rebinding
- [ ] IPC and event golden fixtures committed and enforced in CI
- [ ] Window and settings state survive restart
- [ ] CSP verified: no remote script, font, or frame loads
- [ ] Cold-start and IPC benchmarks meet targets

## Deferred from this phase

- SQLite-backed settings (Phase 9)
- Editor and terminal panes (Phases 12–13)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
