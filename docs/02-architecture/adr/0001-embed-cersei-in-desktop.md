# ADR-0001: Embed Cersei in Desktop

Spec-Version: 1.1.0

**Status:** Superseded (see "Amendment" below)  
**Date:** 2026-08-16

## Context

Eury Agent needs a production-grade agent runtime: tool dispatch, streaming, memory, MCP, sub-agents. Cersei is a Rust SDK designed for this. Alternatives: (a) separate Cersei/agent server, (b) rebuild agent loop in-house, (c) call cloud-only agent API.

Forces: low latency, local filesystem access, privacy, offline-capable editor, single installer.

## Decision

Embed Cersei as a Rust dependency inside the Tauri desktop process. No separate agent server or sidecar for the agent loop.

## Consequences

**Positive:**
- Zero network hop for tool execution.
- Single process to sign, update, and debug.
- Aligns with Cersei's in-process tool dispatch design.

**Negative:**
- Desktop binary size increases (Rust + Cersei crates).
- Cersei version pinned to app release cycle.
- Crash in Cersei can crash app (mitigate: isolate runs with panic boundaries + checkpoints).

**Mitigations:**
- `AgentEngine` trait boundary (ADR-0003).
- Fuzz and integration tests on adapter (Phase 28).

## Amendment (2026-08-17): superseded by the managed gateway

The decision above was not what shipped, and the code is now the source of
truth. The engine wired into the desktop app is `AgentLoopEngine`, which
reaches models over the managed gateway (`/agent/v1/chat/stream`).
`CerseiAdapter` exists but is not on the live path.

**What changed and why.** Org control and billing must be enforced somewhere
the user cannot bypass, which rules out a client-side-only path: server-side
usage metering, cost caps, and model-policy filtering (Phase 11) all depend
on inference flowing through the gateway. Managed users also get model access
without configuring a local provider key.

**What did not change.** The agent loop, tool dispatch, policy evaluation,
sandboxing, and all filesystem access still run in-process on the user's
machine. "Zero network hop for tool execution" above still holds — the hop is
on model inference only. The `AgentEngine` trait boundary (ADR-0003) is what
made this substitution a one-crate change, and it is why an embedded engine
remains available later without re-architecting.

**What this costs.** A network hop on the inference path, reflected in
Phase 4's latency targets, and no offline inference. Offline behavior is
covered by [offline modes](../06-offline-and-degraded-modes.md).

An embedded/BYOK engine behind the same trait is still viable if
BYOK-by-default is revisited; that would be a new ADR, not a revival of this
one as written.
