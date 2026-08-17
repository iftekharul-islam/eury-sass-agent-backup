# Phase 20 — Multi-Agent Orchestration

Spec-Version: 1.1.0

**Track:** D — Intelligence · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Use sub-agents for genuinely parallel or specialized work — planner, implementer, tester, reviewer — with strict budgets, isolation, and a single accountable parent run.

## Why this phase exists here

Some tasks are naturally parallel (test three hypotheses, review a large change). Sub-agents make those faster, but they multiply cost and risk, so they land last in the intelligence track with everything else already governed ([ADR-0010](../02-architecture/adr/0010-multi-agent-orchestration-model.md)).

## In scope

- Sub-agent spawning through Cersei with role-specific prompts and tool subsets
- Parent/child run hierarchy with nested timelines in the UI
- Per-sub-agent budgets: turns, tokens, cost, wall clock, and tool classes
- Result aggregation and conflict handling when children touch the same files
- Write serialization: only one child may hold a write lease on a path
- Cancellation propagation from parent to all children
- Role presets: planner, implementer, tester, reviewer
- Feature flag and kill switch for the whole subsystem

## Feature IDs

`F-027`

## Out of scope

- Distributed or remote sub-agents
- User-authored custom roles (post-GA)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D20.1 | Sub-agent lifecycle with inherited-but-narrowable permissions | [multi-agent](../04-specs/13-multi-agent-spec.md) |
| D20.2 | Nested run timeline UI | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D20.3 | Budget enforcement per child and in aggregate | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |
| D20.4 | Write lease manager preventing concurrent writes to one path | [multi-agent](../04-specs/13-multi-agent-spec.md) |
| D20.5 | Result aggregation with explicit conflict reporting | [multi-agent](../04-specs/13-multi-agent-spec.md) |
| D20.6 | Role presets with distinct tool subsets | [modes](../01-product/03-modes-and-workflows.md) |
| D20.7 | `allowSubAgents` policy flag and server-side kill switch | [workspace policies](../06-enterprise/03-workspace-policies.md) |

## Key decisions and design notes

- A child agent can never hold more permission than its parent, and may hold less.
- Approvals always surface to the human on the parent run; a child cannot approve its own request.
- Writes are serialized by path lease, because concurrent edits to one file is a correctness disaster, not a performance win.
- Sub-agents are off by default at GA and enabled per organization, since cost amplification is the main support risk.

## Contracts touched

- Sub-agent spawn request and budget shape
- Nested run event envelope
- Aggregated result format

## Dependencies

- Phase 17 (plans to parallelize)
- Phase 11 (cost accounting)
- Phase 7 (approvals)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Cost explosion | Surprise bills | Hard aggregate budgets, default off, visible live cost, and abort on cap |
| Conflicting writes | Corrupted work | Path leases plus checkpoints per child; conflicts reported rather than merged |
| Debuggability | Users cannot follow what happened | Nested timelines with per-child attribution and full transcripts |
| Approval flooding | Fatigue | Approvals batched at the parent with the requesting child named |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Budget accounting, lease acquisition, permission inheritance |
| Integration | Parallel children with overlapping file targets; cancellation propagation |
| Eval | Multi-agent task category compared against single-agent baseline for quality and cost |
| Security | A child cannot exceed parent permissions or self-approve |

## Metrics and targets

| Metric | Target |
|---|---|
| Speedup on parallelizable eval tasks | ≥ 1.5× vs. single agent |
| Cost overhead | < 2× single-agent for the same task |
| Write conflicts reaching disk | 0 |
| Orphaned children after parent abort | 0 |

## Exit criteria

- [ ] Sub-agents run with enforced budgets and narrowed permissions
- [ ] Write leases prevent concurrent writes to the same path
- [ ] Cancellation propagates to every child with no orphans
- [ ] Nested timeline makes each child's work attributable
- [ ] Multi-agent eval category shows measurable benefit without unsafe results

## Deferred from this phase

- Custom user-defined roles (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
