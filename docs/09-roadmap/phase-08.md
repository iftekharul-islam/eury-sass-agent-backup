# Phase 08 — Chat Experience

Spec-Version: 1.1.0

**Track:** B — Core runtime · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Turn the streaming pipeline into the product surface: composer, turn rendering, markdown safety, interruption, steering, and context management.

## Why this phase exists here

Chat is where users spend their time and where the deprecated app's worst behaviors lived (fence parsing, regex output surgery, per-token animation). Doing it properly requires tools and approvals to already exist so their surfaces render natively.

## In scope

- Composer: auto-grow, attachments, `@` mentions, `/` commands, persisted drafts
- Turn rendering with interleaved text, reasoning, and tool cards in emission order
- Sanitized markdown: allowlist renderer, code and diff fences, math, sandboxed mermaid
- Autoscroll with sticky-bottom and non-fighting detach behavior
- Abort, steering mid-run, and queued follow-ups
- Turn actions: copy, retry, retry-with-model, edit-and-resend, delete
- Context meter with explicit compaction (`/compact`), never silent truncation
- Citations for files and web sources
- Virtualized message list for long conversations

## Feature IDs

`F-008`, `F-020`, `F-028`, `F-034`, `F-035`

## Out of scope

- Persistence of history (Phase 9)
- Retrieval-driven context (Phase 15)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D8.1 | Composer with attachments, mentions, slash commands, drafts | [chat UX](../05-ui/03-chat-and-streaming-ux.md) |
| D8.2 | Turn renderer with frame-batched streaming and open-block parsing | [chat UX](../05-ui/03-chat-and-streaming-ux.md) |
| D8.3 | Sanitized markdown pipeline with confirm-on-external-link | [injection defense](../03-security/05-prompt-injection-defense.md) |
| D8.4 | Autoscroll and jump-to-latest behavior | [chat UX](../05-ui/03-chat-and-streaming-ux.md) |
| D8.5 | Abort, steering, and queued follow-ups | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D8.6 | Turn actions including retry and conversation forking | [chat UX](../05-ui/03-chat-and-streaming-ux.md) |
| D8.7 | Context meter and explicit compaction with a visible summary | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D8.8 | Message list virtualization above 200 turns | [chat UX](../05-ui/03-chat-and-streaming-ux.md) |

## Key decisions and design notes

- Never delete or rewrite model output for display; hide with explicit affordances instead.
- Only the open markdown block re-parses per delta; closed blocks are memoized.
- No per-token animation — a blinking cursor conveys liveness at a fraction of the cost.
- Compaction is user-visible and reversible in the sense that the original turns remain stored.
- Untrusted content (tool output, web results) renders in a marked region and can never produce active content.

## Contracts touched

- Conversation and message in-memory shapes (persisted in Phase 9)
- Slash-command registry entries
- Attachment reference format

## Dependencies

- Phase 4 (streaming)
- Phase 6 (tool cards)
- Phase 7 (approval surfaces)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Markdown XSS | Compromise of the webview | Allowlist renderer, no raw HTML, CSP, and an injection corpus in CI |
| Streaming jank on long conversations | Poor perceived quality | Virtualization plus frame batching, measured in the streaming benchmark |
| Context overflow surprises | Failed runs, wasted cost | Live meter, warnings at 80%, explicit compaction path |
| Steering semantics confusing | Users think a message was ignored | Queued messages render as pending with a clear label |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Renderer sanitization, autoscroll state machine, composer behaviors |
| Integration | Long streaming session with tool cards and approvals interleaved |
| Security | Injection corpus rendered without active content or link auto-navigation |
| Performance | 500-turn conversation scroll and 200 events/second streaming |
| A11y | Live-region announcement throttling; keyboard-only turn actions |

## Metrics and targets

| Metric | Target |
|---|---|
| Frame rate while streaming | ≥ 55 fps |
| Main-thread work per delta batch | < 4 ms |
| 500-turn conversation scroll | 60 fps |
| Time to first rendered token after `meta` | < 50 ms |

## Exit criteria

- [ ] Full conversation flow works with tools and approvals interleaved
- [ ] Markdown sanitization passes the injection corpus
- [ ] Abort, steering, and queued follow-ups behave as specified
- [ ] Context meter and compaction work with no silent truncation
- [ ] Performance and accessibility targets met

## Deferred from this phase

- History across restarts (Phase 9)
- Retrieval-based context assembly (Phase 15)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
