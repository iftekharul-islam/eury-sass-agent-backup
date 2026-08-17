# Phase 07 — Policy and Approval

Spec-Version: 1.1.0

**Track:** B — Core runtime · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Make deny-by-default real: a local policy engine, grant scopes, approval cards, critical-risk dialogs, and the local audit queue.

## Why this phase exists here

Phase 6 gave the agent power. This phase makes that power consented and recorded. It must land immediately after tools, before any wider testing gives users a habit of unrestricted runs.

## In scope

- Policy document schema, defaults, and the four presets
- Merge engine with the never-widen property
- Grant store: once, run, session, always-per-workspace, with normalized-shape matching
- Risk classification rules driving badges and typed confirmation
- Inline approval cards, critical-risk dialog, queueing, batching, timeout auto-deny
- Approvals pane with decision history and grant revocation
- Local audit queue with hash chaining and redaction before write
- Policy-denied UX with structured denial back to the agent

## Feature IDs

`F-008`, `F-025`, `F-030`

## Out of scope

- Cloud policy distribution (Phase 24)
- Cloud audit ingest (Phase 25)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D7.1 | Policy schema, validator, and the Permissive/Standard/Strict/Regulated presets | [workspace policies](../06-enterprise/03-workspace-policies.md) |
| D7.2 | Merge engine with property tests proving it never widens | [policy engine](../03-security/03-permission-and-policy-engine.md) |
| D7.3 | Grant store with scope expiry and shape-based matching | [policy engine](../03-security/03-permission-and-policy-engine.md) |
| D7.4 | Approval cards with risk levels, real buttons, and safe default focus | [approval UX](../05-ui/05-approval-and-trust-ux.md) |
| D7.5 | Approval queue, batching of identical shapes, 10-minute auto-deny | [approval UX](../05-ui/05-approval-and-trust-ux.md) |
| D7.6 | Approvals pane: pending, history, standing grants, revoke | [approval UX](../05-ui/05-approval-and-trust-ux.md) |
| D7.7 | Local audit queue: hash chain, redaction, durable pre-result write | [audit and retention](../06-enterprise/04-audit-and-retention.md) |
| D7.8 | Mode-to-permission profiles enforced for chat/agent/plan/ask/build | [modes](../01-product/03-modes-and-workflows.md) |

## Key decisions and design notes

- The safest button always has default focus, and Allow is enabled only after 400 ms — accidental approval is a real threat.
- `Esc` always denies. There is no auto-approve on a timer.
- Standing grants match a normalized shape, never a raw string, so `npm test` cannot silently authorize a command substitution.
- Audit events are written durably before the tool result returns to the model, so a crash cannot hide an action.
- Policy evaluation happens at tool-call boundaries only; policy never changes mid-tool.

## Contracts touched

- Policy document schema (`schemaVersion: 1`)
- Grant record shape and scope semantics
- Approval request/response IPC commands
- Audit event envelope

## Dependencies

- Phase 5 (guards)
- Phase 6 (tools to gate)
- Phase 2 (model)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Approval fatigue | Users blanket-allow | Shape batching, per-run scopes, and read tools not requiring approval; measured approvals-per-run as a product metric |
| Merge bugs widening permissions | Security hole | Never-widen property test, fuzzed policy pairs, golden merges |
| Audit queue growth | Disk pressure | Size and age caps with fail-closed behavior when upload is required |
| Risk misclassification | Dangerous op looks routine | Corpus of dangerous commands asserted to classify as elevated or critical |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Merge semantics, grant expiry, risk classification |
| Property | Merge never widens; grant matching is shape-exact |
| Integration | Deny-by-default across every tool and mode from a fresh profile |
| UI | Card button order, `Esc` denies, 400 ms enable delay, keyboard-only decisions |
| Security | Fail-closed on corrupt policy; redaction fuzz corpus |

## Metrics and targets

| Metric | Target |
|---|---|
| Policy evaluation per tool call | < 0.5 ms p95 |
| Approval card paint | < 100 ms from request |
| Deny-by-default suite | 100% pass |
| Approvals per typical agent run | < 3 median (product target) |

## Exit criteria

- [ ] Fresh profile denies every write, execute, and network tool without a grant
- [ ] All four grant scopes behave correctly, including expiry and revocation
- [ ] Merge engine passes the never-widen property tests
- [ ] Approval UX is fully keyboard-operable with a safe default
- [ ] Audit events are chained, redacted, and durable
- [ ] Corrupt or missing policy fails closed

## Deferred from this phase

- Cloud policy distribution and signing (Phase 24)
- Cloud audit ingest and export (Phase 25)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
