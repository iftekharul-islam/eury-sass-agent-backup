# Phase 04 — Agent Core

Spec-Version: 1.2.0

**Track:** B — Core runtime · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Run the agent loop behind the `AgentEngine` trait and stream a real model response end to end, from the Rust core to the UI, with a stub provider for determinism.

## Why this phase exists here

This is the phase that validates the central architectural bet ([ADR-0001](../02-architecture/adr/0001-embed-cersei-in-desktop.md)). If the abstraction or the streaming path is wrong, everything after it is built on sand.

**Architecture as shipped: managed gateway, not embedded Cersei.** This supersedes the original "Cersei embedded, no network hop" framing. The engine wired into the desktop app is `AgentLoopEngine`, which reaches models through the managed gateway (`/agent/v1/chat/stream`) rather than running Cersei in-process; `CerseiAdapter` is retained as scaffolding and is **not** on the live path. The reason is enforcement: org control, cost caps, usage metering, and model-policy filtering (Phase 11) cannot be enforced from a client the user controls. The agent loop, tool dispatch, policy, sandboxing, and all filesystem access still run locally — the hop is on model inference only, and Phase 4's latency targets account for it. The `AgentEngine` trait boundary is what makes this a one-crate substitution, and it keeps an embedded/BYOK engine viable later. Tool calls come from the gateway's typed `tool_call` NDJSON events and are assembled by `ToolCallAccumulator` — never by scanning assistant text for ` ```tool_call ` fences, which would execute prose that merely resembles a tool call.

## In scope

- `AgentEngine` trait: run lifecycle, streaming, abort, hook points
- Gateway-backed engine implementing the trait, isolated in one crate
- Run manager: run ids, concurrency limits, cancellation, cleanup
- Event mapping from engine events to the wire protocol
- Structured `tool_call` event assembly (no parsing of assistant prose)
- Provider abstraction with a deterministic stub provider plus the managed gateway
- Prompt assembly v1: system prompt, mode framing, history, untrusted-content marking
- Token counting, context-window accounting, and a cost estimator
- Structured error mapping into the `EURY_*` taxonomy

## Feature IDs

`F-008`, `F-021`, `F-025`

## Out of scope

- Tools (Phase 6)
- Persistence (Phase 9)
- Retrieval (Phase 15)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D4.1 | `AgentEngine` trait with documented invariants | [engine abstraction](../02-architecture/04-agent-engine-abstraction.md) |
| D4.2 | Gateway-backed engine behind the trait, no provider types leaking outward | [ADR-0003](../02-architecture/adr/0003-agent-engine-trait-boundary.md) |
| D4.3 | Run manager with abort within 250 ms and guaranteed resource cleanup | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D4.4 | Event stream: `meta`, `delta`, `reasoning`, `usage`, `done`, `error` | [event protocol](../04-specs/03-event-protocol-spec.md) |
| D4.5 | Stub provider with scripted responses and tool calls | [test strategy](../08-quality/01-test-strategy.md) |
| D4.6 | BYOK provider client with keychain-sourced credentials | [secrets](../03-security/04-secrets-and-key-management.md) |
| D4.7 | Prompt assembly with explicit untrusted-content regions | [injection defense](../03-security/05-prompt-injection-defense.md) |
| D4.8 | Token and cost accounting with a versioned price table | [latency budget](../02-architecture/05-latency-budget.md) |

## Key decisions and design notes

- The agent loop runs locally; model inference goes through the managed gateway. Tool execution, policy, and file access never leave the machine.
- The trait boundary exists so an engine or provider change is a one-crate change; product code never imports engine-specific types.
- Tool calls come from typed `tool_call` stream events, never from parsing assistant prose.
- The stub provider is a first-class deliverable — deterministic agent tests are impossible without it.
- Aborts are cooperative but bounded: the UI reflects cancellation within 250 ms even if unwinding takes longer.

## Contracts touched

- `AgentEngine` trait
- Run event stream variants
- Provider interface
- Error taxonomy mapping for engine and provider failures

## Dependencies

- Phase 3 (shell, IPC, event plumbing)
- Phase 2 (untrusted-content rules)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Cersei API gaps or instability | Blocked or forked | Trait boundary plus an internal fallback loop kept behind the same trait; vendor pinned by exact version |
| Streaming backpressure | UI stalls | Frame-batched consumption with documented drop priorities; load test with a synthetic fast stream |
| Prompt assembly sprawl | Unpredictable behavior | Assembly is a pure function over typed inputs with golden-output tests |
| Token accounting drift | Wrong cost and context decisions | Per-provider tokenizer tests against recorded responses |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Run lifecycle, abort, concurrency limits, event mapping |
| Integration | Full stream with the stub provider; recorded-cassette replay for real providers |
| Contract | Event golden fixtures for every variant |
| Load | Synthetic 200-events/second stream stays within the frame budget |
| Negative | Provider 429/500/timeout/malformed-chunk handling |

## Metrics and targets

| Metric | Target |
|---|---|
| Time to first token (managed gateway, incl. network hop) | < 800 ms p95 |
| Engine overhead per turn (excl. model) | < 20 ms p95 |
| Abort acknowledged in UI | < 250 ms p95 |
| Frame rate while streaming | ≥ 55 fps |

## Exit criteria

- [ ] A prompt streams a response end to end with the stub and one real provider
- [ ] Abort cancels cleanly with no orphaned tasks or leaked handles
- [ ] No engine-specific type appears outside the adapter crate (enforced by lint)
- [ ] Event golden fixtures cover every variant
- [ ] Latency targets met on reference hardware
- [ ] Provider failures surface as taxonomy codes, never raw strings
- [ ] `PromptAssembler` (D4.7) is invoked by the live path — it is currently dead code

## Deferred from this phase

- Tool execution (Phase 6)
- Retrieval-based context (Phase 15)
- Sub-agents (Phase 20)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
