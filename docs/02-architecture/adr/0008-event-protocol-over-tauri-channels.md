# ADR-0008: Event Protocol over Tauri Channels

Spec-Version: 1.1.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Agent streaming requires high-frequency updates (text deltas, tool output, diff previews). Polling IPC is inefficient; embedding state in command responses loses ordering guarantees.

Alternatives considered: a localhost WebSocket, shared memory, polling `agent_poll_state`, and a single global Tauri event topic.

## Decision

Use a **Tauri `Channel<AgentEvent>` per run**, passed into `agent_run_start`, for all streaming agent output. A single global topic (`agent://app`) carries only low-frequency app-level events: update availability, policy refresh, auth changes, and workspace trust changes.

Per-run channels beat one global topic because teardown is automatic when the run ends, there is no cross-run fan-out cost, and no window has to deserialize and filter another window's deltas in the hot path. Commands start, cancel, and steer runs and carry approval responses. The payload is versioned JSON per the [event protocol spec](../../04-specs/03-event-protocol-spec.md).

## Consequences

**Positive:**
- Low overhead with a total order per run and no demultiplexing in the UI.
- Natural lifecycle: the channel closes with the run, so leaks are structurally unlikely.
- Simple React subscription model, one hook per active run.

**Negative:**
- No built-in backpressure, so coalescing must happen in Rust.
- Recovery after a UI reload needs an explicit snapshot path rather than replaying a durable topic.

**Mitigations:**
- Deltas are coalesced in Rust and rendered at most once per animation frame.
- A gapless `seq` per run detects loss; `agent_run_snapshot` restores full state after a gap or reload.
- Terminal events are never coalesced or dropped.
