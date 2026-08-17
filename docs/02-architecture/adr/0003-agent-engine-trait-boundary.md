# ADR-0003: AgentEngine Trait Boundary

Spec-Version: 1.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Vendor SDK lock-in risk if Cersei types leak into UI and persistence. Testing requires mockable agent interface.

## Decision

All product code depends on `agent_core::AgentEngine` and `agent_types::AgentEvent`. Only `cersei_adapter` crate imports `cersei::*`.

## Consequences

**Positive:**
- Testability with mock engine.
- Potential SDK swap without UI rewrite.
- Clear ownership of event schema versioning.

**Negative:**
- Mapping layer maintenance (Cersei 26 events → our protocol).
- Some Cersei features exposed only through adapter extensions.

**Mitigations:**
- Contract tests on event JSON.
- Adapter version field in `EngineCapabilities`.
