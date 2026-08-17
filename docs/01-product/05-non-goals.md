# Non-Goals

Spec-Version: 1.1.0

Explicit scope exclusions to prevent creep.

**Decision owners:** Product + Engineering · **Status:** Approved for v1 scope · **Review trigger:** an ADR with measurable customer evidence and a named target phase

## Approval evidence

| Gate | Evidence | Recorded |
|---|---|---|
| Product scope | Project owner authorized implementation of the Phase 1 product-definition plan containing these exclusions | 2026-08-16 |
| Engineering consistency | `pnpm product:check` and the full Phase 0 check suite pass against the catalog, modes, entitlements, roadmap, and legacy inventory | 2026-08-16 |
| Security implementation | Deferred to Phase 2; this approval does not claim controls are implemented | Phase 2 |

## Product

| ID | Exclusion | Rationale | Revisit owner |
|---|---|---|---|
| NG-001 | General-purpose chat replacement | Home supports assistant work, but web Eury remains the broad chat product | Product |
| NG-002 | Browser-based/cloud-VM IDE | Local-first execution and trust boundaries are product differentiators | Product + Security |
| NG-003 | Foundation-model training/fine-tuning | Eury governs and routes models; it does not train them | AI Platform |
| NG-004 | Hosting user applications | Browser preview targets a user-owned local dev server only | Platform |
| NG-005 | CI/CD replacement | The agent may invoke tests and workflows but never becomes deployment authority | Platform + Security |
| NG-006 | Legal/compliance advice automation | Enterprise controls produce evidence, not professional advice | Legal + Product |

## Technical

| ID | Exclusion | Rationale | Revisit owner |
|---|---|---|---|
| NG-010 | Agent loop on NestJS servers | The embedded Rust runtime is the latency and local-control boundary | Architecture |
| NG-011 | Repository storage on Eury servers by default | Only explicit references/sync and selected model context may cross the boundary | Security + Product |
| NG-012 | Python runtime in the production path | One Rust execution path avoids duplicate security semantics | Desktop |
| NG-013 | Maintaining `code-old` after sunset | It is an inventory/migration source, never a second product line | Product + Engineering |
| NG-014 | Markdown-fence tool-call parsing | Structured provider tool calls are required for correctness | Agent Runtime |

## Platform

| ID | Exclusion | Rationale | Revisit owner |
|---|---|---|---|
| NG-020 | Mobile clients in v1 | Coding tools, local sandbox, and desktop context are required | Product |
| NG-021 | VS Code extension in v1 | Desktop quality and security ship before another host surface | Product + Desktop |
| NG-022 | Real-time multiplayer editing | Collaboration remains a separate Eury product | Product |

## Enterprise (deferred past Phase 29)

- FedRAMP certification
- Hardware security module (HSM) integration for desktop
- Custom on-prem LLM training

## When to revisit

Items move off this list only via ADR + product sign-off.

The complete preserve/improve/replace/drop decision record for the deprecated
application is the [legacy feature inventory](06-legacy-feature-inventory.md).

## Related documents

- [../00-overview/01-vision-and-scope.md](../00-overview/01-vision-and-scope.md)
- [06-legacy-feature-inventory.md](06-legacy-feature-inventory.md)
