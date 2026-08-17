# Provider and Model Governance

Spec-Version: 1.0.0

This document governs which model may process which Eury Agent data, how a provider changes safely, and how routing remains explainable. It applies to BYOK and managed gateway routes; BYOK changes credential ownership, not policy enforcement.

## Control objectives

1. Every provider/model route has an owner, data classification, residency status, capability declaration, evaluation record, and rollback path.
2. A model change is visible to the user and audit trail; the gateway MUST NOT silently substitute a different model.
3. Organization policy is enforced both before local request assembly and at the managed gateway.
4. New or materially changed model behavior is evaluated before a production rollout.

## Data-class routing

| Data class | Examples | Default routing |
|---|---|---|
| Public | Open-source repository, public documentation | Approved BYOK or managed provider |
| Internal | Private source code, build logs | Approved provider; organization allowlist required |
| Restricted | Credentials, customer data, regulated source | Local-only where possible; managed route only when policy permits a residency-approved provider |
| Prohibited | Private keys, raw secrets, payment instruments | Never placed in model context; redaction and tool guards block it |

Classification is set by organization policy and may only become more restrictive at team, user, workspace, or local level.

## Model catalog contract

Each catalog entry MUST declare provider, model ID, supported capabilities, maximum attachment limits, approved residency, pricing version, lifecycle state, and `disabledReason` when unavailable. Capabilities include tools, vision, image generation, structured output, and reasoning exposure. The desktop renders unavailable capabilities explicitly; it never guesses from a model name.

## Change and rollout process

| Change | Required control |
|---|---|
| New model/provider | Security and privacy review, data-processing review, contract fixture, baseline eval, rollback target |
| Model version upgrade | Regression eval against current production baseline, price/capability diff, staged rollout |
| Capability enablement | Tool/attachment policy review, abuse and cost tests, UI disabled-state coverage |
| Provider incident | Freeze new routing, expose affected state, use only policy-authorized fallback |
| Provider retirement | Announce deprecation window, update catalog, migrate policies, retain audit mapping |

Managed routing uses a canary rollout by organization allowlist, then percentage rollout. BYOK catalog visibility may ship earlier, but it remains disabled until the provider adapter passes its compatibility suite.

## Fallback and disclosure

Fallback is opt-in by policy. Before a route changes, the user sees the target provider/model, capability difference, estimated cost difference, and reason. If the user declines, the run pauses or fails with a typed error. The audit event records requested model, resolved model, route, policy version, and reason; it never records prompt content.

## Evaluation gates

Every supported model/provider pair has pinned fixtures for streaming, tool calling, cancellation, malformed output, vision attachment handling where supported, and cost attribution. Before rollout it must pass:

- tool discipline and prompt-injection evals at the defined security threshold;
- task-quality baseline with no material regression against the current approved version;
- latency, availability, and cost guard checks;
- data residency and redaction verification for managed routes.

A failed gate blocks promotion and preserves the prior catalog version.

## Related documents

- [BYOK and managed gateway ADR](adr/0005-byok-vs-managed-gateway.md)
- [Cloud architecture](03-cloud-architecture.md)
- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Multimodal specification](../04-specs/17-multimodal-and-attachment-spec.md)
- [Workspace policies](../06-enterprise/03-workspace-policies.md)
