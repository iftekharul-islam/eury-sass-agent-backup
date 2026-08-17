# Phase 23 — Cloud Sync and Collaboration

Spec-Version: 1.1.0

**Track:** E — Depth · **Estimated size:** 2 weeks · **Milestone:** M3 RC

## Goal

Optional, off-by-default sync of conversations and memory across a user's devices, plus shareable run transcripts. Reach release candidate.

## Why this phase exists here

Users with multiple machines expect continuity, but sync moves content to the server, so it must be opt-in, policy-gated, and clearly explained. It comes last in the depth track for exactly that reason.

## In scope

- `/agent/v1/sync/*` endpoints with cursor-based delta sync
- Conflict resolution: last-write-wins per message, vector clock per conversation, conflicts surfaced
- Selective sync: choose which workspaces and whether memory is included
- Policy gating via `allowCloudSync` and residency routing
- Shareable, redacted run transcripts with an expiring link
- Clear data-handling disclosure in the UI at the moment of opt-in
- Sync status, last-synced time, and a manual sync trigger
- RC hardening pass: bug burn-down across all prior phases

## Feature IDs

`F-006`, `F-070`, `F-071`

## Out of scope

- Real-time multi-user collaboration
- Shared live sessions

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D23.1 | Sync endpoints with cursors and idempotent upserts | [cloud API](../04-specs/06-cloud-api-contract.md) |
| D23.2 | `AgentSyncCursor` per device and resource | [cloud data model](../04-specs/07-cloud-data-model.md) |
| D23.3 | Conflict handling with user-visible resolution | [offline and degraded modes](../02-architecture/06-offline-and-degraded-modes.md) |
| D23.4 | Selective sync settings with an explicit disclosure screen | [privacy](../03-security/07-privacy-and-data-residency.md) |
| D23.5 | Policy-gated sync with residency routing | [workspace policies](../06-enterprise/03-workspace-policies.md) |
| D23.6 | Redacted, expiring shareable transcripts | [privacy](../03-security/07-privacy-and-data-residency.md) |
| D23.7 | RC bug burn-down and regression sweep | [definition of done](../08-quality/05-definition-of-done.md) |

## Key decisions and design notes

- Sync is off by default and never enabled by an update. Turning it on is an explicit, informed action.
- Conversation content syncs into the existing platform tables via the Agent module; no new content tables are introduced.
- Conflicts are surfaced, not silently resolved, when two devices edited the same conversation.
- Shared transcripts are redacted by the same pipeline as audit payloads and expire by default.

## Contracts touched

- Sync request/response and cursor semantics
- Transcript share payload and redaction rules

## Dependencies

- Phase 10 (identity)
- Phase 9 (local store)
- Phase 16 (memory to sync)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Accidental content upload | Privacy incident | Off by default, explicit disclosure, policy override, and a test asserting no network calls when disabled |
| Sync conflicts losing messages | Data loss | Append-only message model with conflict surfacing; property tests over interleaved edits |
| Residency violations | Compliance breach | Residency is org-level and cannot be overridden downstream; routing tested per region |
| Sync amplifying storage cost | Operational cost | Per-plan sync quotas and pruning |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Cursor advancement, conflict detection, redaction |
| Integration | Two-device sync, offline edits on both, reconciliation |
| Property | Interleaved edits never drop a message |
| Security | Sync disabled means zero content leaves the machine |
| Policy | `allowCloudSync = false` disables the feature entirely |

## Metrics and targets

| Metric | Target |
|---|---|
| Sync round trip (100 messages) | < 2 s p95 |
| Content leaving the device with sync off | 0 bytes |
| Message loss in the interleaving property suite | 0 |

## Exit criteria

- [ ] Two devices converge on the same conversation state
- [ ] Sync is provably inert when disabled
- [ ] Conflicts are surfaced with a user-resolvable choice
- [ ] Policy and residency controls enforced
- [ ] RC regression sweep complete
- [ ] M3 RC milestone declared

## Deferred from this phase

- Real-time collaboration (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
