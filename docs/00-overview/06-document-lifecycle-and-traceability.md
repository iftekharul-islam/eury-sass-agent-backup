# Document Lifecycle and Traceability

Spec-Version: 1.1.0

This is the release-governance map for Eury Agent. It connects product capability families to the decision, normative contract, delivery phase, verification evidence, and accountable role. It is a control document: implementations and releases MUST update its affected row.

## Operating rules

1. A `draft` row may inform discovery but MUST NOT be used to justify a production dependency.
2. A row becomes `implemented` only when its linked contract has code and automated tests.
3. A row becomes `verified` only when its listed evidence is green in CI or has a dated operational record.
4. A release may not claim a capability that is absent from this matrix or marked below its required lifecycle state.
5. The accountable role owns review; it does not mean that role performs every implementation task.

## Traceability matrix

| Capability family | Decision / contract | Delivery | Verification evidence | Accountable role | Lifecycle |
|---|---|---|---|---|---|
| Product definition and packaging | [personas](../01-product/01-personas-and-jobs.md), [feature catalog](../01-product/02-feature-catalog.md), [pricing](../01-product/04-pricing-and-packaging.md), [legacy inventory](../01-product/06-legacy-feature-inventory.md) | 1 | Product-contract check; 100% deprecated-app classification | Product | implemented |
| Embedded local agent | [ADR-0001](../02-architecture/adr/0001-embed-cersei-in-desktop.md), [runtime](../04-specs/01-agent-runtime-spec.md) | 4–7 | Runtime state-machine/property tests; latency benchmark | Desktop | approved |
| Tool safety and approvals | [tool catalog](../04-specs/02-tool-catalog-spec.md), [policy](../03-security/03-permission-and-policy-engine.md) | 6–7 | Tool conformance, policy merge fuzzing, security suite | Security | approved |
| Local data and recovery | [ADR-0004](../02-architecture/adr/0004-sqlite-local-store-and-encryption.md), [local data model](../04-specs/05-local-data-model.md) | 5, 18 | Migration, corruption, backup/restore tests | Desktop | approved |
| Model routing and gateway | [BYOK ADR](../02-architecture/adr/0005-byok-vs-managed-gateway.md), [cloud API](../04-specs/06-cloud-api-contract.md), [model governance](../02-architecture/07-provider-and-model-governance.md) | 11 | Provider contract fixtures, routing/eval gates, cost tests | Backend | draft |
| Web and multimodal work | [tool catalog](../04-specs/02-tool-catalog-spec.md), [multimodal spec](../04-specs/17-multimodal-and-attachment-spec.md) | 11 | Attachment corpus, provider fixtures, privacy and policy tests | Security | draft |
| Workspace retrieval and memory | [index ADR](../02-architecture/adr/0009-index-and-retrieval-strategy.md), [retrieval](../04-specs/09-indexing-and-retrieval-spec.md), [memory](../04-specs/08-memory-spec.md) | 15–16 | Recall/eval suite, freshness and offline tests | Desktop | approved |
| Enterprise identity and policy | [identity](../06-enterprise/01-identity-sso-scim.md), [workspace policies](../06-enterprise/03-workspace-policies.md) | 24–25 | SSO/SCIM contracts, policy enforcement matrix, audit export tests | Security | approved |
| Audit, privacy, and residency | [privacy](../03-security/07-privacy-and-data-residency.md), [audit](../06-enterprise/04-audit-and-retention.md) | 24–25 | Redaction, residency routing, retention/deletion tests | Security | approved |
| Desktop UX and accessibility | [visual language](../05-ui/00-visual-language.md), [accessibility](../05-ui/07-accessibility-and-i18n.md) | 3, 8, 13 | Visual regression, keyboard, axe, manual screen-reader testing | Design | approved |
| Release compatibility and resilience | [compatibility](../07-ops/09-compatibility-and-lifecycle.md), [release management](../07-ops/08-release-management.md) | 27–29 | Upgrade/rollback, DR drill, error-budget and game-day records | Platform | draft |

## Feature delivery traceability

The [feature catalog](../01-product/02-feature-catalog.md) is the forward map
from `F-nnn` to owning phase. Every generated `phase-NN.md` contains a
`Feature IDs` section as the inverse map. The inverse map is maintained in
`09-roadmap/generate_phases.py`, and `pnpm product:check` fails when a catalog
feature is absent from any phase in its declared delivery range.

## Review evidence

The pull request for an affected row MUST link the changed contract, its tests/evals, and any dashboard, drill, or release artifact required by the row. A failed verification regresses `verified` to `implemented` until evidence is restored.

## Related documents

- [Documentation conventions](04-doc-conventions.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Roadmap overview](../09-roadmap/00-roadmap-overview.md)
