# Phase 21 — Background and Scheduled Work

Spec-Version: 1.1.0

**Track:** E — Depth · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Let runs continue outside the foreground: queued runs, background execution on a separate worktree, notifications, and scheduled tasks.

## Why this phase exists here

Long tasks should not hold the UI hostage, and recurring maintenance work (dependency bumps, test triage) is a natural agent job. Requires git worktrees and checkpoints to be safe.

## In scope

- Run queue with concurrency limits and priority
- Background runs on a dedicated git worktree so the user's working tree stays stable
- Progress in the status bar and the Runs surface; OS notifications on completion
- Scheduled runs (cron-like) with policy gating and a per-schedule budget
- Result review flow: diff the worktree result before merging into the main tree
- Resource governance: pause on battery, throttle under load, respect Do Not Disturb
- Approval handling for background runs: queue and notify, never auto-approve

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- Cloud-executed background runs
- Multi-machine scheduling

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D21.1 | Run queue with limits, priority, and fair scheduling | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D21.2 | Worktree-isolated background execution | [feature catalog](../01-product/02-feature-catalog.md) |
| D21.3 | Runs surface with live progress and history | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D21.4 | Scheduler with policy gating and per-schedule budgets | [workspace policies](../06-enterprise/03-workspace-policies.md) |
| D21.5 | Background result review and merge-into-main-tree flow | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D21.6 | OS notifications that never include file contents | [approval UX](../05-ui/05-approval-and-trust-ux.md) |
| D21.7 | Resource governance: battery, load, and DND awareness | [latency budget](../02-architecture/05-latency-budget.md) |

## Key decisions and design notes

- Background runs never write to the user's working tree; they use a worktree and produce a reviewable result.
- Scheduled runs never auto-approve anything. A schedule that would need approval pauses and notifies.
- Notifications carry the tool name and run title only — never payloads.
- Background work yields to foreground work for CPU and model concurrency.

## Contracts touched

- Run queue and schedule record schemas
- Background run result envelope

## Dependencies

- Phase 14 (git worktrees)
- Phase 18 (checkpoints)
- Phase 17 (plans as schedulable units)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Unattended runs consuming budget | Cost surprise | Per-schedule budgets, global caps, and visible spend attribution per schedule |
| Worktree merge conflicts | User frustration | Explicit review-and-merge step with conflict presentation; never auto-merge |
| Background runs blocking on approval forever | Wasted work | Approval timeout with a resumable pause and clear notification |
| Resource contention | Sluggish machine | Throttling, battery awareness, and a hard concurrency cap |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Queue ordering, schedule parsing, budget accounting |
| Integration | Background run in a worktree, review, merge; conflicting-change case |
| Security | Scheduled run cannot auto-approve; policy gating enforced |
| Endurance | 24-hour scheduler soak with no leaks |

## Metrics and targets

| Metric | Target |
|---|---|
| Foreground latency impact while a background run is active | < 10% |
| Scheduled run reliability over 7 days | > 99% |
| Auto-approvals in background runs | 0 |

## Exit criteria

- [ ] Runs can be queued and executed in the background on a worktree
- [ ] Background results are reviewed before touching the working tree
- [ ] Scheduled runs respect policy and budgets and never auto-approve
- [ ] Resource governance keeps the foreground responsive

## Deferred from this phase

- Cloud-executed runs (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
