# Phase 02 — Security Foundation

Spec-Version: 1.2.0

**Track:** A — Foundations · **Estimated size:** 1–2 weeks · **Milestone:** M0 Docs

## Goal

Produce the threat model, sandbox design, policy model, and secret handling rules that constrain every later phase — before any tool can touch a filesystem.

## Why this phase exists here

Retrofitting containment onto a working agent is how agents ship with escapes. The sandbox and policy design must exist before the tool layer, or the tool layer will be written against the wrong assumptions.

## In scope

- Threat model: assets (A-001..A-009), actors (ACT-001..ACT-007), boundaries (B-001..B-006), controls (C-001..C-016), ranked threats (T-001..T-016)
- Layered sandbox design for macOS (Seatbelt), Linux (Landlock+seccomp), Windows (restricted token+Job object) with fail-closed degradation
- Canonical machine-readable schemas: `security-types.schema.json`, `workspace-policy.schema.json`, `sandbox-capabilities.schema.json`
- Permission model: tool classes, grant scopes, deny-by-default, fail-closed rules, and monotonic trust
- Secret handling: canonical inventory (SEC-001..SEC-010), OS keychain, no-plaintext-fallback, and redaction
- Prompt injection defense strategy: `ContextBlock`, `ContextProvenance`, and the untrusted-content model
- Supply chain requirements: pinned deps, SPDX 2.3 SBOM, signing, `deny.toml`, `.gitleaks.toml`, `agent-security.yml`, Dependabot, CODEOWNERS
- Executable attack corpora (56 cases) in `agent/tests/fixtures/security/` with schema and manifest
- Semgrep security rules and positive/negative test fixtures
- Security checklists in PR template, Definition of Done, and release management

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- Implementation of the sandbox (Phase 5)
- Policy engine code (Phase 7)
- Penetration testing (Phase 28)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D2.1 | Threat model with ranked risks and named mitigations | [threat model](../03-security/01-threat-model.md) |
| D2.2 | Sandbox design per platform, including documented degradation | [sandbox model](../03-security/02-sandbox-model.md) |
| D2.3 | Permission and policy engine design with merge semantics and JSON schemas | [policy engine](../03-security/03-permission-and-policy-engine.md) |
| D2.4 | Secrets and key management rules, inventory, no-plaintext-fallback, and redaction | [secrets](../03-security/04-secrets-and-key-management.md) |
| D2.5 | Prompt injection defense plan and the initial attack corpus | [injection defense](../03-security/05-prompt-injection-defense.md) |
| D2.6 | Supply chain controls wired into CI (`cargo audit`, `cargo deny`, `pnpm audit`, gitleaks, Semgrep) | [supply chain](../03-security/06-supply-chain-and-signing.md) |
| D2.7 | Custom Semgrep rules and positive/negative test fixtures | [security testing](../08-quality/04-security-testing.md) |
| D2.8 | Attack test corpora and manifest under `tests/fixtures/security/` | [security testing](../08-quality/04-security-testing.md) |
| D2.9 | Automated security-contract check script (`scripts/check-security-contracts.mjs`) | [CI/CD](../07-ops/02-ci-cd-pipelines.md) |
| D2.10 | Per-PR, definition-of-done, and release security checklists | [definition of done](../08-quality/05-definition-of-done.md) |

## Key decisions and design notes

- Deny by default for write, execute, and network tools ([ADR-0006](../02-architecture/adr/0006-deny-by-default-permissions.md)); no dev-mode exception that could ship.
- Defense in depth: policy check, path/command guard, and OS sandbox are three independent layers, and no layer is trusted alone.
- Fail closed everywhere: missing policy, unreadable keychain, or unavailable sandbox disables privileged tools rather than proceeding.
- All model-adjacent content (file contents, tool output, web results) is untrusted data, never instructions.
- The legacy `CODE_API_TOKEN`-unset-means-open pattern is explicitly named as a prohibited anti-pattern.
- GrantScope contains only `session`, `oneTime`, and `workspace`; `deny` is strictly a policy decision.
- Child-process egress defaults off (`networkDuringExecute: false`).

## Contracts touched

- Tool classes and grant scopes used by policy and UI
- Untrusted-content marking convention used by prompt assembly
- Security checklist items referenced by CI and reviews
- Security JSON schemas (`security-types`, `workspace-policy`, `sandbox-capabilities`)
- Attack fixture schema (`security-fixture.schema.json`) and manifest

## Dependencies

- Phase 0 (CI to host the security jobs)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Platform sandbox gaps | Weaker containment on some OS | Documented degradation, path/command guard always active, capability reported in the UI |
| Over-restrictive defaults | Product feels unusable | Grant scopes and mode profiles tuned in Phase 7 with real usage, never by weakening the default |
| Injection defense treated as prompt engineering | False confidence | Defense is structural (privilege separation and approvals), with the prompt layer as a secondary measure |

## Test plan

| Layer | Coverage |
|---|---|
| Static | Semgrep rules fire on seeded violations; `rules.rs` and `rules.tsx` test fixtures |
| Supply chain | CI fails on an introduced vulnerable dependency via `cargo audit` / `deny` |
| Corpus | 56 traversal, command, injection, secrets, SSRF, MCP, and policy fixtures committed and verified |
| Security contract | `pnpm security:check` validates asset/threat/control IDs, JSON schemas, and doc consistency |

## Metrics and targets

| Metric | Target |
|---|---|
| Threat model coverage | Every asset and Critical risk has at least one named mitigation |
| Security CI jobs | Running on every PR and weekly on main |
| Attack fixture coverage | 56 machine-readable test cases across 7 categories |

## Exit criteria

- [x] Threat model reviewed and signed off with stable asset, threat, and control IDs
- [x] Sandbox design covers all three platforms with explicit fail-closed degradation
- [x] Permission model documented with canonical schemas and fail-closed semantics
- [x] Security CI workflows, `deny.toml`, `.gitleaks.toml`, Semgrep rules, and CODEOWNERS configured
- [x] 56 attack test fixtures committed under `tests/fixtures/security/`
- [x] Security checklists in PR template, definition of done, and release management
- [x] Automated security-contract check passes (`pnpm security:check`)

## Deferred from this phase

- Sandbox implementation (Phase 5)
- Policy engine implementation (Phase 7)
- External pentest (Phase 28)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
