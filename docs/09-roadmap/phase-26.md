# Phase 26 — Observability and Reliability

Spec-Version: 1.1.0

**Track:** F — Enterprise and GA · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Instrument everything: traces, metrics, structured logs, opt-in desktop telemetry, crash reporting, dashboards, alerts, and the runbook-per-alert rule.

## Why this phase exists here

Before GA we must be able to see and diagnose production. An SLO without instrumentation is a wish, and an alert without a runbook is a page nobody can action.

## In scope

- OpenTelemetry tracing across every `/agent/v1/*` route with the defined span tree
- Metric set with cardinality guards and no per-user labels
- Structured cloud logging with enforced redaction
- Opt-in desktop telemetry with a transparent event viewer
- Crash reporting with symbolization and path scrubbing
- Dashboards for gateway, auth, quota, policy, audit, and release health
- Alerts with severities, routing, and a runbook link per alert
- Error-budget tracking and burn alerts
- `--export-logs` diagnostic bundle for support
- Correlated request ids surfaced in desktop error dialogs

## Feature IDs

None — this phase establishes prerequisites or governance contracts.

## Out of scope

- Public status page automation (Phase 29)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D26.1 | Tracing with the documented span tree and attributes | [observability](../07-ops/05-observability-and-slos.md) |
| D26.2 | Metrics with cardinality guards | [observability](../07-ops/05-observability-and-slos.md) |
| D26.3 | Redaction-enforced structured logging with a unit test on secret shapes | [telemetry spec](../04-specs/14-telemetry-spec.md) |
| D26.4 | Opt-in desktop telemetry with a live sample viewer in Settings | [telemetry spec](../04-specs/14-telemetry-spec.md) |
| D26.5 | Crash reporting with symbolization and scrubbing | [observability](../07-ops/05-observability-and-slos.md) |
| D26.6 | Seven dashboards covering every SLI | [observability](../07-ops/05-observability-and-slos.md) |
| D26.7 | Alert rules with runbook links and error-budget burn alerts | [runbooks](../07-ops/06-runbooks.md) |
| D26.8 | `--export-logs` redacted diagnostic bundle | [environments and config](../07-ops/01-environments-and-config.md) |
| D26.9 | Request-id correlation from desktop dialog to cloud trace | [error taxonomy](../04-specs/15-error-taxonomy.md) |

## Key decisions and design notes

- Telemetry is opt-in and transparent: the user can see the exact payload we would send.
- No metric is labeled by user or device id; per-user analysis belongs in the audit tables.
- Every alert links to a runbook entry; an alert without one is not allowed to page.
- Redaction is a shared, tested utility rather than a per-call-site habit.

## Contracts touched

- Telemetry event schema and allowed event list
- Trace attribute names
- Metric names and labels

## Dependencies

- Phase 25 (surfaces to observe)
- Phase 11 (gateway)
- Phase 10 (auth)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Telemetry leaking content | Privacy incident | Allowlisted event fields, redaction tests, and a fuzz corpus asserting no secret survives |
| Metric cardinality explosion | Cost and slow queries | Cardinality guard in review and a CI check on label sets |
| Alert fatigue | Real incidents missed | Severity discipline, tuned thresholds, and a quarterly alert review |
| Trace overhead | Latency | Sampling with always-on error capture; overhead measured in the benchmark |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Redaction utility against a secret corpus; event schema validation |
| Integration | End-to-end trace continuity from desktop request id to cloud spans |
| Security | No content or secret in logs, metrics, telemetry, or crash reports |
| Ops | Every alert fires in a staged failure and its runbook resolves it |

## Metrics and targets

| Metric | Target |
|---|---|
| Tracing overhead | < 5 ms p95 per request |
| Alerts without a runbook link | 0 |
| SLI dashboard coverage | 100% of defined SLOs |
| Secrets found in telemetry corpus test | 0 |

## Exit criteria

- [ ] Every SLI is instrumented and dashboarded
- [ ] Every alert has a tested runbook
- [ ] Desktop telemetry is opt-in with a visible payload sample
- [ ] Redaction tests pass across logs, metrics, telemetry, and crash reports
- [ ] Request-id correlation works end to end
- [ ] Error-budget burn alerts configured

## Deferred from this phase

- Status page automation (Phase 29)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
