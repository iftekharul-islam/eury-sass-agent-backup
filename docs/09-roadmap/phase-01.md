# Phase 01 — Product Definition

Spec-Version: 1.2.0

**Track:** A — Foundations · **Estimated size:** 1 week · **Milestone:** M0 Docs

## Goal

Lock what we are building and for whom, with a feature catalog that is prioritized, sized, and traceable to a phase.

## Why this phase exists here

The deprecated app grew by accretion. Writing the catalog before the code means every later phase can point at a numbered feature, and scope arguments are settled once rather than per sprint.

## In scope

- Personas, jobs to be done, and the deprecated app's feature inventory
- Feature catalog with priority (P0–P3), size, owning phase, persona, entitlement, and lifecycle
- Mode definitions: chat, agent, plan, ask, build
- Pricing and packaging tiers mapped to entitlements
- Explicit non-goals
- Competitive analysis with the specific behaviors we intend to beat

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- Marketing site copy
- Final visual design
- Any implementation

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D1.1 | Personas and jobs document | [personas](../01-product/01-personas-and-jobs.md) |
| D1.2 | Feature catalog, every row mapped to a phase | [feature catalog](../01-product/02-feature-catalog.md) |
| D1.3 | Mode semantics: default permissions and allowed tools per mode | [modes](../01-product/03-modes-and-workflows.md) |
| D1.4 | Pricing tiers mapped to quotas, gateway access, and policy features | [pricing](../01-product/04-pricing-and-packaging.md) |
| D1.5 | Non-goals, signed off by product and engineering | [non-goals](../01-product/05-non-goals.md) |
| D1.6 | Preserve/avoid list distilled from `code-old` | [competitive landscape](../00-overview/03-competitive-landscape.md) |
| D1.7 | Authoritative `code-old` feature inventory with preserve/improve/replace/drop decisions | [legacy inventory](../01-product/06-legacy-feature-inventory.md) |
| D1.8 | Automated product-contract validation for features, modes, entitlements, and legacy coverage | [CI/CD](../07-ops/02-ci-cd-pipelines.md) |

## Key decisions and design notes

- Modes are permission profiles, not prompt presets — this is what makes `ask` safe by construction.
- The catalog is the only place feature scope lives; phase files reference it rather than redefining it.
- Anything from the old app that is not in the catalog is intentionally dropped, and the reason is recorded.

## Contracts touched

- Feature IDs (F-nnn) referenced by phase files and PRs
- Mode names used by the runtime, policy engine, and UI
- Stable entitlement and quota names used by the cloud gateway
- Legacy feature IDs (`L-nnn`) and disposition vocabulary

## Dependencies

- Phase 0 (doc conventions)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Scope creep during implementation | Slipped milestones | Feature IDs plus a phase mapping; new features enter the catalog with a phase, or they wait |
| Modes under-specified | Inconsistent permissions | Each mode ships with an explicit default tool set asserted in tests |
| Pricing assumptions wrong | Rework in Phase 25 | Entitlement names are decoupled from prices; tiers can be re-priced without code change |

## Test plan

| Layer | Coverage |
|---|---|
| Review | Every catalog row has a priority, size, and phase |
| Traceability | Every mode named in the catalog exists in the mode spec |
| Product contract | No duplicate feature IDs, invalid phases, unknown entitlements, or incomplete legacy rows |

## Metrics and targets

| Metric | Target |
|---|---|
| Catalog coverage of `code-old` features | 100% classified as preserve / improve / replace / drop |
| Features without an owning phase | 0 |

## Exit criteria

- [x] Feature catalog complete and mapped to phases
- [x] Mode semantics defined with default permissions per mode
- [x] Pricing tiers mapped to entitlements
- [x] Non-goals signed off
- [x] Every `code-old` feature classified
- [x] Automated product-contract check passes

## Deferred from this phase

- Marketing positioning (Phase 29)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
