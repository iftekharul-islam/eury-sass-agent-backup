# Phase 18 — Checkpoints and Rollback

Spec-Version: 1.1.0

**Track:** D — Intelligence · **Estimated size:** 1–2 weeks · **Milestone:** —

## Goal

Make every agent write reversible: automatic checkpoints, per-turn restore, and a clear preview of what a revert will do.

## Why this phase exists here

The single most valuable safety feature after approvals. Users tolerate an agent that makes mistakes if undo is instant and trustworthy.

## In scope

- Automatic checkpoint before the first write of each turn
- Content-addressed snapshot storage with deduplication
- Restore per turn, per run, and per file, with a preview diff before applying
- Handling of renames, deletes, new files, binary files, and unusual encodings
- Interaction with git: checkpoints are independent of, and complementary to, commits
- Retention: age and size caps with visible storage usage and pruning
- Checkpoint audit events for create and restore
- Crash recovery offering resume or revert of an interrupted run

## Feature IDs

`F-065`

## Out of scope

- Cloud-backed checkpoints
- Cross-machine restore

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D18.1 | Checkpoint creation hooked into the write path | [checkpoint spec](../04-specs/12-checkpoint-and-rollback-spec.md) |
| D18.2 | Content-addressed store with dedup and compression | [checkpoint spec](../04-specs/12-checkpoint-and-rollback-spec.md) |
| D18.3 | Restore with a preview diff and explicit file list | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D18.4 | `CheckpointBadge` per turn and the Changes panel revert actions | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D18.5 | Retention policy with storage visibility in Settings | [local data model](../04-specs/05-local-data-model.md) |
| D18.6 | Crash recovery flow offering resume or revert | [runbooks](../07-ops/06-runbooks.md) |
| D18.7 | Audit events for checkpoint create and restore | [audit and retention](../06-enterprise/04-audit-and-retention.md) |

## Key decisions and design notes

- Checkpoints are snapshots of files the agent touched, not whole-repo copies — bounded cost, bounded restore.
- Restore always previews first. A revert that surprises the user is as bad as the original bad edit.
- Checkpoints do not create git commits; users who want commits get them explicitly in Phase 14.
- Checkpoint storage is capped and prunable, and its size is always visible.

## Contracts touched

- Checkpoint record schema
- Restore request/response IPC commands
- Checkpoint audit events

## Dependencies

- Phase 6 (write path)
- Phase 9 (storage)
- Phase 13 (diff preview)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Restore losing user edits made after the agent's write | Data loss | Preview shows conflicts; concurrent user edits block a silent overwrite and require explicit choice |
| Storage growth | Disk pressure | Dedup, compression, age and size caps, visible usage |
| Binary and large files | Slow or huge snapshots | Size thresholds with an explicit skip and a clear warning that those files are not revertible |
| Partial restore leaving an inconsistent tree | Broken build | Restores are atomic per operation with a rollback on failure |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Snapshot creation, dedup, retention pruning |
| Property | Write → checkpoint → restore returns byte-identical content across encodings |
| Integration | Renames, deletes, new files, binaries, and user-edited-after-write cases |
| Fault injection | Kill during restore; tree ends consistent |
| Performance | Checkpoint overhead per write; restore of a 100-file change |

## Metrics and targets

| Metric | Target |
|---|---|
| Checkpoint overhead per write | < 15 ms p95 |
| Restore of a 100-file change | < 2 s p95 |
| Storage per typical run | < 5 MB |
| Restore fidelity in the property suite | 100% |

## Exit criteria

- [ ] Every agent write is covered by a checkpoint
- [ ] Restore works per turn, per run, and per file with a preview
- [ ] Round-trip fidelity holds for binaries, renames, and unusual encodings
- [ ] Retention caps enforced with visible storage usage
- [ ] Crash recovery offers resume or revert

## Deferred from this phase

- Cloud or cross-machine checkpoints (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
