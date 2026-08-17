# Phase 06 — Tool Layer v1

Spec-Version: 1.1.0

**Track:** B — Core runtime · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Ship the core tool catalog on top of the sandbox: read, search, write, patch, and execute, with structured results and streaming progress.

## Why this phase exists here

Tools are what make the agent useful and what make it dangerous. They land after the sandbox and before the approval UI so that every tool is deniable from the moment it exists.

## In scope

- Tool registry with JSON-schema declarations and strict argument validation
- Read tools: `read_file`, `list_dir`, `glob`, `grep`
- Write tools: `write_file`, `edit_file`, `apply_patch` with atomic writes
- Execute tool: `run_command` with streamed stdout/stderr and exit codes
- Diff computation in Rust with hunk-level output
- Encoding and line-ending detection and preservation
- Tool result envelope: truncation with byte counts, never silent loss
- Tool activity events: start, progress, end, with durations
- Cost/limit guard hook points for Phase 11

## Feature IDs

`F-008`, `F-025`, `F-029`, `F-040`, `F-041`, `F-042`, `F-043`

## Out of scope

- Approval prompts (Phase 7)
- Network tools (Phase 11)
- MCP tools (Phase 19)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D6.1 | Tool registry with schema validation and versioned tool ids | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D6.2 | Read/search tools with deterministic, bounded output | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D6.3 | Write tools with atomic replace and preserved encoding/EOL | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D6.4 | `run_command` with streaming, timeout, and output caps | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D6.5 | Rust-side diff engine emitting hunks | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D6.6 | Tool activity timeline in the UI with per-class expanded views | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D6.7 | Tool result truncation policy with full payload retained locally | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D6.8 | Structured tool errors mapped to the taxonomy | [error taxonomy](../04-specs/15-error-taxonomy.md) |

## Key decisions and design notes

- Tool calls are structured events from the engine, never parsed out of markdown — the single biggest correctness fix over the deprecated app.
- Diffs are computed in Rust and streamed as hunks; the webview never diffs large files.
- Writes are atomic (temp file plus rename) so an interrupted write cannot corrupt a source file.
- Encoding and line endings are preserved; silent normalization is treated as data loss.
- Every tool declares its class, which is what policy and approvals key on.

## Contracts touched

- Tool schemas and ids
- Tool result envelope
- `tool_start`/`tool_progress`/`tool_end` events
- Diff hunk format

## Dependencies

- Phase 4 (engine and tool-call events)
- Phase 5 (sandbox)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Tool output floods context | Cost and latency blowups | Hard output caps, structured truncation, retrieval-based reading in Phase 15 |
| Patch application fragility | Failed or wrong edits | Fuzzed patch corpus; exact-match then fuzzy fallback with an explicit failure rather than a guess |
| Encoding regressions | Corrupted files | Round-trip tests across UTF-8/16, BOM, CRLF, and mixed-EOL fixtures |
| Long-running commands | Hung runs | Timeouts, progress events, promotion to a terminal tab in Phase 12 |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Each tool's argument validation and result shape |
| Integration | Real filesystem and real processes through the sandbox |
| Property | Patch apply/revert round trips |
| Security | Every tool re-run through the traversal corpus |
| Performance | `read_file`, `grep`, and diff benchmarks against targets |

## Metrics and targets

| Metric | Target |
|---|---|
| `read_file` 10 KB | < 10 ms p95 |
| `write_file` 10 KB | < 25 ms p95 |
| `grep` across 10k files | < 300 ms p95 |
| Diff 500-line file | < 20 ms p95 |
| Tool argument validation rejection rate on the fuzz corpus | 100% |

## Exit criteria

- [ ] All v1 tools work through the sandbox with no direct filesystem access
- [ ] Tool activity timeline shows structured status and durations
- [ ] Diffs render with hunk-level detail from Rust-computed patches
- [ ] Encoding and EOL round trips are lossless
- [ ] Traversal corpus passes for every tool
- [ ] Tool performance targets met

## Deferred from this phase

- Approval gating (Phase 7)
- Live write preview in the editor (Phase 13)
- Checkpoints (Phase 18)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
