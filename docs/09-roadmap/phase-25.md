# Phase 25 — Policy, Audit, Quotas

Spec-Version: 1.1.0

**Track:** F — Enterprise and GA · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Complete the enterprise control loop: cloud audit ingest with integrity verification, SIEM export, retention, quota and budget enforcement, and policy dry-run.

## Why this phase exists here

Governance is only credible when the evidence trail is verifiable and the spend controls actually stop spend. This phase turns the design from Phases 7 and 24 into an auditable system.

## In scope

- `POST /agent/v1/audit/batch` with signature verification and idempotency
- Hash-chain verification tooling and gap detection alerts
- Retention tiers, nightly purge, legal hold, and archive export
- SIEM delivery: webhook, HEC, and bucket pull, with OCSF mapping
- Audit search, filtering, and queued large exports in the admin console
- `AgentUsageGuard`: seat, concurrency, daily, monthly, and budget enforcement with reservations
- Usage dashboards, budget controls, and cost reconciliation
- Policy dry-run against recent run metadata
- Module isolation check enforced as a release gate

## Feature IDs

`F-082`, `F-083`, `F-084`, `F-085`

## Out of scope

- Billing invoicing changes
- Custom compliance reports (post-GA)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D25.1 | Audit ingest with Ed25519 verification and duplicate handling | [audit and retention](../06-enterprise/04-audit-and-retention.md) |
| D25.2 | Chain verification per device with an evidence-quality report | [audit and retention](../06-enterprise/04-audit-and-retention.md) |
| D25.3 | Retention purge, legal hold, and archive to object storage | [audit and retention](../06-enterprise/04-audit-and-retention.md) |
| D25.4 | SIEM delivery in three modes with OCSF option | [audit and retention](../06-enterprise/04-audit-and-retention.md) |
| D25.5 | Quota and budget guard with token reservations | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |
| D25.6 | Usage dashboards and budget administration | [admin console](../06-enterprise/06-admin-console-spec.md) |
| D25.7 | Monthly cost reconciliation job with a drift alert | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |
| D25.8 | Policy dry-run analysis over recent runs | [admin console](../06-enterprise/06-admin-console-spec.md) |
| D25.9 | `dailyManagedRuns` enforcement implemented in the Agent module, with legacy seed translation at its boundary | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |

## Key decisions and design notes

- Audit is append-only with no update or delete path; purge is the only deletion and is itself logged.
- A detected chain gap is a security event at critical severity, not a warning.
- Quota checks never fail open on a hard limit, even when Redis is down.
- In-flight runs are never killed by a quota boundary; overshoot is bounded by the reservation instead.
- The legacy plan limit that was never enforced is implemented here, in the Agent module, not by patching billing code.

## Contracts touched

- Audit batch request/response and error codes
- SIEM payload formats
- Usage response shape and quota error details

## Dependencies

- Phase 7 (local audit queue)
- Phase 24 (policy distribution)
- Phase 11 (cost accounting)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Audit ingest as a hot path | Latency and cost | Batching, idempotency, async archive, and load testing at 10× projected volume |
| Quota races | Limits exceeded | Atomic Redis operations with a Lua script; concurrency race tests |
| Purge deleting too much | Compliance breach | Dry-run counts before deletion, legal hold checks, and purge summaries |
| Money math errors | Billing disputes | Integer micro-USD, versioned price table, and monthly reconciliation with alerting |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Signature verification, chain verification, retention selection, quota math |
| Contract | Audit and usage endpoints against golden fixtures |
| Concurrency | Parallel requests cannot exceed a daily quota |
| Security | Tampered, replayed, and wrongly signed batches rejected; cross-org access denied |
| Load | 10× projected audit volume; gateway guard chain under load |

## Metrics and targets

| Metric | Target |
|---|---|
| Audit batch (500 events) | < 300 ms p95 |
| Guard chain overhead | < 10 ms p95 |
| Quota overshoot under concurrency | 0 |
| Cost reconciliation drift | < 2% |
| Chain verification on 1M events | < 60 s |

## Exit criteria

- [ ] Audit batches ingest with verified signatures and idempotency
- [ ] Chain verification detects tampering and produces an evidence report
- [ ] Retention, legal hold, and archive work with logged purges
- [ ] SIEM delivery works in all three modes
- [ ] Quotas and budgets enforce without races and without failing open
- [ ] Policy dry-run produces accurate would-have-blocked counts
- [ ] Module isolation check is a release gate

## Deferred from this phase

- Customer-specific compliance reports (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
