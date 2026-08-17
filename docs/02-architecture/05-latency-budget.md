# Latency Budget

Spec-Version: 1.0.0

## Principle

**LLM inference dominates end-to-end latency.** Local architecture MUST NOT add meaningful overhead vs. a direct provider call. Target: local overhead < 100 ms p95 per turn (excluding model and network).

## Budget breakdown (single turn, Agent mode)

| Stage | Budget (p95) | Owner |
|-------|----------------|-------|
| UI keystroke → IPC receive | 5 ms | Tauri |
| Context assembly (index + memory + history) | 30 ms | agent-core |
| Policy evaluation | 2 ms | agent-policy |
| Serialize request | 3 ms | agent-core |
| **Network to provider (BYOK)** | 50–2000+ ms | External |
| **Network to gateway (managed)** | 60–2100+ ms | Cloud + external |
| First token (TTFT) | Provider-dependent | — |
| Tool dispatch (read_file, warm cache) | 10 ms | agent-sandbox |
| Tool dispatch (grep 10k files) | 200 ms | agent-index |
| Tool dispatch (shell, short cmd) | 500 ms | agent-sandbox |
| Event UI render (per chunk) | 8 ms | React |
| SQLite persist (async) | 5 ms (non-blocking) | agent-store |

## Cersei-specific expectations

Per [Cersei docs](https://cersei.tryatlas.cc/docs):

- In-process tool dispatch: sub-millisecond for pure I/O tools.
- Graph memory recall: ~100 µs (validate in our harness — Phase 28).

We do **not** treat vendor benchmarks as SLOs until reproduced in [../08-quality/03-performance-benchmarks.md](../08-quality/03-performance-benchmarks.md).

## Anti-patterns (forbidden)

| Pattern | Latency cost | Alternative |
|---------|--------------|-------------|
| Separate agent microservice | +20–100 ms RTT per turn | Embedded Cersei |
| Re-read entire file tree per turn | Seconds | Incremental index |
| Synchronous SQLite on hot path | 10–50 ms | Async write-behind |
| Full conversation in every request | Large TTFT | Compaction + retrieval |
| Markdown fence tool parsing | Retry loops | Native structured tool calls |

## BYOK vs managed gateway

| Path | Extra hops | When to use |
|------|------------|-------------|
| BYOK direct | 0 (desktop → provider) | Solo dev, lowest latency |
| Managed gateway | 1 (desktop → Nest → provider) | Teams, no key management |

Gateway overhead target: < 30 ms p95 excluding upstream LLM.

## Streaming UX budget

User perceives responsiveness when:

- First UI feedback (typing indicator) < 50 ms after send.
- First `text_delta` or `tool_start` < TTFT + 20 ms local.
- Tool card appears < 100 ms after `tool_start`.

## Measurement

Instrument with OpenTelemetry spans:

- `agent.run.start` → `agent.context.build` → `agent.model.request` → `agent.tool.execute` → `agent.run.complete`

Dashboards in Phase 26.

## Related documents

- [ADR-0001](adr/0001-embed-cersei-in-desktop.md)
- [ADR-0005](adr/0005-byok-vs-managed-gateway.md)
- [06-offline-and-degraded-modes.md](06-offline-and-degraded-modes.md)
