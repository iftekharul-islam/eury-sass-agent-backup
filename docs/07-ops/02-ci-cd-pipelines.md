# CI/CD Pipelines

Spec-Version: 1.3.0

Agent workflows are separate files with Agent-specific names, so they never interfere with existing backend/frontend/IDE pipelines.

## Workflows

| File | Trigger | Purpose |
|---|---|---|
| `agent-ci.yml` | PR touching `agent/**`, push to `main` | Lint, typecheck, unit, integration, build smoke |
| `agent-e2e.yml` | PR label `e2e`, nightly | Playwright/WebDriver end-to-end on 3 OSes |
| `agent-eval.yml` | Nightly, pre-release | Agent eval harness against pinned models |
| `agent-bench.yml` | Weekly on `main`, pre-release | Performance benchmarks with regression gate |
| `agent-security.yml` | PR, weekly | `cargo audit`, `pnpm audit`, gitleaks, Semgrep, license check |
| `agent-release.yml` | Tag `agent-v*` | Sign, package, notarize, SBOM, publish, register release |
| `agent-backend-ci.yml` | PR touching `backend/src/modules/agent/**` | Nest unit + contract tests + module isolation check |
| `agent-docs-check.yml` | PR touching `agent/docs/**`, `agent/README.md`, or `agent/mockups/**` | Links, terminology, versions, generated phases, traceability, stale-reference checks |

Path filters keep unrelated PRs fast; a change to `agent/**` never triggers the IDE pipeline and vice versa.

## `agent-ci.yml`

```
jobs:
  lint-web      → pnpm biome/eslint, tsc --noEmit, i18n key check
  lint-rust     → cargo fmt --check, cargo clippy -D warnings
  test-web      → vitest run --coverage, jest-axe component suite
  test-rust     → cargo nextest run (workspace), cargo test --doc
  contract      → event-protocol + IPC golden fixture diff + product-contract validation
  build         → tauri build --debug on ubuntu (fast smoke)
  build-matrix  → tauri build on macos-14, windows-2022 (main + release PRs only)
```

| Concern | Rule |
|---|---|
| Runners | `ubuntu-latest`, `macos-14` (Apple Silicon), `windows-2022` |
| Toolchain | Pinned via `rust-toolchain.toml` and `.node-version`; no floating `latest` |
| Cache | `Swatinem/rust-cache`, pnpm store cache, Tauri target cache keyed by lockfile hash |
| Concurrency | Cancel in-progress runs per branch |
| Timeouts | 20 min per job; 45 min for `build-matrix` |
| Flake policy | No automatic retries on test jobs; a flaky test is quarantined with an issue, not retried |
| Required to merge | `lint-*`, `test-*`, `contract`, `build`, `agent-security.yml`, `agent-docs-check.yml` when documentation paths change |

Full-matrix Tauri builds are expensive, so PRs get one debug build and `main` gets all three. A release tag always builds all platforms from scratch with caches disabled.

## Backend CI

`agent-backend-ci.yml` runs the Agent module's unit tests, `/agent/v1/*` contract tests against a throwaway Postgres and Redis, `prisma validate` plus `prisma migrate diff` to prove migrations are additive, and the **module isolation check**:

```
fail if backend/src/modules/agent/** imports from
  ../auth/  ../code/  ../eury/  ../billing/  ../organizations/  ../admin/
  (except type-only imports of Prisma-generated types)
```

That check is the automated form of rule R3 in [backend module structure](../04-specs/16-backend-module-structure.md), and it is a Phase 25 exit criterion.

## Release pipeline

```
tag agent-vX.Y.Z
  ├─ verify: tag version == package.json == tauri.conf.json == Cargo.toml
  ├─ build (macos-14, windows-2022, ubuntu-latest), caches disabled
  ├─ sign: Developer ID (mac) / Authenticode (win) / GPG detached (linux)
  ├─ notarize + staple (mac), wait for ticket
  ├─ generate SBOM (SPDX) + checksums + provenance attestation
  ├─ sign the update manifest with the release key
  ├─ upload artifacts to agent-releases/<version>/
  ├─ POST /agent/v1/admin/releases  (create, inactive)
  ├─ smoke: download each artifact, verify checksum + signature, launch --verify-install
  └─ create GitHub Release (draft) with notes, SBOM, checksums
```

Activation to `stable` is a **manual** step in the admin console after smoke tests and staged rollout ([release management](08-release-management.md)). CI never activates a stable release automatically.

## Secrets in CI

| Secret | Used by | Scope |
|---|---|---|
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD` | release | macOS signing |
| `APPLE_ID`, `APPLE_APP_PASSWORD`, `APPLE_TEAM_ID` | release | notarization |
| `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` | release | Authenticode |
| `AGENT_UPDATE_SIGNING_KEY` | release | Manifest signing |
| `AGENT_RELEASE_UPLOAD_TOKEN` | release | Admin API upload |
| `EVAL_PROVIDER_KEYS` | eval | Nightly eval only, spend-capped |

All release secrets live in a protected environment requiring manual approval, are unavailable to PRs from forks, and are never exposed to test or eval jobs.

## Supply chain controls

- Lockfiles committed; `pnpm install --frozen-lockfile` and `cargo build --locked`.
- Dependency review on PRs; new dependencies need justification in the PR description.
- `cargo-deny` for licenses and duplicate/yanked crates.
- Third-party GitHub Actions pinned to commit SHAs.
- Build provenance attestation attached to every release artifact.
- Weekly automated dependency-update PRs, batched by ecosystem.

Details: [supply chain and signing](../03-security/06-supply-chain-and-signing.md).

## Branch protection

Required: CI green, one review (two for `agent/src-tauri/**`, `agent-sandbox`, `agent-policy`, and the backend Agent auth files), linear history, signed commits on `main`, no force-push, and up-to-date-with-base before merge.

## Artifacts and retention

| Artifact | Retention |
|---|---|
| Test/coverage reports | 30 days |
| Build logs | 90 days |
| Installers (CI) | 14 days (permanent copy in object storage) |
| SBOM, checksums, provenance | Permanent, attached to the release |
| Benchmark/eval JSON results | Permanent, appended to a trend dataset |

## Local parity

`pnpm agent:ci` runs the same lint/test/contract steps locally. Pre-commit hooks run formatters and the fast lint set only; CI remains the source of truth.

`pnpm docs:check` runs the documentation gate defined in [documentation conventions](../00-overview/04-doc-conventions.md). It verifies links and headings, document lifecycle metadata, terminology/API registries, roadmap generation, and traceability references without modifying files.

`pnpm product:check` validates Phase 1 contracts: unique `F-nnn` and `L-nnn`
identifiers, catalog metadata and phase ranges, canonical mode parity, known
entitlement keys, and complete legacy dispositions.

## Related documents

- [Packaging, signing, notarization](03-packaging-signing-notarization.md)
- [Release management](08-release-management.md)
- [Test strategy](../08-quality/01-test-strategy.md)
- [Backend module structure](../04-specs/16-backend-module-structure.md)
