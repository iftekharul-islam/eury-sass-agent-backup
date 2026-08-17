# Phase 16 — Memory

Spec-Version: 1.1.0

**Track:** D — Intelligence · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Give the agent durable, inspectable, user-controlled memory: the `EURY.md` hierarchy, a local memory graph, and explicit pinning.

## Why this phase exists here

Repeating project conventions every session is the most common source of user frustration. Memory must be visible and editable, or it becomes an unpredictable hidden prompt.

## In scope

- `EURY.md` hierarchy: global, workspace, subdirectory, plus `.eury.local.md` overrides
- `CLAUDE.md` compatibility import as a one-time, explicit action
- Memory graph with entities, relations, and provenance, stored locally
- Automatic memory extraction proposals that require user confirmation
- Explicit pin and forget actions; memory inspector UI with search
- Recall integrated into context assembly under a token budget
- Memory scoping rules: never leak one workspace's memory into another
- Export and import of memory

## Feature IDs

`F-062`, `F-063`

## Out of scope

- Cloud memory sync (Phase 23)
- Org-shared memory (post-GA)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D16.1 | `EURY.md` hierarchy loader with precedence and size caps | [memory spec](../04-specs/08-memory-spec.md) |
| D16.2 | Memory graph store with provenance per entry | [memory spec](../04-specs/08-memory-spec.md) |
| D16.3 | Extraction proposals shown for confirmation, never auto-written | [memory spec](../04-specs/08-memory-spec.md) |
| D16.4 | Memory inspector: browse, search, edit, pin, forget | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D16.5 | Recall in context assembly with a bounded token budget | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D16.6 | Per-workspace scoping with an explicit global tier | [memory spec](../04-specs/08-memory-spec.md) |
| D16.7 | Memory export/import and the `CLAUDE.md` import path | [memory spec](../04-specs/08-memory-spec.md) |

## Key decisions and design notes

- Memory is never written silently. Every automatic extraction is a proposal the user accepts or rejects.
- `EURY.md` is plain markdown in the repository, so it is reviewable and versioned by git like any other convention document.
- Memory is scoped per workspace by default; cross-workspace leakage is a bug, not a feature.
- Recall latency is measured independently rather than trusted from vendor claims ([benchmarks](../08-quality/03-performance-benchmarks.md)).
- Untrusted repository content cannot write memory — a hostile repo must not be able to plant instructions.

## Contracts touched

- `EURY.md` precedence rules
- Memory entry schema with provenance
- Recall query and result shapes

## Dependencies

- Phase 15 (assembly pipeline)
- Phase 9 (storage)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Memory poisoning from repo content | Persistent misdirection | Untrusted content cannot write memory; proposals require confirmation and show their source |
| Stale memory | Wrong behavior over time | Provenance and timestamps shown; easy forget; conflicting entries surfaced |
| Memory bloat | Token waste | Size caps, relevance-ranked recall, and a visible budget |
| Vendor recall claims unverified | Wrong performance assumptions | Independent benchmark before relying on graph recall in the hot path |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Hierarchy precedence, extraction proposal generation, scoping |
| Integration | Recall affects behavior in a scripted multi-session scenario |
| Security | Hostile repo cannot write memory or influence global tier |
| Performance | Recall latency benchmark against the < 1 ms target |

## Metrics and targets

| Metric | Target |
|---|---|
| Memory recall | < 1 ms p95 |
| Memory token budget per run | < 2000 tokens default |
| Silent memory writes | 0 |

## Exit criteria

- [ ] `EURY.md` hierarchy loads with correct precedence
- [ ] Memory graph recall meets the latency target on our own benchmark
- [ ] No memory is written without user confirmation
- [ ] Memory inspector supports search, edit, pin, and forget
- [ ] Hostile-repo memory poisoning test passes

## Deferred from this phase

- Cloud memory sync (Phase 23)
- Team-shared memory (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
