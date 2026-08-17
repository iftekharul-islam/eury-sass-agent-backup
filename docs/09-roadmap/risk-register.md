# Risk Register

Spec-Version: 1.1.0

Scored as Likelihood × Impact. Impact levels: Low (annoyance), Medium (rework or delay), High (milestone slip or significant cost), Critical (security breach, data loss, or launch blocker).

## Technical risks

| ID | Risk | L | I | Mitigation | Owner | Phase |
|---|---|---|---|---|---|---|
| R01 | Cersei breaking API change or maintenance gap | Medium | High | `AgentEngine` trait boundary confines the blast radius to one crate; exact version pinning; an internal fallback loop can implement the same trait | Platform | 4 |
| R02 | Sandbox escape via path, symlink, or command trick | Low | Critical | Three independent layers (guards, OS sandbox, policy); traversal and command corpora on every PR; continuous fuzzing; external pentest | Security | 5, 28 |
| R03 | Prompt injection causes an unapproved privileged action | Medium | Critical | Structural defense: untrusted-content marking plus approvals for every privileged action; injection corpus gated at 100% | Security | 2, 8, 19 |
| R04 | WebView differences across platforms | Medium | Medium | Three-OS CI matrix, visual smoke tests, conservative CSS, feature detection | Desktop | 3 |
| R05 | Keychain unavailable or unreliable (notably Linux) | Medium | Medium | Secret Service with a clear failure path; never fall back to a file; documented remedy | Desktop | 10 |
| R06 | Index performance and battery cost on very large repos | High | Medium | Incremental indexing, throttling, battery awareness, exclusions, documented degradation beyond 200k files | Desktop | 15 |
| R07 | Sync conflicts lose messages | Medium | High | Append-only message model, conflict surfacing rather than silent resolution, interleaving property tests | Backend | 23 |
| R08 | Streaming backpressure causes UI stalls | Medium | Medium | Frame batching with documented drop priorities; synthetic fast-stream load test in CI | Desktop | 4, 8 |
| R09 | Local database corruption | Low | High | WAL, integrity check on open, pre-migration snapshots, quarantine-and-recover flow, fault-injection tests | Desktop | 9 |
| R10 | Checkpoint restore overwrites the user's later edits | Medium | High | Preview before restore, conflict detection, explicit user choice; never a silent overwrite | Desktop | 18 |
| R11 | Malicious or unstable MCP server | Medium | Critical | Approval registry with hash pinning, process sandboxing, network policy, untrusted results, hostile-server suite | Security | 19 |
| R12 | Multi-agent cost amplification | Medium | High | Hard aggregate budgets, off by default at GA, live cost display, abort on cap | Platform | 20 |
| R13 | Accidental coupling of the Agent module to legacy modules | Medium | High | CI import-isolation check, path ownership in CODEOWNERS, the deletion test | Backend | 10, 25 |
| R14 | Migration off `code-old` loses user data | Medium | Medium | No automatic import of legacy secrets; guided re-login; tested migration guide; export path first | Desktop | 9, 29 |
| R15 | Update pipeline ships a broken build | Medium | High | Health check with auto safe mode, revert with database snapshot, staged rollout, tested rollback | Ops | 27 |
| R16 | Signing or notarization credentials expire or fail | Medium | High | Expiry monitoring with advance alerts, pipeline exercised on every canary, documented manual fallback | Ops | 27 |
| R17 | Signing key compromise | Low | Critical | HSM custody, split offline backup, planned rotation, RB-11 response procedure | Security | 27 |
| R18 | Quota race allows limit overshoot | Medium | Medium | Atomic Redis operations via Lua, token reservations, concurrency race tests | Backend | 25 |
| R19 | Cost accounting drift versus provider invoices | Medium | Medium | Integer micro-USD, versioned price table, monthly reconciliation with a 2% drift alert | Backend | 11, 25 |
| R20 | Audit chain gap goes unnoticed | Low | High | Per-device hash chain, server-side gap detection at critical severity, admin verification tooling | Security | 7, 25 |

## Product and process risks

| ID | Risk | L | I | Mitigation | Owner | Phase |
|---|---|---|---|---|---|---|
| R21 | Approval fatigue leads users to blanket-allow | High | High | Shape batching, per-run scopes, read tools ungated, approvals-per-run tracked as a product metric | Product | 7 |
| R22 | Deny-by-default makes the product feel unusable | Medium | High | Mode profiles tuned with real usage; scopes broadened, never defaults weakened | Product | 7, 17 |
| R23 | Agent quality below user expectations | Medium | High | Eval suite with gates as the ship criterion rather than impressions; retrieval quality measured against a labeled set | QA | 28 |
| R24 | Eval harness flakiness makes gates untrustworthy | High | Medium | Deterministic stub mode, low temperature, multi-sample for borderline tasks, trend-based judgment | QA | 9, 28 |
| R25 | Rust capacity on the team | Medium | High | Focused crate ownership, pairing, review discipline, TypeScript-heavy surfaces kept in the UI layer | Engineering | 0+ |
| R26 | Scope creep across 30 phases | High | High | Feature catalog with phase mapping; new work enters the catalog or waits; exit criteria are objective | Product | All |
| R27 | Enterprise SSO and SCIM integration complexity | Medium | High | Test tenants for four IdPs, strict standards compliance, phased rollout with design partners | Backend | 24 |
| R28 | Plan mode produces plans users do not want | Medium | Medium | Human editing before execution, re-plan on failure, plan-quality eval category | Product | 17 |
| R29 | Documentation drifts from implementation | High | Medium | Spec updates required in the same PR; docs site generated from these specs so drift breaks the build | Engineering | All |
| R30 | Vendor performance claims do not reproduce | Medium | Low | Independent measurement in `bench/REPORT.md`; no external claim without our own number | QA | 28 |
| R31 | Launch-day load exceeds capacity | Medium | High | Load tested at 2× projected peak, staged rollout, pre-provisioned headroom | Ops | 29 |
| R32 | Support volume overwhelms the team at GA | Medium | Medium | Runbooks, diagnostic bundle command, beta-derived FAQ, staffed rotation with a game-day rehearsal | Ops | 29 |

## Highest-attention risks

R02, R03, R11, R17, and R20 are the Critical-impact items. Each has a dedicated automated suite that is a release gate, and none may be waived for a release ([security testing](../08-quality/04-security-testing.md)).

R21 and R26 are the highest-likelihood product risks. Both are tracked with metrics rather than opinion: approvals per run, and features shipped outside their assigned phase.

## Review cadence

| When | What |
|---|---|
| Every phase retrospective | Re-score risks touching that phase; add new ones discovered |
| Monthly during active development | Full register review with owners |
| Before each release | Confirm no Critical-impact risk lacks a passing gate |
| After every incident | Add or re-score the relevant risk and link the post-incident report |

A risk whose mitigation has no test or owner is treated as unmitigated.

## Related documents

- [Open questions](open-questions.md)
- [Threat model](../03-security/01-threat-model.md)
- [Security testing](../08-quality/04-security-testing.md)
- [Incident response](../03-security/09-incident-response.md)
