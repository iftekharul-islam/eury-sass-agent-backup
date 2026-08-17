# Phase 29 — GA Launch

Spec-Version: 1.1.0

**Track:** F — Enterprise and GA · **Estimated size:** 2 weeks · **Milestone:** M4 GA

## Goal

Ship 1.0: documentation, migration path off the deprecated app, support readiness, pricing live, legacy sunset communicated, and a staged public rollout.

## Why this phase exists here

Launch is an operational project, not a build step. Everything technical is done; this phase makes it supportable, purchasable, and survivable.

## In scope

- Public documentation site generated from these specs, with a getting-started path
- Migration guide from `code-old`, tested by someone who did not write it
- Marketing download page at `/{locale}/eury-agent` with checksums
- Pricing and checkout live for all tiers with entitlement verification
- Support readiness: runbooks, on-call rotation, escalation, game-day exercise
- Status page automation wired to SLO alerts
- Legacy sunset announcement with dates and deprecation headers on `/code/*`
- Staged public rollout with go/no-go criteria at each step
- Post-launch review, roadmap for 1.1, and a bug bounty program

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- New features

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D29.1 | Public docs site with getting started, guides, and reference | [doc conventions](../00-overview/04-doc-conventions.md) |
| D29.2 | Tested migration guide from the deprecated app | [naming map](../00-overview/05-naming-and-migration-map.md) |
| D29.3 | Download page with per-platform artifacts and verifiable checksums | [packaging](../07-ops/03-packaging-signing-notarization.md) |
| D29.4 | Pricing and checkout live with entitlements enforced end to end | [pricing](../01-product/04-pricing-and-packaging.md) |
| D29.5 | Support runbooks, rotation, and a completed game-day | [runbooks](../07-ops/06-runbooks.md) |
| D29.6 | Status page automation driven by SLO alerts | [observability](../07-ops/05-observability-and-slos.md) |
| D29.7 | Legacy sunset announcement and deprecation headers on `/code/*` | [naming map](../00-overview/05-naming-and-migration-map.md) |
| D29.8 | Staged rollout executed with documented go/no-go decisions | [release management](../07-ops/08-release-management.md) |
| D29.9 | Security disclosure policy and bug bounty launch | [incident response](../03-security/09-incident-response.md) |
| D29.10 | Post-launch review and the 1.1 roadmap | [roadmap overview](00-roadmap-overview.md) |

## Key decisions and design notes

- Docs are generated from these specs so documentation drift is a build failure rather than a discovery.
- The migration guide is validated by someone uninvolved in writing it — self-tested instructions are not tested.
- Legacy `/code/*` keeps working for six months after GA; the sunset date is announced at launch, not later.
- Rollout is staged with explicit go/no-go criteria, and stopping is a normal outcome rather than a failure.

## Contracts touched

- Public documentation URLs
- Download page artifact naming
- Deprecation header format on legacy endpoints

## Dependencies

- Phase 27 (signed releases)
- Phase 28 (quality gates)
- Phase 25 (billing controls)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Launch-day load | Outage | Load tested at 2× projected peak; staged rollout; scaling headroom pre-provisioned |
| Support volume | Slow responses | Runbooks, diagnostic bundle command, and a triaged FAQ from beta feedback |
| Migration friction | Users stranded on the old app | Tested guide, export/import path, and six months of overlap |
| Pricing or entitlement bugs | Revenue and trust | End-to-end entitlement tests per tier before launch |
| Security report on day one | Reputation | Disclosure policy, bounty, and an incident process rehearsed in the game-day |

## Test plan

| Layer | Coverage |
|---|---|
| End to end | Purchase → install → login → first successful run, per tier |
| Migration | Deprecated app data exported and the new app set up following only the written guide |
| Load | 2× projected launch peak on auth, manifest, and gateway |
| Ops | Game-day covering a gateway outage, a bad release, and a security report |

## Metrics and targets

| Metric | Target |
|---|---|
| First-run success rate | > 95% |
| Install to first successful run | < 5 min median |
| Launch-week SEV1 count | 0 |
| Support contacts per 1000 active users | < 20 |
| Crash-free sessions | > 99.5% |

## Exit criteria

- [ ] Docs site published and accurate against shipped behavior
- [ ] Migration guide validated by an independent tester
- [ ] Pricing live with entitlements enforced for every tier
- [ ] Support rotation staffed with a completed game-day exercise
- [ ] Legacy sunset announced with dates and deprecation headers live
- [ ] Staged rollout completed to 100% with no open SEV1/2
- [ ] M4 GA milestone declared

## Deferred from this phase

- 1.1 feature work
- Additional locales beyond English and Bangla

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
