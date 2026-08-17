# Phase 11 — Model Routing and Cost

Spec-Version: 1.4.0

**Track:** C — Product surfaces · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Support Eury-managed gateway inference with a policy-aware model catalog, network tools, and real cost accounting with caps. BYOK is deferred to a follow-up phase — all desktop traffic for this phase routes through the managed gateway.

## Why this phase exists here

Model choice, cost visibility, and the managed path are what make the product viable commercially and predictable for users. It lands after identity because the gateway requires authentication and entitlements.

## In scope

- Model catalog served by the Agent module with policy-aware `allowed` flags
- Managed gateway `POST /agent/v1/chat/stream` with NDJSON passthrough and abort propagation
- Network tools: `web_search` and `web_fetch` through the Agent module proxy
- Vision attachments (base64 inline) and consent-gated image generation for capable models
- Cost estimation with a versioned price table; per-run and per-day caps, enforced server-side
- Model picker UI with context window, cost hint, and disabled reasons
- Usage surface in Settings backed by `GET /agent/v1/usage/current`

## Feature IDs

`F-022`, `F-031`, `F-032`, `F-033`, `F-045`, `F-048`, `F-049`

## Out of scope

- Quota enforcement at the org level (Phase 25)
- Budgets and admin dashboards (Phase 25)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D11.1 | Provider abstraction routing four catalog providers (OpenAI, Anthropic, Google, xAI) through the managed gateway | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D11.2 | `GET /agent/v1/models` with plan and policy filtering | [cloud API](../04-specs/06-cloud-api-contract.md) |
| D11.3 | Managed gateway with tee'd NDJSON, 300 s timeout, and abort within 250 ms | [cloud architecture](../02-architecture/03-cloud-architecture.md) |
| D11.4 | Server-side usage metering and cost-cap enforcement: `AgentUsageCounter` incremented per completed stream, cost/token caps on `AgentChatStreamDto` | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |
| D11.5 | Network tools with SSRF protections and untrusted-content marking | [injection defense](../03-security/05-prompt-injection-defense.md) |
| D11.6 | Client-side cost guard enforcing `maxCostPerRunUsdMicros` and `maxTokensPerRun` | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |
| D11.7 | Model picker and run cost display | [chat UX](../05-ui/03-chat-and-streaming-ux.md) |
| D11.8 | Versioned price table with integer micro-USD accounting, single-sourced between desktop and backend | [usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md) |
| D11.9 | Capability-gated image input and generation, encrypted attachment storage, and explicit save flow | [multimodal spec](../04-specs/17-multimodal-and-attachment-spec.md) |

## Key decisions and design notes

- The managed gateway is the only inference path delivered in this phase; BYOK (ADR-0005) is deferred to a follow-up phase and re-evaluated once server-side usage metering is live.
- Money is integer micro-USD end to end; no floating-point currency anywhere.
- Model failover never silently changes the model — the user and the audit log always see which model ran.
- Web results are untrusted content and are marked as such before entering a prompt.
- Vision observations and generated images are untrusted content; generated images never write into a project until the user approves a normal file save.
- Provider/model rollout is governed by data classification, evaluation gates, staged rollout, and explicit fallback disclosure.
- The gateway is a thin proxy: it validates, meters, and forwards; it never runs the agent loop — so caps and usage must be enforced server-side, not only in the desktop client.

## Contracts touched

- `/agent/v1/models` and `/agent/v1/chat/stream` shapes
- `/agent/v1/tools/web-search` shape
- Image provider capability, generation, and attachment-reference shapes
- Provider/model governance, routing, and rollout evidence
- Provider client interface
- Price table format and versioning

## Dependencies

- Phase 4 (engine and provider interface)
- Phase 10 (auth and entitlements)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Provider API drift | Broken streaming | Recorded cassettes per provider in CI; adapter per provider; contract tests |
| Gateway added latency | Feels slower than a direct provider call | Guard chain budget of 10 ms; streaming passthrough without buffering; measured in CI |
| Cost estimates wrong | User distrust or overspend | Versioned price table, monthly reconciliation, estimates labeled as estimates |
| SSRF via fetch tools | Internal network exposure | Address allowlisting, blocked link-local and metadata ranges, DNS rebinding protection |
| Caps enforced only client-side | Non-official clients bypass spend limits entirely | Server-side `AgentUsageCounter` increment and cap check on every gateway stream, not just the desktop `CostGuard` |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Provider adapters against recorded streams; price math |
| Contract | Gateway request/response and error codes |
| Integration | Gateway streaming path produces a spec-conformant event stream end to end |
| Security | SSRF corpus; gateway auth token never leaves the OS keychain; no token in logs |
| Security | Image metadata stripping; generated output cannot create a workspace file without a separate write approval |
| Evaluation | Provider capability fixtures, fallback disclosure, and model-change regression gates |
| Performance | Gateway added latency and guard chain benchmarks |

## Metrics and targets

| Metric | Target |
|---|---|
| Gateway added latency | < 400 ms p95 |
| Guard chain (auth + quota + policy) | < 10 ms p95 |
| Time to first token via gateway | < 1.2 s p95 |
| Cost estimate error vs. provider invoice | < 2% |

## Exit criteria

- [ ] Managed gateway streams inference for catalog models via `EuryGatewayProvider`
- [ ] Model catalog reflects plan and policy restrictions with visible reasons, enforced server-side (not a hardcoded allow)
- [ ] Abort propagates from desktop through the gateway to the provider
- [ ] Cost and usage caps are enforced server-side (`AgentUsageCounter` incremented, caps checked before forwarding), not only by the desktop `CostGuard`
- [ ] Network tools pass the SSRF corpus
- [ ] Vision attachments and image generation honor model capability, policy, cost consent, and explicit-save rules
- [ ] Latency targets met

## Deferred from this phase

- Org quotas, budgets, and dashboards (Phase 25)
- BYOK key management, per-provider validation, and BYOK/gateway fallback routing (follow-up phase; ADR-0005 remains the target end state)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
