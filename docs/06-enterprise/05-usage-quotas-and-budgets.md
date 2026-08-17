# Usage, Quotas, and Budgets

Spec-Version: 1.2.0

Two independent limit systems: **entitlement quotas** (what the plan includes) and **budgets** (what the org chooses to spend). Both are enforced at the managed gateway; BYOK traffic is subject to policy cost caps only, since the user pays the provider directly.

## Limits

| Limit | Unit | Source | Enforced at |
|---|---|---|---|
| `dailyManagedRuns` | managed agent runs/UTC day | Agent entitlement limits | `POST /agent/v1/chat/stream` (pre-flight) |
| `monthlyTokens` | prompt+completion tokens | `Plan.limits` | gateway, per request and post-usage |
| `concurrentRuns` | simultaneous runs/user | `Plan.limits` | gateway + desktop scheduler |
| `maxCostPerRunUsdMicros` | integer micro-USD | org policy | desktop cost-guard hook + gateway |
| `maxCostPerDayUsdMicros` | integer micro-USD | org policy | gateway |
| `maxTokensPerRun` | tokens | org policy | desktop, before each model call |
| `orgMonthlyBudgetUsd` | USD | org budget setting | gateway |
| `seatMonthlyBudgetUsd` | USD | org budget setting | gateway |
| MCP/tool call rate | calls/min | policy | desktop |

**Known gap being fixed:** legacy `dailyMessages` exists in plan seed data but
is never enforced. The Agent module translates it to `dailyManagedRuns` and
implements enforcement in its own `AgentUsageGuard` (Phase 25) rather than
patching legacy billing code.

## Enforcement pipeline

```
POST /agent/v1/chat/stream
  1. AgentAuthGuard        → principal
  2. AgentUsageGuard
       a. seat + entitlement check         → EURY_ENTITLEMENT_NO_SEAT
       b. concurrent runs (Redis)          → EURY_QUOTA_CONCURRENCY
       c. daily messages (Redis, atomic)   → EURY_QUOTA_EXCEEDED
       d. monthly tokens projection        → EURY_QUOTA_EXCEEDED
       e. org / seat budget                → EURY_BUDGET_EXCEEDED
  3. forward upstream
  4. on `usage` event: increment counters, flush rollup
```

Checks are cheap and ordered by cost: the seat check is a cached read, counters are Redis `INCR`, and the budget check uses a cached org total refreshed every 60 s. Total added latency budget: ≤ 10 ms p95.

Reservations prevent overshoot on long runs: the guard reserves `maxOutputTokens` at request start and reconciles against actual usage on `done` or on disconnect.

## Redis keys

| Key | Value | TTL |
|---|---|---|
| `agent:quota:{userId}:{YYYYMMDD}:runs` | counter | 48 h |
| `agent:quota:{userId}:{YYYYMM}:tokens` | counter | 40 d |
| `agent:quota:{userId}:concurrent` | set of runIds | 1 h per member |
| `agent:budget:{orgId}:{YYYYMM}:micros` | counter | 40 d |
| `agent:reserve:{runId}` | reserved tokens | 10 min |

Redis is the hot path; `AgentUsageCounter` in Postgres is the durable rollup flushed every 60 s and at shutdown. If Redis is unavailable, the guard falls back to the Postgres rollup and applies a conservative multiplier — it never fails open on a hard quota.

## Soft and hard thresholds

| Threshold | Behavior |
|---|---|
| 80% of any limit | Warning toast + status-bar chip; one notification per period |
| 95% | Persistent banner; admin email for org-level limits |
| 100% quota | New runs blocked (`429`); in-flight runs finish |
| 100% budget | New managed calls blocked; BYOK fallback offered if policy allows |
| Overage enabled (Enterprise) | Continue up to a contracted overage ceiling, metered and flagged |

In-flight runs are never killed by a quota boundary — killing a run mid-write is worse than a small overage. The overshoot is bounded by the reservation.

## Desktop surfaces

| Surface | Content |
|---|---|
| Status bar | Context usage, session cost, throttle indicator |
| Settings → Usage | Runs today vs. limit, tokens this month, cost this month, budget remaining, reset date, per-model breakdown |
| Run status bar | Per-run tokens and cost, live |
| Blocked run dialog | Which limit, current value, reset time, and the exact action available (wait, switch to BYOK, ask admin) |

Data comes from `GET /agent/v1/usage/current` (cached 60 s) plus locally computed BYOK cost. BYOK cost is estimated locally from token counts and a bundled price table; it is labeled as an estimate and never billed.

## Admin surfaces

`/admin/agent/usage`: org totals, per-seat table, per-model split, daily trend, top spenders, denied-request counts by reason, CSV export. Budgets are set here (`orgMonthlyBudgetUsd`, `seatMonthlyBudgetUsd`, alert thresholds, overage ceiling) and every change is audited.

## Cost accounting

- Money is stored as integer micro-USD (`costUsdMicros`); no floating point anywhere in the ledger.
- Prices come from a versioned price table keyed by `provider/model/effectiveFrom`, so historical runs keep their original cost.
- Cached-token and reasoning-token pricing are modeled separately where providers charge differently.
- Reconciliation job compares gateway-recorded usage against provider invoices monthly; drift over 2% raises an alert.

## Errors

| Code | HTTP | Meaning |
|---|---|---|
| `EURY_QUOTA_EXCEEDED` | 429 | Plan quota reached; `Retry-After` set to period reset |
| `EURY_QUOTA_CONCURRENCY` | 429 | Too many simultaneous runs |
| `EURY_BUDGET_EXCEEDED` | 402 | Org or seat budget exhausted |
| `EURY_ENTITLEMENT_NO_SEAT` | 403 | No managed-gateway seat assigned |
| `EURY_COST_CAP_EXCEEDED` | — | Local cost guard aborted the run before the call |

Every response includes `details` with the limit name, current value, limit value, and reset timestamp so the UI can render an actionable message without guessing.

## Testing

| Test | Assertion |
|---|---|
| Atomicity | Concurrent requests cannot exceed a daily limit (Redis Lua, race test) |
| Reservation | Aborted and disconnected runs release reservations |
| Redis down | Degraded path still blocks at hard limits |
| Boundary | In-flight run completes when the limit is crossed mid-stream |
| Rollup | Postgres totals match Redis within one flush interval |
| Money | No float arithmetic; price-table version pinned per event |

## Delivery

The local cost guard, per-run caps, and cost accounting ship in Phase 11. Gateway quota enforcement, budgets, dashboards, and invoice reconciliation ship in Phase 25.

## Related documents

- [Pricing and packaging](../01-product/04-pricing-and-packaging.md)
- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Workspace policies](03-workspace-policies.md)
- [Admin console](06-admin-console-spec.md)
