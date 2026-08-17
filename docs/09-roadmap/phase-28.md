# Phase 28 — Quality and Evaluation

Spec-Version: 1.1.0

**Track:** F — Enterprise and GA · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Bring quality infrastructure to GA standard: the full eval suite with gates, the complete E2E matrix, benchmark baselines, accessibility verification, and an external penetration test.

## Why this phase exists here

This is the phase where we find out whether the product is actually good, using our own measurements rather than impressions. Everything it gates on must already exist.

## In scope

- Eval suite expanded to 80+ tasks across all categories
- LLM-judge rubrics with fixed judge versions and multi-sample scoring
- Nightly eval in CI with gating thresholds and per-model qualification
- E2E suite: ten flows across three platforms, including a keyboard-only pass
- Benchmark baselines committed per platform with a regression gate
- Load, spike, and soak tests for cloud and desktop
- Accessibility verification: automated gates plus manual screen-reader passes
- External penetration test of desktop, cloud, and pipeline, with remediation
- Independent verification of vendor performance claims in `bench/REPORT.md`
- Flaky-test quarantine process and a test-health dashboard

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- New product features

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D28.1 | 80+ eval tasks with assertions and rubrics | [eval harness](../08-quality/02-agent-eval-harness.md) |
| D28.2 | Nightly eval with gates on pass rate, unsafe count, and cost | [eval harness](../08-quality/02-agent-eval-harness.md) |
| D28.3 | Ten E2E flows green on macOS, Windows, and Linux | [test strategy](../08-quality/01-test-strategy.md) |
| D28.4 | Committed benchmark baselines and a CI regression gate | [benchmarks](../08-quality/03-performance-benchmarks.md) |
| D28.5 | Load, spike, and 8-hour soak suites | [benchmarks](../08-quality/03-performance-benchmarks.md) |
| D28.6 | WCAG 2.2 AA verification report including manual passes | [accessibility](../05-ui/07-accessibility-and-i18n.md) |
| D28.7 | External pentest report with all critical and high findings closed | [security testing](../08-quality/04-security-testing.md) |
| D28.8 | `bench/REPORT.md` with independently measured vendor claims | [benchmarks](../08-quality/03-performance-benchmarks.md) |
| D28.9 | Test-health dashboard and quarantine process | [test strategy](../08-quality/01-test-strategy.md) |

## Key decisions and design notes

- An unsafe eval result blocks release regardless of pass rate. Safety and quality are scored separately and never traded.
- Vendor performance claims are never repeated externally without a matching entry in our own report.
- Flaky tests are quarantined with an owner and a deadline, never retried into green.
- Eval assertions are never weakened to make a release pass; the release waits instead.

## Contracts touched

- Eval task and result schemas
- Benchmark baseline format

## Dependencies

- Phase 20 (features to evaluate)
- Phase 27 (releasable artifacts to test)
- Phase 26 (instrumentation)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Eval cost | Budget overrun | Spend caps, cassette replay for parser-level checks, and tiered run frequency |
| Eval nondeterminism | Noisy gates | Low temperature, multi-sample for borderline tasks, and trend-based judgment |
| Pentest findings late | Delayed GA | Pentest scheduled at the start of the phase with buffer for remediation and retest |
| Benchmark noise in CI | False failures | Runner variance measured and used as the noise allowance; reference hardware for published numbers |

## Test plan

| Layer | Coverage |
|---|---|
| Meta | Harness correctness verified with the stub provider |
| E2E | Ten flows on three platforms, plus keyboard-only |
| Performance | Full benchmark suite against baselines |
| Security | Pentest plus the complete per-release security checklist |
| A11y | Automated gates plus VoiceOver, NVDA, and Orca passes |

## Metrics and targets

| Metric | Target |
|---|---|
| Eval pass rate (primary model) | ≥ 85% |
| Unsafe eval results | 0 |
| Injection and tool-discipline categories | 100% |
| E2E flake rate | < 1% |
| Critical/high pentest findings open | 0 |

## Exit criteria

- [ ] Eval suite ≥ 80 tasks with all gates met on every supported model
- [ ] E2E suite green on all three platforms including keyboard-only
- [ ] Benchmark baselines committed with the regression gate active
- [ ] WCAG 2.2 AA verified with manual screen-reader passes
- [ ] Pentest complete with critical and high findings closed and retested
- [ ] `bench/REPORT.md` published with our own measurements

## Deferred from this phase

- Bug bounty program (Phase 29 / post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
