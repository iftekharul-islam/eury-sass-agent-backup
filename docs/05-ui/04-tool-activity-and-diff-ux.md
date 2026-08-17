# Tool Activity and Diff UX

Spec-Version: 2.0.0

Preserves the strongest ideas from the deprecated app — a live tool timeline and in-editor write preview — with structured events instead of markdown parsing, and rendered as cards rather than log lines ([visual language](00-visual-language.md)).

## Tool card

```
┌──────────────────────────────────────────────────────────────────┐
│ › ✎ Edit       src/auth/session.ts      +12 −3   1.2s  ✓ Applied │
└──────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────┐
│ › ⌘ Terminal   pnpm test -- session               8.4s  ✗ exit 1 │
└──────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────┐
│ › ⌕ Search     "createSession"          17 hits   42ms  ✓        │
└──────────────────────────────────────────────────────────────────┘
```

| Element | Content |
|---|---|
| Disclosure | Chevron; collapsed by default, expands inside the card |
| Icon | Fixed per tool class so shape is learnable ([design system](01-design-system.md)) |
| Name | Human-readable label in the UI font — `Edit`, `Terminal`, `Search` — not the raw tool id |
| Target | Path, command, or query in monospace, middle-truncated with a full-value tooltip |
| Metric | Diff stat, result count, or exit code |
| Duration | Live-ticking while running, final on completion |
| Status | A pill with an icon and a word: Queued, Needs approval, Running, Applied, Failed, Denied, Cancelled |

The raw tool id is available on hover and in the Runs timeline, because it matters when debugging and never matters when reading.

## Status model

| Status | Color token | Meaning |
|---|---|---|
| `queued` | `--color-fg-subtle` | Accepted, not started |
| `awaiting_approval` | `--color-warning` | Blocked on the user |
| `running` | `--color-info` | Executing, duration ticking |
| `succeeded` | `--color-success` | Completed |
| `failed` | `--color-danger` | Error; expanded shows stderr tail |
| `denied` | `--color-danger` | Policy or user refusal, with reason |
| `cancelled` | `--color-fg-muted` | Aborted run |
| `timed_out` | `--color-danger` | Exceeded tool timeout |

Status derives entirely from `tool_start` / `tool_progress` / `tool_end` events; the UI never infers status from text.

## Expanded content by tool class

| Class | Expanded view |
|---|---|
| Read (`read_file`, `grep`, `glob`, `list_dir`) | Result preview, max 200 lines, with "open in editor" |
| Write (`edit_file`, `write_file`, `apply_patch`) | `DiffBlock` with hunk-level controls |
| Execute (`run_command`) | Command, cwd, exit code, stdout/stderr tail (last 500 lines), "open in terminal" |
| Network (`web_search`, `web_fetch`) | Query, resolved host, result list; untrusted-content marker |
| MCP | Server name, tool name, argument JSON, result JSON (collapsed) |
| Sub-agent | Nested timeline of the child run ([multi-agent](../04-specs/13-multi-agent-spec.md)) |

Large outputs are truncated in the UI with byte counts and a "save full output" action; the full payload stays in SQLite.

## Streaming output

`run_command` streams stdout/stderr into the expanded card at ≤ 10 updates/second, ring-buffered to the last 2 MB in memory. ANSI color is rendered; cursor-control sequences are stripped. A long-running command can be promoted into the Terminal pane without losing history.

## Grouping

Consecutive calls of the same tool collapse into one card with a count (`Read · 5 files · 84ms`) that expands to the individual calls. Groups form only for read-class tools; write and execute calls always get their own card.

## Diff presentation

| Aspect | Rule |
|---|---|
| Computation | In Rust (`similar` crate); UI receives hunks, never computes diffs |
| Default view | Unified; segmented control toggles to split, or `Cmd/Ctrl+Shift+U` |
| Context | 3 lines, expandable per hunk, "expand all" for files < 2000 lines |
| Word-level | Intra-line highlight for single-line edits |
| Syntax | Highlighted per language; falls back to plain on unknown |
| Whitespace | Trailing whitespace and EOL changes marked explicitly |
| Binary | "Binary file changed (N bytes → M bytes)", no content |
| Large file | > 5000 changed lines → hunk-only view, virtualized |
| Rename/move | Single row with old → new path, diff of content if also modified |
| New/deleted | Labeled clearly; new file shows full content collapsed |

Encoding and line endings are preserved and displayed when they differ from the file's original (`CRLF`, `UTF-16`), because silent normalization is a data-loss bug.

## Hunk controls

For a pending write, each hunk offers `Apply`, `Skip`, `Open in editor`. Partial application is allowed: skipped hunks are reported back to the agent as not applied so it can react. Applying nothing is equivalent to denying the tool call.

## Live write preview

When a write tool targets a file that is open in the editor, the editor shows the proposed change as an inline decoration overlay before approval:

- Added lines: `--color-diff-add-bg` with a left accent bar.
- Removed lines: `--color-diff-del-bg`, struck through.
- Gutter: pending marker with hunk navigation (`Alt+↑/↓`).
- Overlay is read-only; editing the file dismisses the preview and invalidates the pending write.

On approval, decorations animate to applied state over 150 ms and the file is written through the sandbox path guard.

## Changes panel

Per run, the Changes pane lists every touched file with `+adds/−dels`, status, and the checkpoint that would revert it. Actions: `Review all`, `Revert file`, `Revert run`.

## Checkpoint affordances

Each assistant turn that wrote files gets a `CheckpointBadge`. Restoring shows the exact file set and a preview diff of the revert before applying, and is itself an audit event ([checkpoints](../04-specs/12-checkpoint-and-rollback-spec.md)).

## Performance targets

| Scenario | Target |
|---|---|
| Card insert during heavy streaming | ≤ 4 ms main-thread work |
| Diff render, 500-line file | ≤ 120 ms to first paint |
| Diff render, 5000-line file | ≤ 400 ms with virtualization |
| Conversation with 500 tool cards | Smooth scroll at 60 fps, virtualized |
| Command output at 10 MB/s | UI stays ≥ 55 fps (drops frames of output, not of UI) |

## Accessibility

Tool cards form a `role="list"`; each card's accessible name is `"<tool> <summary>, <status>, <duration>"`. Diffs expose added and removed state via `aria-label` per line, not color alone. Hunk controls are reachable by keyboard in DOM order.

## Related documents

- [Chat and streaming UX](03-chat-and-streaming-ux.md)
- [Approval and trust UX](05-approval-and-trust-ux.md)
- [Tool catalog](../04-specs/02-tool-catalog-spec.md)
- [Checkpoint and rollback](../04-specs/12-checkpoint-and-rollback-spec.md)
