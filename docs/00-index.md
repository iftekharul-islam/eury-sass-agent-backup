# Documentation Index

Spec-Version: 1.4.0

Complete map of the Eury Agent specification set: **120 documents** (90 specs and guides, 30 phase plans), plus the clickable [design mockup](../mockups/README.md). Read in section order for context, or jump to the table that matches your task.

## Start here

| If you are… | Read |
|---|---|
| New to the project | [vision and scope](00-overview/01-vision-and-scope.md) → [system architecture](02-architecture/01-system-architecture.md) → [roadmap overview](09-roadmap/00-roadmap-overview.md) |
| Implementing a phase | The phase file, then every spec it references |
| Writing backend code | [backend module structure](04-specs/16-backend-module-structure.md) → [cloud API contract](04-specs/06-cloud-api-contract.md) → [cloud data model](04-specs/07-cloud-data-model.md) |
| Writing desktop Rust | [desktop runtime](02-architecture/02-desktop-runtime.md) → [agent runtime](04-specs/01-agent-runtime-spec.md) → [sandbox model](03-security/02-sandbox-model.md) |
| Writing UI | [the mockup](../mockups/README.md) → [visual language](05-ui/00-visual-language.md) → [design system](05-ui/01-design-system.md) → [app shell](05-ui/02-app-shell-and-navigation.md) → [chat UX](05-ui/03-chat-and-streaming-ux.md) |
| Renaming anything | [naming and migration map](00-overview/05-naming-and-migration-map.md) |
| Reviewing a PR | [definition of done](08-quality/05-definition-of-done.md) |
| Reading Bangla | [README.bn.md](README.bn.md) |

## 00 Overview (6)

| # | Document | Contents |
|---|---|---|
| 01 | [Vision and scope](00-overview/01-vision-and-scope.md) | What we build, what we replace, stakeholders |
| 02 | [Glossary](00-overview/02-glossary.md) | Shared vocabulary |
| 03 | [Competitive landscape](00-overview/03-competitive-landscape.md) | What we must beat, and the anti-patterns we avoid |
| 04 | [Doc conventions](00-overview/04-doc-conventions.md) | Spec versions, linking, terminology rules |
| 05 | [Naming and migration map](00-overview/05-naming-and-migration-map.md) | Authoritative `code` → `agent` rename table |
| 06 | [Document lifecycle and traceability](00-overview/06-document-lifecycle-and-traceability.md) | Ownership, lifecycle states, and capability-to-evidence map |

## 01 Product (6)

| # | Document | Contents |
|---|---|---|
| 01 | [Personas and jobs](01-product/01-personas-and-jobs.md) | Who it is for |
| 02 | [Feature catalog](01-product/02-feature-catalog.md) | Every feature with priority, size, and phase |
| 03 | [Modes and workflows](01-product/03-modes-and-workflows.md) | chat, agent, plan, ask, build as permission profiles |
| 04 | [Pricing and packaging](01-product/04-pricing-and-packaging.md) | Tiers and entitlements |
| 05 | [Non-goals](01-product/05-non-goals.md) | What we deliberately will not build |
| 06 | [Deprecated-app feature inventory](01-product/06-legacy-feature-inventory.md) | Source-backed preserve/improve/replace/drop classification |

## 02 Architecture (7 + 10 ADRs)

| # | Document | Contents |
|---|---|---|
| 01 | [System architecture](02-architecture/01-system-architecture.md) | Components, trust boundaries, data flows, NFRs |
| 02 | [Desktop runtime](02-architecture/02-desktop-runtime.md) | Process model, crates, paths, lifecycle |
| 03 | [Cloud architecture](02-architecture/03-cloud-architecture.md) | Control plane, the self-contained module, auth, gateway |
| 04 | [Agent engine abstraction](02-architecture/04-agent-engine-abstraction.md) | The `AgentEngine` trait and Cersei adapter |
| 05 | [Latency budget](02-architecture/05-latency-budget.md) | Where every millisecond goes |
| 06 | [Offline and degraded modes](02-architecture/06-offline-and-degraded-modes.md) | Behavior without the cloud |
| 07 | [Provider and model governance](02-architecture/07-provider-and-model-governance.md) | Data-class routing, provider/model rollout, fallback, and eval gates |

| ADR | Decision |
|---|---|
| [0001](02-architecture/adr/0001-embed-cersei-in-desktop.md) | Embed Cersei in the desktop core |
| [0002](02-architecture/adr/0002-tauri-over-electron-and-pyside.md) | Tauri over Electron and PySide |
| [0003](02-architecture/adr/0003-agent-engine-trait-boundary.md) | `AgentEngine` trait boundary |
| [0004](02-architecture/adr/0004-sqlite-local-store-and-encryption.md) | Encrypted SQLite local store |
| [0005](02-architecture/adr/0005-byok-vs-managed-gateway.md) | BYOK and managed gateway together |
| [0006](02-architecture/adr/0006-deny-by-default-permissions.md) | Deny-by-default permissions |
| [0007](02-architecture/adr/0007-rust-workspace-crate-split.md) | Rust workspace crate split |
| [0008](02-architecture/adr/0008-event-protocol-over-tauri-channels.md) | Event protocol over Tauri channels |
| [0009](02-architecture/adr/0009-index-and-retrieval-strategy.md) | Local index and hybrid retrieval |
| [0010](02-architecture/adr/0010-multi-agent-orchestration-model.md) | Multi-agent orchestration model |

## 03 Security (9)

| # | Document | Contents |
|---|---|---|
| 01 | [Threat model](03-security/01-threat-model.md) | Assets, actors, surfaces, mitigations |
| 02 | [Sandbox model](03-security/02-sandbox-model.md) | Path guard, command guard, OS sandboxing |
| 03 | [Permission and policy engine](03-security/03-permission-and-policy-engine.md) | Classes, scopes, merge, fail-closed |
| 04 | [Secrets and key management](03-security/04-secrets-and-key-management.md) | Keychain rules, redaction, rotation |
| 05 | [Prompt injection defense](03-security/05-prompt-injection-defense.md) | Untrusted content model |
| 06 | [Supply chain and signing](03-security/06-supply-chain-and-signing.md) | Dependencies, SBOM, provenance |
| 07 | [Privacy and data residency](03-security/07-privacy-and-data-residency.md) | What leaves the machine, and when |
| 08 | [Compliance baseline](03-security/08-compliance-baseline.md) | SOC 2, ISO, GDPR mapping |
| 09 | [Incident response](03-security/09-incident-response.md) | Severities, escalation, disclosure |

## 04 Specs (17)

This is the normative layer. Each document states its invariants, exact types, limits, error codes, and a numbered **conformance test** table. An implementation is correct when those tests pass, so these specs are the input to both the code and the test suite.

| # | Document | Contents |
|---|---|---|
| 01 | [Agent runtime](04-specs/01-agent-runtime-spec.md) | Run states, context assembly and budgets, turn loop, cancellation, compaction, persistence points |
| 02 | [Tool catalog](04-specs/02-tool-catalog-spec.md) | Every tool with schema, class, risk, limits, path normalization, truncation, redaction |
| 03 | [Event protocol](04-specs/03-event-protocol-spec.md) | Core → UI events, per-run channels, coalescing, gap recovery, versioning |
| 04 | [IPC commands](04-specs/04-ipc-command-spec.md) | Every `invoke` command with capability flags, rate limits, and versioning |
| 05 | [Local data model](04-specs/05-local-data-model.md) | Full SQLite DDL, encryption, invariants, migrations, retention, corruption handling |
| 06 | [Cloud API contract](04-specs/06-cloud-api-contract.md) | Every `/agent/v1/*` endpoint |
| 07 | [Cloud data model](04-specs/07-cloud-data-model.md) | `Agent*` Prisma models |
| 08 | [Memory](04-specs/08-memory-spec.md) | `EURY.md` hierarchy, extraction with confirmation, recall scoring, relation graph |
| 09 | [Indexing and retrieval](04-specs/09-indexing-and-retrieval-spec.md) | Index schema, chunking, ranking formula, freshness, degraded modes |
| 10 | [MCP integration](04-specs/10-mcp-integration-spec.md) | Transports, trust fingerprints, process isolation, failure handling |
| 11 | [Plan format](04-specs/11-plan-format-spec.md) | Front matter, step schema, status lifecycle, build execution |
| 12 | [Checkpoint and rollback](04-specs/12-checkpoint-and-rollback-spec.md) | Content-addressed snapshots, two-phase restore, retention |
| 13 | [Multi-agent](04-specs/13-multi-agent-spec.md) | Roles, permission intersection, worktrees, budgets |
| 14 | [Telemetry](04-specs/14-telemetry-spec.md) | Opt-in event catalog, bucketing, crash reporting |
| 15 | [Error taxonomy](04-specs/15-error-taxonomy.md) | Complete `EURY_*` registry with HTTP mapping and UI presentation |
| 16 | [Backend module structure](04-specs/16-backend-module-structure.md) | Self-contained NestJS Agent module |
| 17 | [Multimodal and attachments](04-specs/17-multimodal-and-attachment-spec.md) | Attachment lifecycle, vision, generated images, retention, and audit |

## 05 UI (9)

Plus the clickable [mockup](../mockups/README.md), which is the visual source of truth for everything in this section.

| # | Document |
|---|---|
| 00 | [Visual language](05-ui/00-visual-language.md) — what we clone, what a desktop GUI does better, and the ten wireframes |
| 01 | [Design system](05-ui/01-design-system.md) |
| 02 | [App shell and navigation](05-ui/02-app-shell-and-navigation.md) |
| 03 | [Chat and streaming UX](05-ui/03-chat-and-streaming-ux.md) |
| 04 | [Tool activity and diff UX](05-ui/04-tool-activity-and-diff-ux.md) |
| 05 | [Approval and trust UX](05-ui/05-approval-and-trust-ux.md) |
| 06 | [Editor, terminal, preview](05-ui/06-editor-terminal-preview.md) |
| 07 | [Accessibility and i18n](05-ui/07-accessibility-and-i18n.md) |
| 08 | [Keyboard and command palette](05-ui/08-keyboard-and-command-palette.md) |

## 06 Enterprise (7)

| # | Document |
|---|---|
| 01 | [Identity, SSO, SCIM](06-enterprise/01-identity-sso-scim.md) |
| 02 | [RBAC and org model](06-enterprise/02-rbac-and-org-model.md) |
| 03 | [Workspace policies](06-enterprise/03-workspace-policies.md) |
| 04 | [Audit and retention](06-enterprise/04-audit-and-retention.md) |
| 05 | [Usage, quotas, budgets](06-enterprise/05-usage-quotas-and-budgets.md) |
| 06 | [Admin console](06-enterprise/06-admin-console-spec.md) |
| 07 | [Air-gapped and self-hosted](06-enterprise/07-air-gapped-and-self-hosted.md) |

## 07 Ops (9)

| # | Document |
|---|---|
| 01 | [Environments and configuration](07-ops/01-environments-and-config.md) |
| 02 | [CI/CD pipelines](07-ops/02-ci-cd-pipelines.md) |
| 03 | [Packaging, signing, notarization](07-ops/03-packaging-signing-notarization.md) |
| 04 | [Auto-update and rollback](07-ops/04-auto-update-and-rollback.md) |
| 05 | [Observability and SLOs](07-ops/05-observability-and-slos.md) |
| 06 | [Runbooks](07-ops/06-runbooks.md) |
| 07 | [Backup and DR](07-ops/07-backup-and-dr.md) |
| 08 | [Release management](07-ops/08-release-management.md) |
| 09 | [Compatibility and lifecycle](07-ops/09-compatibility-and-lifecycle.md) |

## 08 Quality (5)

| # | Document |
|---|---|
| 01 | [Test strategy](08-quality/01-test-strategy.md) |
| 02 | [Agent eval harness](08-quality/02-agent-eval-harness.md) |
| 03 | [Performance benchmarks](08-quality/03-performance-benchmarks.md) |
| 04 | [Security testing](08-quality/04-security-testing.md) |
| 05 | [Definition of done](08-quality/05-definition-of-done.md) |

## 09 Roadmap (33)

[Overview](09-roadmap/00-roadmap-overview.md) · [Risk register](09-roadmap/risk-register.md) · [Open questions](09-roadmap/open-questions.md) · `phase-00.md` … `phase-29.md`

Phase files are generated from `09-roadmap/generate_phases.py`; edit the generator rather than the files.

## Completeness checklist

The documentation phase (M0) is complete when every row is true.

- [x] Every section has the document count listed above, with no placeholders
- [x] Every document carries a `Spec-Version` header
- [x] Ten ADRs cover every locked architectural decision
- [x] Every `/agent/v1/*` endpoint has a request shape, response shape, and error codes
- [x] Every Prisma model the Agent owns is `Agent`-prefixed, with additive-only changes to existing models
- [x] Every legacy `code` identifier has a mapped Agent replacement
- [x] No document instructs edits to a shared feature service
- [x] Every tool class has a policy field and an approval path
- [x] Every spec in section 04 has invariants, exact types, limits, and a conformance test table
- [x] One vocabulary per concept: five modes, six tool classes, four grant scopes, four risk levels
- [x] Every SLO has a metric, a dashboard, an alert, and a runbook
- [x] Every phase has deliverables, contracts, tests, measurable targets, and exit criteria
- [x] Every phase deliverable references a spec that exists
- [x] Risk register and open questions are populated with owners
- [x] Bangla executive summary present

## Conventions reminder

Every document declares `Spec-Version: MAJOR.MINOR.PATCH`. A behavior-changing edit bumps MINOR, a contract-breaking edit bumps MAJOR. When implementation reveals a spec is wrong, the fix updates the spec in the same pull request ([doc conventions](00-overview/04-doc-conventions.md)).
