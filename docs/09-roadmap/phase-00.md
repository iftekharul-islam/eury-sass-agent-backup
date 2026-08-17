# Phase 00 — Governance and Repo Foundation

Spec-Version: 1.2.0

**Track:** A — Foundations · **Estimated size:** 1 week · **Milestone:** M0 Docs

## Goal

Establish the `agent/` workspace, toolchain, CI skeleton, and the documentation and decision process that every later phase depends on.

## Why this phase exists here

Every convention that is cheap now becomes expensive later: crate layout, naming, lint rules, and the isolation boundary against the legacy `code` stack. Nothing here is user-visible, and skipping it is how the previous app accumulated its worst problems.

## In scope

- `agent/` monorepo workspace: `apps/desktop`, `crates/*`, `docs/`, `bench/`, `eval/`, `tests/`
- Rust workspace with pinned toolchain (`rust-toolchain.toml`) and shared lint config
- pnpm workspace, pinned Node version, TypeScript strict mode
- Formatting and lint gates: `cargo fmt`, `cargo clippy -D warnings`, Biome/ESLint, `tsc --noEmit`
- `agent-ci.yml` skeleton running lint and unit jobs on three OSes
- ADR process, doc conventions, `Spec-Version` headers, changelog format
- Naming and migration map as the enforced source of truth for identifiers
- CODEOWNERS with security-sensitive paths requiring two reviewers
- The design mockup for all ten screens, so UI decisions are settled before any component exists

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- Any product feature
- Cersei integration
- Backend changes

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D0.1 | Cargo workspace with the eight crates stubbed and building | [ADR-0007](../02-architecture/adr/0007-rust-workspace-crate-split.md) |
| D0.2 | Tauri 2 + React 19 + Tailwind 4 app that opens an empty window | [desktop runtime](../02-architecture/02-desktop-runtime.md) |
| D0.3 | `agent-ci.yml` with lint, typecheck, unit, debug-build jobs | [CI/CD](../07-ops/02-ci-cd-pipelines.md) |
| D0.4 | Lint rule set including no-`unwrap` and no-`std::fs`-outside-sandbox stubs | [security testing](../08-quality/04-security-testing.md) |
| D0.5 | ADR template and the first ten ADRs committed | [doc conventions](../00-overview/04-doc-conventions.md) |
| D0.6 | Naming and migration map published and referenced from the README | [naming map](../00-overview/05-naming-and-migration-map.md) |
| D0.7 | CODEOWNERS, PR template with the feature-done checklist | [definition of done](../08-quality/05-definition-of-done.md) |
| D0.8 | `version.json` and the version-consistency CI check | [packaging](../07-ops/03-packaging-signing-notarization.md) |
| D0.9 | Clickable design mockup covering all ten screens, offline, no build step | [mockups](../../mockups/README.md), [visual language](../05-ui/00-visual-language.md) |
| D0.10 | Token-parity check: the mockup's `:root` block matches the design system table | [design system](../05-ui/01-design-system.md) |

## Key decisions and design notes

- Crate split is fixed now because moving module boundaries later invalidates every import and review habit.
- Toolchain versions are pinned, never floating — a silent compiler upgrade is an unreviewed change.
- `clippy -D warnings` from commit one; retrofitting lint cleanliness never happens.
- The `agent/` tree is self-contained: no build step reaches into `backend/`, `frontend/`, `ide/`, or `code-old/`.
- The mockup ships in Phase 0, not alongside the UI phases. Settling layout, tokens, and approval treatment on day one is what stops Phase 5 from re-litigating them, and a single offline HTML file is the cheapest artifact that survives review.

## Contracts touched

- Repository layout and crate names
- CI job names (referenced by branch protection)
- Doc conventions and `Spec-Version` semantics

## Dependencies

- None — this is the entry point

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Premature crate boundaries | Churn in later phases | Boundaries follow ADR-0007, which was derived from the tool/policy/store separation the design already requires |
| CI cost on three OSes | Slow PRs | Path filters, caching, and full matrix only on `main` and release PRs |
| Toolchain drift between contributors | Works-on-my-machine bugs | Pinned toolchain files plus a `--verify-install` style doctor script |

## Test plan

| Layer | Coverage |
|---|---|
| Build | Workspace builds clean on macOS, Windows, Linux |
| Lint | Formatting, clippy, ESLint, and typecheck gates fail on a seeded violation |
| Meta | Version-consistency check fails on a deliberate mismatch |
| Meta | Token-parity check fails when a design-system token and the mockup disagree |

## Metrics and targets

| Metric | Target |
|---|---|
| Cold CI run | < 12 min |
| Cached CI run | < 5 min |
| Empty-app cold start | < 1 s |

## Exit criteria

- [ ] Workspace builds and lints clean on all three platforms
- [ ] CI runs on PRs with required checks configured in branch protection
- [ ] Empty Tauri window launches on all three platforms
- [ ] ADR process documented and the initial ADRs merged
- [ ] Naming and migration map merged and linked from the README
- [ ] PR template enforces the feature-done checklist
- [ ] Mockup opens offline in a browser and covers all ten wireframes, in both themes and all five accents

## Deferred from this phase

- Release signing (Phase 27)
- E2E harness (Phase 28)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
- [Mockups](../../mockups/README.md)
- [Visual language](../05-ui/00-visual-language.md)
