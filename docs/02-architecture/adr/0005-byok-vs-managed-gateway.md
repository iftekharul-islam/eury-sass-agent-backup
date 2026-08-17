# ADR-0005: BYOK vs Managed Gateway

Spec-Version: 1.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Users want lowest latency (direct to OpenAI/Anthropic). Enterprises want centralized billing, key custody, and audit without distributing provider keys.

## Decision

Support **both** paths:

1. **BYOK:** API keys in OS keychain; desktop calls provider directly.
2. **Managed:** Desktop calls `POST /agent/v1/chat/stream`; Nest validates plan, applies quota, forwards to Eury LLM service.

User/org policy may restrict to managed only.

## Consequences

**Positive:**
- Solo dev latency optimal with BYOK.
- Enterprise control with managed path.
- Revenue alignment (managed usage metered).

**Negative:**
- Two code paths to test.
- Gateway adds ~10–30 ms (acceptable per latency budget).

**Mitigations:**
- Shared request/response types in `agent-types`.
- Integration tests for both paths.
