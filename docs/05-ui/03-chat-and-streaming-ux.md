# Chat and Streaming UX

Spec-Version: 3.2.0

## Turn anatomy

A turn is a header row plus a left-indented body. No bubbles, no double-sided layout: the assistant needs the full content width for diffs and tool cards. Implemented in [`agent/mockups/`](../../mockups/README.md).

```
(M) You                                                    09:41
    Fix the flaky auth session test.
    [📎 src/auth/session.test.ts]

(⬢) Eury   [Sonnet 4] [Agent]                              09:41
    › Thought for 2.1s
    I'll reproduce the failure first, then look at the timer logic.

    ┌──────────────────────────────────────────────────────────┐
    │ ⌄ ⌘ Terminal   pnpm test session        3.1s  ✗ Failed   │
    │   FAIL src/auth/session.test.ts > refreshes before…      │
    └──────────────────────────────────────────────────────────┘
    ┌──────────────────────────────────────────────────────────┐
    │ › ▢ Read   src/auth/session.test.ts    12ms  ✓ 142 lines │
    └──────────────────────────────────────────────────────────┘

    The test advances fake timers before the promise resolves…

    ┌──────────────────────────────────────────────────────────┐
    │ › ✎ Edit   src/auth/session.test.ts  +4 −2  [View diff] ✓ │
    └──────────────────────────────────────────────────────────┘

    ⟳ Verifying the fix…   18s · Esc to interrupt
```

| Element | Rendering |
|---|---|
| User turn | Circular initial avatar, name, timestamp; attachments as file chips |
| Assistant turn | Accent square avatar, name, model badge, mode badge, timestamp |
| Reasoning | Collapsed `› Thought for Ns` disclosure, expandable, off by default |
| Tool card | One card per invocation: chevron, tool icon, tool name, target, duration, status pill |
| Tool detail | Expands inside the card: command output, read ranges, search hits, or a diff |
| Write tool | Shows line counts and a **View diff** action; the diff opens inline or in the Changes pane |
| Streaming | Spinner, a plain-language verb, elapsed time, and the interrupt hint |

Prose, reasoning, and tool cards interleave **in emission order**, always. Nothing is reordered or grouped after the fact, because the order is the explanation of what the agent did. Cards are collapsed by default and expand in place without a scroll jump.

The task checklist and per-run totals live in the active [plan card](05-approval-and-trust-ux.md) and the Runs pane, not in a permanent dashboard rail, so a long run does not push its own summary off screen. Per-turn totals (tokens, cost, duration) appear on the turn's hover action strip and in the Runs pane.

## Composer

| Element | Behavior |
|---|---|
| Input | Auto-growing textarea, 1–12 rows, then internal scroll |
| Send | `Enter`; newline `Shift+Enter`; queue-while-running `Cmd/Ctrl+Enter` |
| Mode | `ModeSelect` in the composer toolbar, mirrored in the title bar — chat, ask, plan, agent, build ([modes](../01-product/03-modes-and-workflows.md)) |
| Model | `ModelSelect` in the composer toolbar, with per-model context window and cost hint |
| Attach | Files (`@`), images (paste/drop), terminal selection, current diff |
| `@` mention | Fuzzy file/symbol picker; inserts a path reference, not file contents |
| `/` command | Slash commands (`/plan`, `/test`, `/review`, `/clear`, `/compact`, `/checkpoint`) |
| Draft | Persisted per conversation; survives restart |
| Token meter | Live estimate of prompt tokens vs. window; warns at 80% |

Attachments never inline whole files into the prompt; the runtime resolves references through retrieval so the model sees only relevant ranges ([indexing and retrieval](../04-specs/09-indexing-and-retrieval-spec.md)).

## Web and visual media

Web research and visual work must be visible in the transcript, with source and privacy information available before a user relies on the result.

| Surface | Behavior |
|---|---|
| Web result | Compact activity line with search/fetch status; results cite host + title as numbered chips. Opening a source requires the existing external-host confirmation. |
| Image attachment | Paste, drop, or use the attachment menu. The composer shows a thumbnail, filename or `Pasted image`, dimensions, byte size, and remove button before sending. |
| Image inspection | `read_image` appears as a compact activity line. Its expanded detail offers the image preview, local-downscale notice when applicable, and an OCR/observation summary. |
| Generated images | A successful generation renders an accessible gallery in the assistant turn: thumbnail, alt text, dimensions, provider, and generated-at time. |
| Save image | The gallery’s **Save to project** action opens the ordinary file-write approval path; generation never deposits files in the workspace by itself. |
| Unsupported model | The attachment remains available, but the composer labels image understanding as unavailable and offers a vision-capable model picker. |

Image requests preserve aspect ratio in the transcript and never autoplay animated media. A full-size preview opens in a modal with Copy, Download, and Save to project actions. Provider-bound image previews remain in the encrypted attachment store and are removed with the conversation according to the retention policy.

## Streaming pipeline

```
Rust core ──channel──> useRunStream() ──rAF batch──> React state ──> DOM
```

| Rule | Value |
|---|---|
| Transport | Tauri channel, one run per channel ([event protocol](../04-specs/03-event-protocol-spec.md)) |
| Coalescing | Deltas buffered and flushed once per animation frame (~16 ms) |
| Max flush size | 8 KB per frame; excess carries to the next frame |
| Rendering | Markdown re-parsed only on block boundaries; open block renders as plain text until closed |
| Virtualization | Messages virtualized above 200 turns; only the streaming turn is un-virtualized |
| Time to first token | ≤ 800 ms p95 for BYOK, ≤ 1.2 s managed ([latency budget](../02-architecture/05-latency-budget.md)) |
| Frame budget | ≥ 55 fps while streaming and while a terminal is attached |

Backpressure: if the UI cannot keep up, the hook drops intermediate `reasoning` frames first, then coalesces `delta` frames more aggressively. It never drops `tool_*`, `approval_*`, `checkpoint_*`, `usage`, `done`, or `error` events.

## Autoscroll

Sticky-bottom while the user is within 64 px of the bottom. Any upward scroll or wheel gesture detaches and shows a "Jump to latest" pill with a new-content count. `End` or clicking the pill re-attaches. Autoscroll never fights the user, and never re-attaches on its own.

## Reasoning display

Collapsed `▸ Thinking (Ns)` summary by default; expandable. Reasoning text is stored in SQLite but excluded from context reconstruction on later turns unless the provider requires it. Setting `ui.showReasoning` can pin it open.

## Interruption and steering

| Action | Key | Effect |
|---|---|---|
| Abort run | `Esc` (twice) or Stop button | Cancels model stream and in-flight tools; partial output kept and labeled `aborted` |
| Steer mid-run | Type + `Cmd/Ctrl+Enter` | Message queues and is injected at the next turn boundary |
| Queue follow-up | Type + `Enter` while running | Sent automatically when the run completes |

Abort must reflect in the UI within 250 ms even if a tool takes longer to unwind; the row shows `cancelling…` then `cancelled`.

## Turn actions

Turns have no visible action bar. Hovering a turn reveals a right-aligned strip, and `⌘↑` / `⌘↓` selects a turn for keyboard use. Both routes offer the same actions:

| Action | Applies to | Notes |
|---|---|---|
| Copy | any | Markdown source, not rendered HTML |
| Retry | assistant | Re-runs from that turn; prior turn's tools are not replayed |
| Retry with different model | assistant | Opens picker, preserves context |
| Edit and resend | user | Forks the conversation; original branch stays reachable |
| Delete | any | Deletes this turn and all later turns after confirm |
| Restore checkpoint | assistant with writes | Reverts files to before that turn ([checkpoints](../04-specs/12-checkpoint-and-rollback-spec.md)) |
| Share as issue | assistant | Copies redacted transcript to clipboard |

## Context management

Context usage shows as `used / window`. Approaching the limit, the UI offers, in order: drop reasoning, summarize older turns (`/compact`), start a fresh conversation carrying a summary. Compaction is explicit and shows exactly what was summarized — no silent truncation.

## Markdown rendering

| Concern | Rule |
|---|---|
| Sanitization | Allowlist renderer; no raw HTML, no inline styles, no `javascript:` URLs |
| Links | External links open in the OS browser after a confirm dialog showing the host |
| Code fences | `CodeBlock` with copy, "open file", and "insert at cursor" |
| Diff fences | Rendered as `DiffBlock` when the fence language is `diff` or a patch header is detected |
| Math | KaTeX, bundled |
| Mermaid | Rendered in a sandboxed offscreen renderer, image output only |
| Images | Only `file://` inside the workspace, `data:` from tools, or `https:` after confirm |
| Untrusted content | Tool output and web content render inside a marked region and can never produce active content ([prompt injection defense](../03-security/05-prompt-injection-defense.md)) |

## Citations

Web and file citations render as numbered chips. Clicking a file citation opens the file at the cited range; clicking a web citation shows host + title and requires confirm to open.

## Anti-patterns (from the deprecated app)

| Do not | Instead |
|---|---|
| Parse tool calls out of markdown fences | Use structured tool-call events |
| Regex-delete model text before display | Render everything; hide via explicit UI affordances |
| Animate every token | Frame-batched appends |
| Re-parse the whole markdown tree per delta | Parse only the open block |
| Block the UI thread on large diffs | Compute diffs in Rust, stream hunks |

## Accessibility

The message list is an `aria-log` with `aria-relevant="additions text"`. Streaming updates announce at most once per second via a debounced live region. Full requirements: [accessibility and i18n](07-accessibility-and-i18n.md).

## Related documents

- [Tool activity and diff UX](04-tool-activity-and-diff-ux.md)
- [Multimodal and attachments](../04-specs/17-multimodal-and-attachment-spec.md)
- [Event protocol](../04-specs/03-event-protocol-spec.md)
- [Modes and workflows](../01-product/03-modes-and-workflows.md)
