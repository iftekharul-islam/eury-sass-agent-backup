# Test Strategy

Spec-Version: 1.1.0

## Principles

1. **Security code gets the most tests.** `agent-sandbox` and `agent-policy` are the highest-coverage crates because a bug there is a breach, not a defect.
2. **Contracts are tested as golden files.** The event protocol, IPC commands, and `/agent/v1/*` responses are pinned so a change is visible in review.
3. **No mocked sandbox in sandbox tests.** Path guards and command guards are tested against a real temp filesystem and real processes.
4. **Determinism by default.** A stub model provider makes agent behavior reproducible; live models are used only in nightly evals.
5. **No flake retries.** A flaky test is quarantined with an issue and a deadline, never auto-retried into green.

## Layers

| Layer | Scope | Tools | Runs |
|---|---|---|---|
| Unit (Rust) | Pure logic in each crate | `cargo nextest`, `proptest` | Every PR |
| Unit (Web) | Components, hooks, reducers | `vitest`, Testing Library, `jest-axe` | Every PR |
| Integration (Rust) | Store + migrations, sandbox on a real FS, policy merge, engine with a stub provider | `cargo nextest` with fixtures | Every PR |
| IPC contract | Every command's request/response against golden JSON | `cargo test` + shared fixtures | Every PR |
| Event contract | Every event variant serialized against golden JSON, plus TS type parity | Rust + `vitest` | Every PR |
| Cloud unit | Agent services with a mocked Prisma | `jest` | Every PR touching the module |
| Cloud contract | `/agent/v1/*` against a real Postgres + Redis | `jest` + `supertest` + testcontainers | Every PR touching the module |
| Isolation | Agent module imports nothing from other feature modules | lint rule in CI | Every PR |
| E2E | Full user flows in the packaged app | Playwright + `tauri-driver` | Nightly, pre-release, `e2e` label |
| Agent eval | Task success on fixture repos | Custom harness | Nightly, pre-release |
| Performance | Latency and throughput benchmarks | `criterion`, custom harness | Weekly, pre-release |
| Security | SAST, deps, fuzz, DAST | `cargo audit`, Semgrep, gitleaks, `cargo-fuzz`, ZAP | PR + weekly |

## Coverage targets

| Area | Line coverage | Rationale |
|---|---|---|
| `agent-sandbox` | 90% + fuzzing | Containment boundary |
| `agent-policy` | 90% | Authorization decisions |
| Cloud Agent auth | 90% | Token issuance and rotation |
| `agent-store` | 85% | Data integrity and migrations |
| `agent-core` | 80% | Run lifecycle |
| `agent-tools` | 80% | Tool argument validation |
| `agent-index`, `agent-memory` | 75% | Quality measured by evals too |
| UI components | 70% | Behavior over pixels |
| Cloud Agent module (overall) | 85% | Small surface, high stakes |

Coverage is a floor, not a goal. A PR that raises coverage while removing a branch assertion is rejected.

## Critical regression suites

These run on every PR and may never be skipped:

| Suite | Contents |
|---|---|
| Path traversal corpus | `../`, symlinks, junctions, UNC paths, `~` expansion, null bytes, unicode normalization, long paths, case-insensitive collisions, `/proc/self` |
| Command guard corpus | Shell metacharacters, `$( )`, backticks, `;`/`&&` chaining, env-var indirection, quoted `sudo`, `rm -rf` variants, PATH hijack |
| Policy merge properties | Property: merging can never widen; fuzzed over random policy pairs |
| Deny-by-default | Every write/execute/network tool denied without a grant, for each mode |
| Prompt injection corpus | Instructions embedded in file contents, tool output, web results, MCP results, and image alt text ([defense](../03-security/05-prompt-injection-defense.md)) |
| Secret redaction | Known secret shapes never appear in logs, telemetry, or audit payloads |
| Event/IPC golden files | Byte-level diff of serialized contracts |
| Auth negative tests | No token, expired, wrong audience, revoked device, reused refresh, missing config |
| Checkpoint round trip | Write → checkpoint → restore for text, binary, renames, deletes, CRLF, and UTF-16 files |

## E2E flows

Each runs on all three platforms nightly:

1. First launch → device login (against a staging cloud) → workspace open → trust prompt.
2. Read-only question with retrieval; verify cited files.
3. Agent edit with approval → diff review → partial hunk apply → save.
4. Shell command needing approval → deny → agent proposes an alternative.
5. Plan mode → generate plan → execute step by step → checkpoint restore.
6. Abort mid-run; verify no orphaned processes and consistent local state.
7. Offline: disconnect, verify degraded banner, BYOK still works, audit queues.
8. Update N−1 → N; verify data preserved.
9. Policy tightened server-side mid-session; verify standing grants revoked.
10. Keyboard-only walkthrough of flows 1–5.

## Test data and fixtures

| Fixture | Purpose |
|---|---|
| `tiny-node`, `mid-python`, `large-monorepo` (50k files) | Indexing, retrieval, and performance |
| `hostile-repo` | Injection payloads, symlink escapes, huge and binary files, weird encodings |
| `broken-tests-repo` | Eval tasks that require fixing a failing test |
| Stub provider | Scripted responses and tool calls, deterministic |
| Recorded provider cassettes | Replayed real provider streams for parser tests |

Fixtures live in `agent/tests/fixtures/`; the large monorepo is generated by a script rather than committed.

## Environments

| Test type | Backend |
|---|---|
| Unit, integration | None or in-process |
| Cloud contract | Ephemeral Postgres + Redis via testcontainers |
| E2E | Staging cloud with seeded orgs, plans, and policies |
| Eval | Staging cloud + pinned live models, spend-capped |

No test ever runs against production, and no test account has a production entitlement.

## PR requirements

- New logic has unit tests; new contracts have golden files.
- Bug fixes include a regression test that fails before the fix.
- Security-relevant changes (sandbox, policy, auth, IPC surface) need two reviewers and the security checklist.
- Coverage must not decrease for the touched crate.
- Skipped or `#[ignore]`d tests require a linked issue.

## What is deliberately not tested

Vendor internals (we test our adapter, not Cersei's tokenizer), exact model wording (asserted by structure and eval rubric instead), and pixel-perfect rendering (visual regression is limited to a small set of layout-critical fixtures).

## Related documents

- [Agent eval harness](02-agent-eval-harness.md)
- [Performance benchmarks](03-performance-benchmarks.md)
- [Security testing](04-security-testing.md)
- [Definition of done](05-definition-of-done.md)
