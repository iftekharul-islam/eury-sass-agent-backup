# Observability and SLOs

Spec-Version: 1.2.0

## Service level objectives (cloud)

| SLI | Definition | SLO | Window |
|---|---|---|---|
| Auth availability | Non-5xx on `/agent/v1/auth/*` | 99.95% | 30 d rolling |
| Gateway availability | Non-5xx on `/agent/v1/chat/stream` | 99.9% | 30 d rolling |
| Gateway added latency | p95 of (our processing time, excluding upstream) | < 400 ms | 30 d |
| Time to first byte | p95 from request to first NDJSON line | < 1.5 s | 30 d |
| Policy fetch availability | Non-5xx on `/agent/v1/policies/effective` | 99.95% | 30 d |
| Audit ingest success | Accepted batches / submitted batches | 99.9% | 30 d |
| Release manifest availability | Non-5xx on `/agent/v1/releases/latest` | 99.99% | 30 d |
| Device login success | Approved polls / started flows (excluding user cancels) | 99% | 7 d |

Error budgets: gateway 0.1% ≈ 43 min/month. Burning 50% of a budget freezes non-essential releases for that surface; 100% triggers a reliability-only sprint.

## Capacity and resilience ownership

Platform owns the gateway capacity model, including active streams, provider connection pools, audit backlog, and release-manifest capacity. Capacity forecasts are reviewed monthly and before a material customer onboarding or model rollout. A forecast breach opens a tracked capacity action before utilization exceeds 70% of the tested limit.

| Situation | Required action |
|---|---|
| 50% error-budget burn | Freeze non-essential changes to the affected surface; platform lead reviews recovery plan |
| 100% error-budget burn | Reliability-only work until the SLO is restored |
| Provider outage | Disable affected route in the catalog; offer only policy-authorized fallback; publish status and preserve local/offline behavior |
| Audit backlog threshold | Page Security and Platform; preserve local queue; do not discard audit records to recover capacity |
| Restore/DR failure | Block release promotion until a successful repeat exercise is recorded |

At least quarterly, Platform and Security run a documented game day covering one provider outage, one policy-distribution failure, one encrypted-local-store recovery, and one audit-ingestion backlog. Findings receive owners and target dates in the risk register.

## Desktop objectives

Client-side, tracked from opt-in telemetry as p95 per version:

| SLI | Target |
|---|---|
| Cold start to interactive | < 2 s |
| Time to first token (BYOK) | < 800 ms |
| IPC round-trip | < 1 ms |
| Crash-free sessions | > 99.5% |
| Update success rate | > 99% |
| Frame rate while streaming | ≥ 55 fps |

Definitions and measurement methodology: [latency budget](../02-architecture/05-latency-budget.md) and [performance benchmarks](../08-quality/03-performance-benchmarks.md).

## Cloud instrumentation

### Traces

OpenTelemetry on every `/agent/v1/*` route. Span tree for a gateway request:

```
POST /agent/v1/chat/stream
├── agent.auth.verify
├── agent.usage.check           (redis)
├── agent.policy.resolve        (cache|db)
├── agent.model.validate
├── agent.upstream.request      ← the only long span
│   └── agent.upstream.first_byte (event)
└── agent.usage.record
```

Attributes: `agent.run_id`, `agent.device_id`, `agent.org_id`, `agent.provider`, `agent.model`, `agent.policy_version`, `agent.client_version`. Never prompt text, never tool arguments.

Sampling: 100% of errors and slow requests (> 2 s added latency), 5% of the rest, always-on for a request that carries a debug header from an internal account.

### Metrics

| Metric | Type | Labels |
|---|---|---|
| `agent_http_requests_total` | counter | route, method, status |
| `agent_http_duration_seconds` | histogram | route |
| `agent_chat_stream_total` | counter | provider, model, outcome |
| `agent_chat_stream_added_latency_seconds` | histogram | provider |
| `agent_chat_stream_ttfb_seconds` | histogram | provider, model |
| `agent_chat_stream_active` | gauge | provider |
| `agent_tokens_total` | counter | provider, model, kind (prompt/completion) |
| `agent_cost_micros_total` | counter | org, provider, model |
| `agent_quota_denied_total` | counter | reason |
| `agent_policy_denied_total` | counter | rule |
| `agent_auth_flow_total` | counter | step, outcome |
| `agent_refresh_reuse_total` | counter | — |
| `agent_audit_events_ingested_total` | counter | outcome |
| `agent_audit_gap_detected_total` | counter | — |
| `agent_release_manifest_requests_total` | counter | platform, channel |

Cardinality guard: no metric is labeled by `userId` or `deviceId`; per-user analysis happens in the audit/usage tables, not in metrics.

### Logs

Structured JSON to `backend/logs/agent/`, with `requestId`, `runId`, `userId`, `orgId`, route, status, and duration. Prohibited fields: prompt text, completion text, tool arguments, file contents, tokens, secrets. Redaction is a shared utility (`agent-redaction.util.ts`) with a unit test asserting known secret shapes are stripped.

## Desktop instrumentation

| Concern | Rule |
|---|---|
| Telemetry | Opt-in; off by default; a single toggle covers metrics and crash reports |
| Content | Event names, durations, error codes, versions, coarse platform info only |
| Crash reports | Minidump + stack, symbolized server-side; scrubbed of paths and env |
| Local logs | `logs/agent.log`, rotated 10 MB × 7, `debug` capped to 24 h then reverting to `info` |
| Diagnostics | `--export-logs` produces a redacted zip (logs, versions, policy summary, sanitized settings) for support |
| Transparency | Settings → Privacy shows exactly what a telemetry event contains, with a live sample |

Event schema and the full allowed-event list: [telemetry spec](../04-specs/14-telemetry-spec.md).

## Dashboards

| Dashboard | Panels |
|---|---|
| Gateway health | RPS, error rate by status, added latency p50/p95/p99, TTFB, active streams, upstream error breakdown |
| Auth health | Device flow funnel, refresh rotation rate, reuse detections, SSO assertion outcomes |
| Quotas and cost | Denials by reason, spend by org, top models, budget burn-down |
| Policy | Denials by rule, stale-policy blocks, exception requests |
| Audit pipeline | Ingest rate, rejection reasons, queue backlog estimates, gap detections |
| Release health | Adoption by version, update failure rate, crash-free sessions, revert rate |
| SLO overview | Each SLI against its objective with error-budget burn |

## Alerting

| Alert | Condition | Severity |
|---|---|---|
| Gateway error rate | > 2% for 5 min | SEV2 |
| Gateway down | > 20% 5xx for 2 min | SEV1 |
| Auth error rate | > 1% for 5 min | SEV1 |
| Added latency | p95 > 1 s for 10 min | SEV3 |
| Refresh reuse spike | > 10/hour | SEV2 (security) |
| Audit gap detected | any | SEV2 (security) |
| Audit ingest failures | > 5% for 15 min | SEV3 |
| Release manifest 5xx | any sustained 2 min | SEV2 |
| Crash rate | crash-free < 99% on the active version | SEV2 |
| Update failure rate | > 5% on the active version | SEV2 |
| Error budget burn | 50% consumed mid-window | SEV3 |

Routing, severities, and on-call expectations: [incident response](../03-security/09-incident-response.md). Every alert links to the matching entry in [runbooks](06-runbooks.md); an alert without a runbook link is not allowed to page.

## Correlation

`X-Agent-Request-Id` is generated by the desktop, echoed by the cloud, attached to traces, logs, and audit events, and shown in the desktop's error dialogs. A user can paste one id into support and the whole path is reconstructible without asking for their code.

## Data retention

| Data | Retention |
|---|---|
| Traces | 7 days (30 for sampled errors) |
| Metrics | 15 months downsampled |
| Cloud logs | 30 days hot, 90 days cold |
| Crash reports | 90 days |
| Desktop local logs | 7 rotations |

## Related documents

- [Telemetry spec](../04-specs/14-telemetry-spec.md)
- [Runbooks](06-runbooks.md)
- [Latency budget](../02-architecture/05-latency-budget.md)
- [Incident response](../03-security/09-incident-response.md)
- [Compatibility and lifecycle](09-compatibility-and-lifecycle.md)
