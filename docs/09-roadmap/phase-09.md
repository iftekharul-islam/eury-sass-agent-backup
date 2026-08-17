# Phase 09 — Local Persistence

Spec-Version: 1.1.0

**Track:** B — Core runtime · **Estimated size:** 2 weeks · **Milestone:** M1 Alpha

## Goal

Persist everything locally in an encrypted SQLite database with migrations, and reach the internal alpha: chat, tools, approvals, and history that survive restart.

## Why this phase exists here

Persistence is what turns a demo into a tool. It is also the phase that replaces the deprecated app's whole-file JSON store, which was slow, corruptible, and unqueryable ([ADR-0004](../02-architecture/adr/0004-sqlite-local-store-and-encryption.md)).

## In scope

- SQLite schema: workspaces, conversations, messages, runs, tool_calls, grants, settings, audit_queue
- Migration framework with forward-only, transactional migrations and pre-migration snapshots
- Encryption at rest with the key held in the OS keychain
- WAL mode, multi-window concurrency, and integrity checks on open
- Corruption recovery: WAL recovery, quarantine, fresh start with import offer
- Data export and import (conversations, runs, plans, settings) as JSON
- Retention and pruning for large tool payloads
- Deterministic eval harness skeleton running against the stub provider
- Alpha packaging for internal dogfood (unsigned)

## Feature IDs

`F-008`, `F-068`, `F-069`

## Out of scope

- Cloud sync (Phase 23)
- Index storage (Phase 15)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D9.1 | Full local schema with indexes on every query path | [local data model](../04-specs/05-local-data-model.md) |
| D9.2 | Migration framework with snapshot-before-migrate | [local data model](../04-specs/05-local-data-model.md) |
| D9.3 | At-rest encryption keyed from the OS keychain | [secrets](../03-security/04-secrets-and-key-management.md) |
| D9.4 | Multi-window safe access with WAL and busy-timeout handling | [desktop runtime](../02-architecture/02-desktop-runtime.md) |
| D9.5 | Corruption detection, quarantine, and recovery flow | [runbooks](../07-ops/06-runbooks.md) |
| D9.6 | Export/import round trip covering all local entities | [backup and DR](../07-ops/07-backup-and-dr.md) |
| D9.7 | Settings migrated from file-backed storage to SQLite | [local data model](../04-specs/05-local-data-model.md) |
| D9.8 | Eval harness skeleton with deterministic tasks in CI | [eval harness](../08-quality/02-agent-eval-harness.md) |
| D9.9 | Internal alpha build and dogfood feedback loop | [release management](../07-ops/08-release-management.md) |

## Key decisions and design notes

- SQLite with WAL, not JSON files: queryable, incremental, and crash-resistant.
- Encryption key lives in the keychain; the database file alone is useless if copied.
- Migrations are forward-only within a major version, and every one takes a snapshot first so a bad release can be reverted.
- The agent is not a backup system: user code safety comes from git, and this store holds conversation state.
- The eval skeleton lands here because the stub provider plus persistence is the minimum needed to score behavior.

## Contracts touched

- Local schema and migration version table
- Export bundle format
- Settings key namespace (`ui.*`, `agent.*`, `policy.*`)

## Dependencies

- Phase 8 (data worth persisting)
- Phase 7 (grants and audit queue)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Migration bugs losing data | Unacceptable | Snapshots, transactional migrations, round-trip tests, and a forced-restore test in CI |
| Encryption breaking keychain portability | Users locked out | Documented recovery path; export before migration; clear error rather than silent reset |
| Multi-window write contention | Lock errors | WAL, short transactions, busy timeout, and a concurrency stress test |
| Large payload growth | Disk bloat | Payload caps with pruning policy and a visible storage breakdown in Settings |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Each migration up-path; schema constraints |
| Integration | Restart persistence, multi-window concurrency, WAL recovery |
| Property | Export → wipe → import equality |
| Fault injection | Kill mid-write and mid-migration; database remains usable |
| Performance | 500-message conversation load benchmark |

## Metrics and targets

| Metric | Target |
|---|---|
| Conversation load (500 messages) | < 50 ms p95 |
| Message insert during streaming | < 2 ms p95 |
| Migration time on a 1 GB database | < 10 s |
| Data loss in fault-injection suite | 0 |

## Exit criteria

- [ ] All state survives restart, including drafts, pane history, terminal tabs, grants, and audit queue
- [ ] Migrations are transactional with a pre-migration snapshot
- [ ] Encryption verified: no plaintext content in the database file
- [ ] Corruption recovery works without losing unaffected data
- [ ] Export/import round trip is lossless
- [ ] Alpha build in internal dogfood with feedback captured
- [ ] M1 Alpha milestone declared

## Deferred from this phase

- Cloud sync (Phase 23)
- Full eval suite (Phase 28)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
