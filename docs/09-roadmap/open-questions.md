# Open Questions

Spec-Version: 1.3.0

Decisions that are deliberately deferred, plus decisions that were open during the documentation phase and have since been closed. Question IDs are stable and are referenced from other documents, so an ID is never reused.

## Closing process

1. Raise or revisit the question at the target phase kickoff.
2. If the answer changes an interface, a boundary, or a cost profile, write an ADR.
3. Update this table to **Closed**, with the resolution and the ADR or spec link.
4. Update every document that references the question ID.

## Open

| ID | Question | Options | Decision owner | Target phase |
|---|---|---|---|---|
| Q05 | Should a user be able to belong to multiple organizations? | Single org (current platform model) / multi-org with an active-org switcher | Product + Backend | 24 |
| Q07 | Do we add language-server intelligence (completion, hover, rename) to the editor? | No (agent-first, stay lean) / LSP for the top 5 languages / defer to the separate IDE product | Product | Post-GA |
| Q09 | How long do legacy `/code/*` endpoints run after Agent GA? | 6 months (planned) / 12 months / until measured usage is zero | Product + Backend | 29 |
| Q10 | Does the Agent surface ever need to scale independently of the main Nest app? | Keep as a module (current) / extract to a service if gateway traffic dominates | Platform | Post-GA, traffic-driven |
| Q11 | How much of the desktop UI ships translated at GA? | English + Bangla shell only / English + Bangla full app / add more locales | Product | 28 |
| Q12 | Self-hosted and air-gapped pricing model | Per-seat / flat platform fee / contracted | Product | 27 |
| Q13 | Do we bundle the embedding model or download it on demand? | Bundle (installer size) / on-demand download / lexical-only default | Desktop | 15 |
| Q14 | Do we offer a headless CLI for CI use, and when? | No / read-only CLI / full run CLI | Product | Post-GA |
| Q15 | Overage billing for Enterprise beyond budget ceilings | Hard stop / metered overage / contracted ceiling | Product + Billing | 25 |
| Q16 | Retention default for Enterprise audit in the cloud | 90 days / 12 months / customer-chosen at signup | Security + Product | 25 |
| Q21 | Default retention for managed image attachments and generated assets | No cloud retention / 24 hours / conversation retention / customer-configured | Security + Product | 11 |
| Q22 | Which provider/model combinations are approved for Internal and Restricted source code by region? | Provider-specific allowlists and residency matrix | Security + Platform | 11 |
| Q23 | What support response and service-credit commitments are sold with Enterprise? | Business-hours support / 24×7 severity-based support / contracted custom SLA | Product + Support + Legal | 29 |
| Q24 | Which monthly-token and concurrent-run defaults accompany each managed package? | Fixed public limits / dynamic fair-use limits / contract-only above Pro | Product + Billing | 11 |

## Closed

| ID | Question | Resolution | Reference |
|---|---|---|---|
| Q01 | Monaco or CodeMirror 6 for the editor? | CodeMirror 6 — roughly a seventh of the bundle size, faster mount, better decoration API for diff overlays | [editor/terminal/preview](../05-ui/06-editor-terminal-preview.md) |
| Q02 | Database encryption approach | Application-level encryption with the key in the OS keychain; the database file alone is useless if copied | [ADR-0004](../02-architecture/adr/0004-sqlite-local-store-and-encryption.md) |
| Q03 | Embedding provider for the semantic index | Local only. No repository content goes to a hosted embedding service; lexical search is the reliable floor | [ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md) |
| Q04 | Where does Cersei live? | Embedded in the desktop Rust core behind the `AgentEngine` trait; no sidecar, no service hop | [ADR-0001](../02-architecture/adr/0001-embed-cersei-in-desktop.md) |
| Q06 | Cersei built-in filesystem tools or our own? | Our own, routed through `agent-sandbox`, so every filesystem and process access has one enforced chokepoint | [sandbox model](../03-security/02-sandbox-model.md) |
| Q08 | Real-time collaboration inside the Agent window? | No for v1. Optional per-user sync only; collaboration stays in the web product | [non-goals](../01-product/05-non-goals.md) |
| Q17 | Extend the legacy `DesktopRelease` model or create our own? | Own `AgentRelease` table. No legacy model is extended, aliased, or read | [cloud data model](../04-specs/07-cloud-data-model.md) |
| Q18 | Reuse the legacy `/auth/ide/*` device flow? | No. The Agent module owns its own auth surface, tables, and guards | [module structure](../04-specs/16-backend-module-structure.md) |
| Q19 | Separate Agent microservice or a module in the existing Nest app? | Module. A separate service adds a hop, a deploy target, and an on-call surface for a thin control plane | [cloud architecture](../02-architecture/03-cloud-architecture.md) |
| Q20 | Linux packaging targets for v1 | `.AppImage` and `.deb` built on Ubuntu 22.04 for glibc portability; `.rpm` deferred | [packaging](../07-ops/03-packaging-signing-notarization.md) |

## Related documents

- [Risk register](risk-register.md)
- [Roadmap overview](00-roadmap-overview.md)
- [Doc conventions](../00-overview/04-doc-conventions.md)
