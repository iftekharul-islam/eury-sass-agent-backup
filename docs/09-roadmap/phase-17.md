# Phase 17 — Modes and Plan Execution

Spec-Version: 1.1.0

**Track:** D — Intelligence · **Estimated size:** 2 weeks · **Milestone:** M2 Beta

## Goal

Ship the five modes as real permission profiles and plan mode as a durable, reviewable, step-by-step execution model. Reach beta.

## Why this phase exists here

Plan mode is the answer to 'the agent did too much at once'. It converts a risky long run into a reviewable sequence, and it needs tools, policy, memory, and the editor already in place.

## In scope

- Mode profiles enforcing distinct tool sets and approval requirements
- Plan format as markdown with structured frontmatter, stored in `<workspace>/.eury/plans/`
- Plan generation, editing, and re-planning after a failed step
- Step-by-step execution with per-step approval, skip, and retry
- Plan progress in the context panel with per-step tool attribution
- `requirePlanBeforeWrite` policy support for plan-gated organizations
- Plan resume across restarts
- Beta packaging and the opt-in beta channel

## Feature IDs

`F-023`, `F-024`, `F-026`, `F-064`

## Out of scope

- Multi-agent execution of plan steps (Phase 20)
- Scheduled plan runs (Phase 21)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D17.1 | Mode profiles with per-mode default tools and approval rules | [modes](../01-product/03-modes-and-workflows.md) |
| D17.2 | Plan file format with a validating parser and writer | [plan format](../04-specs/11-plan-format-spec.md) |
| D17.3 | Plan generation and human editing before execution | [plan format](../04-specs/11-plan-format-spec.md) |
| D17.4 | Step executor with approval, skip, retry, and re-plan | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D17.5 | Plan progress UI with step-to-tool attribution | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D17.6 | Plan-gated writes enforced from policy | [workspace policies](../06-enterprise/03-workspace-policies.md) |
| D17.7 | Plan resume after restart or abort | [checkpoint spec](../04-specs/12-checkpoint-and-rollback-spec.md) |
| D17.8 | Beta build on the beta channel with dogfood telemetry | [release management](../07-ops/08-release-management.md) |

## Key decisions and design notes

- Plans are markdown files in the repository: diffable, reviewable, and shareable without our app.
- Modes are permission profiles first and prompt framing second, which is what makes `ask` mode genuinely safe.
- A failed step stops execution and offers re-planning rather than improvising past the failure.
- Plan state is persisted after every step so a crash resumes rather than restarts.

## Contracts touched

- Plan file schema and frontmatter
- Plan step execution events
- Mode profile definitions

## Dependencies

- Phase 7 (approvals)
- Phase 13 (review surfaces)
- Phase 16 (memory in planning)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Plans too coarse or too granular | Poor usability | Eval tasks scoring plan quality; step-count guidance in the prompt; user editing always available |
| Plan drift from reality | Steps no longer make sense | Re-plan on failure; plan references files by path with content hashes where relevant |
| Mode confusion | Unexpected permissions | Mode badge always visible; approval cards state the mode; mode-permission matrix tested |
| Plan files cluttering repos | User annoyance | Single directory, gitignore guidance, easy cleanup |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Plan parse/serialize round trip; mode profile resolution |
| Integration | Generate, edit, execute, fail a step, re-plan, resume after restart |
| Eval | Plan-mode task category with validity and execution assertions |
| Security | Each mode's tool set enforced; `ask` mode cannot write |

## Metrics and targets

| Metric | Target |
|---|---|
| Plan generation latency | < 15 s p95 (model-bound) |
| Plan parse | < 5 ms |
| Plan-mode eval pass rate | ≥ 85% |
| Plan resume success after forced kill | 100% |

## Exit criteria

- [ ] All five modes enforce distinct, tested permission profiles
- [ ] Plans generate, validate, edit, execute, and resume correctly
- [ ] Failed steps offer re-planning instead of silent continuation
- [ ] Plan-gated write policy works
- [ ] Beta channel build shipped to dogfood users
- [ ] M2 Beta milestone declared

## Deferred from this phase

- Parallel step execution via sub-agents (Phase 20)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
