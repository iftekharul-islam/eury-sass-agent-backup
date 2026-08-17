# Phase 12 — Terminal

Spec-Version: 1.1.0

**Track:** C — Product surfaces · **Estimated size:** 1–2 weeks · **Milestone:** —

## Goal

A real PTY-backed terminal pane with high-throughput rendering, plus promotion of long-running tool commands into that pane.

## Why this phase exists here

Developers need to see and drive the shell themselves, and agent-run commands need somewhere to live when they outgrow a collapsed tool card.

## In scope

- PTY management in Rust with `portable-pty`; ConPTY on Windows
- xterm.js integration with a ring buffer and frame-coalesced writes
- Up to four sessions per workspace as tabs, with resize and signal forwarding
- Sanitized environment; no secrets injected
- Promotion of a `run_command` tool execution into a terminal view
- Explicit `share terminal output` action to expose text to the agent
- Selection to composer; clear, kill, and scrollback controls

## Feature IDs

`F-044`, `F-053`

## Out of scope

- Agent typing into a user terminal (explicitly never)
- Remote or SSH terminals

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D12.1 | PTY lifecycle with resize, signals, and reliable kill | [editor/terminal/preview](../05-ui/06-editor-terminal-preview.md) |
| D12.2 | xterm.js renderer sustaining 10 MB/s without UI stall | [editor/terminal/preview](../05-ui/06-editor-terminal-preview.md) |
| D12.3 | Terminal pane with per-session tabs and scrollback | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D12.4 | Tool-to-terminal promotion preserving history | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D12.5 | Explicit output-sharing action with an untrusted-content marker | [injection defense](../03-security/05-prompt-injection-defense.md) |
| D12.6 | Windows ConPTY support with a degradation warning when unavailable | [desktop runtime](../02-architecture/02-desktop-runtime.md) |

## Key decisions and design notes

- The agent never types into a user terminal; agent commands always go through `run_command` with policy and sandbox enforcement.
- Terminal output only becomes model context when the user explicitly shares it, and then it is marked untrusted.
- Promotion is a view change, not a permission change.
- Environment is sanitized: keychain secrets are never injected into a shell the model can influence.

## Contracts touched

- Terminal IPC commands (create, write, resize, kill)
- Terminal output event framing

## Dependencies

- Phase 5 (process supervision)
- Phase 6 (`run_command`)
- Phase 3 (pane shell)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Windows PTY differences | Broken terminal on Windows | ConPTY-first implementation, platform test matrix, honest degradation |
| Output flooding the UI | Freeze | Ring buffer, frame coalescing, and dropping output frames rather than UI frames |
| Terminal used as an injection channel | Agent misdirection | Sharing is explicit and marked untrusted |
| Orphaned shells | Resource leak | Process-group kill on tab close and on app quit; leak test |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | PTY lifecycle, resize, signal handling |
| Integration | Interactive programs (`vim`, `top`), ANSI rendering, large output |
| Performance | 10 MB/s throughput with frame-rate assertion |
| Platform | All three OSes including ConPTY |
| Leak | No orphaned processes after close or quit |

## Metrics and targets

| Metric | Target |
|---|---|
| First prompt after tab open | < 300 ms |
| Throughput | ≥ 10 MB/s with UI ≥ 55 fps |
| Keystroke echo latency | < 16 ms p95 |
| Orphaned shells | 0 |

## Exit criteria

- [ ] Terminal works on all three platforms with interactive programs
- [ ] Throughput and latency targets met
- [ ] Tool commands can be promoted to a terminal without losing history
- [ ] Output sharing is explicit and marked untrusted
- [ ] No orphaned processes in the leak suite

## Deferred from this phase

- Remote and container shells (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
