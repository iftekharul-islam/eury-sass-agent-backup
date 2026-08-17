# Documentation Conventions

Spec-Version: 1.1.0

Rules for writing and maintaining Eury Agent docs.

## Language

- **Primary:** English (all specs, ADRs, roadmap).
- **Secondary:** Bangla summary at [../README.bn.md](../README.bn.md) only; do not duplicate full specs in Bangla unless explicitly requested.

## File naming

- Folders: `NN-topic/` with two-digit prefix for sort order.
- Files: `NN-kebab-case.md` within folders.
- ADRs: `02-architecture/adr/NNNN-short-title.md` (four-digit sequence).

## ADR format

Each ADR must include:

1. **Status** — Proposed | Accepted | Deprecated | Superseded by ADR-XXXX
2. **Context** — problem and forces
3. **Decision** — what we chose
4. **Consequences** — positive, negative, mitigations

## Spec format

Implementation specs (`04-specs/`) must include:

1. **Scope** — what this spec covers
2. **Normative language** — MUST, SHOULD, MAY (RFC 2119)
3. **Types / schemas** — JSON, SQL, Rust-like pseudotypes
4. **Limits** — numeric bounds (timeouts, sizes, counts)
5. **Error codes** — reference [15-error-taxonomy.md](../04-specs/15-error-taxonomy.md)
6. **Version** — spec version header `Spec-Version: 1.0.0`

## Phase document format

Each `09-roadmap/phase-NN.md` MUST contain:

```markdown
# Phase NN — Title

## Goal
## In scope
## Out of scope
## Design notes
## Deliverables
## Interfaces / contracts touched
## Dependencies
## Risks
## Test plan
## Metrics
## Exit criteria
```

## Cross-references

- Link to other docs with relative paths: `[agent runtime](../04-specs/01-agent-runtime-spec.md)`.
- Link to code (when it exists): `` `crates/agent-core/src/engine.rs` ``.
- Link to backend: `` `backend/src/modules/agent/auth/agent-auth.service.ts` ``.
- Never cite a legacy `code`-stack path as the implementation target; use the [naming and migration map](05-naming-and-migration-map.md).

## Change control

1. Architecture change → new or updated ADR before implementation.
2. API change → bump `Spec-Version` and document migration in spec changelog section.
3. Phase scope change → update phase file + `00-roadmap-overview.md`.
4. A security, privacy, model-routing, or data-residency change → security owner review before merge.
5. A documentation change that affects a traceability row → update the [traceability matrix](06-document-lifecycle-and-traceability.md) in the same PR.

## Lifecycle and ownership

Every normative document has a lifecycle state in the [traceability matrix](06-document-lifecycle-and-traceability.md):

| State | Meaning |
|---|---|
| `draft` | Direction is being explored; implementation MUST NOT depend on it |
| `approved` | Reviewers accepted the contract; implementation may begin |
| `implemented` | Code exists, but conformance evidence is incomplete |
| `verified` | Required tests/evals and operational evidence pass |
| `deprecated` | Supported only for the published transition window |

Each normative spec, ADR, and phase has a named accountable role: Product, Desktop, Backend, Security, Platform, or Design. The role is accountable for review at least once per release cycle and immediately after a related incident, security finding, provider change, or material architecture decision.

## Documentation quality gate

`agent-docs-check` is a required CI job whenever `agent/docs/**`, `agent/README.md`, or `agent/mockups/**` changes. It MUST fail on:

1. Broken relative links or links to a missing heading.
2. Missing or malformed `Spec-Version` headers on normative documents.
3. A reference to an unknown `EURY_*` error, `/agent/v1` route, tool name, mode, tool class, or grant scope.
4. A phase deliverable without a linked spec, test/eval, owner, or measurable exit criterion.
5. A feature family without a traceability row, or a traceability row pointing at a missing document.
6. Generated phase files differing from `09-roadmap/generate_phases.py`.
7. Stale document counts or migration terminology (`code` identifiers outside explicitly marked legacy references).

The job reports the offending file and rule. It does not auto-rewrite specifications.

## Diagrams

- Use Mermaid in markdown for architecture and flows.
- No proprietary diagram formats.

## Secrets in docs

- Never commit real API keys, tokens, or customer data.
- Use placeholders: `ANTHROPIC_API_KEY`, `Bearer <token>`.

## Cursor indexing

Root `.cursorignore` contains `docs/**` which may hide `agent/docs/`. Prefer anchoring ignore to `/docs/**` or add `!agent/docs/**` so agent documentation remains indexed.

## Review gates

| Doc type | Reviewer |
|----------|----------|
| ADR | Tech lead + security for security-related |
| Security | Security owner |
| Cloud API contract | Backend owner |
| UI spec | Design + frontend owner |
| Phase exit | Product + engineering sign-off |
