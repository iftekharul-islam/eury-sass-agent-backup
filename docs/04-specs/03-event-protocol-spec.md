# Event Protocol Specification

Spec-Version: 2.0.0

The wire format between the Rust core and the React UI. This is a versioned contract with golden fixtures on both sides ([ADR-0008](../02-architecture/adr/0008-event-protocol-over-tauri-channels.md)).

## Transport

| Channel | Mechanism | Carries |
|---|---|---|
| Run stream | A Tauri `Channel<AgentEvent>` passed into `agent_run_start`, one per run | All high-frequency run events |
| App topic | The global Tauri event `agent://app` | Update availability, policy refresh, auth changes, workspace trust changes |

Per-run channels are used because they give us natural teardown, no cross-run fan-out cost, and no filtering in the UI hot path. A global topic would force every window to deserialize every other run's deltas.

| Property | Guarantee |
|---|---|
| Ordering | Total order per run, in emission order |
| Sequencing | `seq` starts at 1, increments by exactly 1, never reused (matches the runtime's I5) |
| Delivery | At-least-once for terminal events, exactly-once in practice; the UI must be idempotent on `seq` |
| Serialization | JSON, camelCase, `serde` with `#[serde(tag = "type", rename_all = "camelCase")]` |
| Backpressure | Deltas are coalesced in Rust before emit; the channel is never used as a queue |
| Teardown | The channel closes after the terminal event; late sends are dropped with a warning, never a panic |

## Envelope

```typescript
interface AgentEventEnvelope {
  specVersion: string;          // semver of THIS protocol, e.g. "2.0.0"
  runId: string;
  seq: number;                  // 1-based, gapless
  ts: string;                   // RFC3339 with milliseconds, monotonic per run
  parentRunId?: string;         // present for sub-agent events
  event: AgentEvent;
}
```

The UI **MUST** ignore unknown `event.type` values and unknown fields rather than erroring, so a newer core never breaks an older UI beyond missing features.

## Event catalog

### Lifecycle

```typescript
| { type: "runQueued"; position: number }
| { type: "runStarted"; mode: Mode; model: string; route: "byok" | "managed";
    policyDigest: string; contextWindow: number }
| { type: "phaseChanged"; phase: RunPhase }        // mirrors the runtime state machine
| { type: "turnStarted"; turn: number }
| { type: "turnCompleted"; turn: number; usage: TokenUsage; stopReason: string }
| { type: "runCompleted"; outcome: RunOutcome }
| { type: "runFailed"; error: ErrorPayload }
| { type: "runCancelled"; reason: "user" | "quit" | "parent" | "policy"; partialText: string }
| { type: "runLimited"; limit: "turns" | "wall" | "cost" | "tokens" | "toolCalls"; detail: string }
```

### Content

```typescript
| { type: "textDelta"; text: string }
| { type: "reasoningDelta"; text: string; summarized: boolean }
| { type: "reasoningEnded"; tokens: number }
| { type: "citations"; sources: Citation[] }
```

### Tools

```typescript
| { type: "toolCallProposed"; toolCallId: string; name: string; class: ToolClass;
    risk: Risk; input: object; inputRedacted: boolean }
| { type: "toolCallStarted"; toolCallId: string; startedAt: string }
| { type: "toolOutputDelta"; toolCallId: string; stream: "stdout" | "stderr"; text: string }
| { type: "toolCallEnded"; toolCallId: string; ok: boolean; durationMs: number;
    summary: string; metadata?: object; checkpointId?: string;
    error?: ErrorPayload }
| { type: "writePreview"; toolCallId: string; path: string; hunks: DiffHunk[]; complete: boolean }
| { type: "fileChanged"; path: string; action: FileAction; linesAdded: number; linesRemoved: number }
```

### Approvals and policy

```typescript
| { type: "approvalRequired"; approvalId: string; toolCallId: string; name: string;
    class: ToolClass; risk: Risk; summary: string; payload: ApprovalPayload;
    scopesOffered: GrantScope[]; expiresAt: string }
| { type: "approvalResolved"; approvalId: string; decision: GrantDecision;
    scope?: GrantScope; source: "user" | "policy" | "timeout" }
| { type: "policyDenied"; toolCallId?: string; rule: string; scope: "org" | "workspace" | "user";
    message: string; contactHint?: string }
```

### Context and cost

```typescript
| { type: "contextUsage"; usedTokens: number; windowTokens: number; pct: number;
    level: "ok" | "warning" | "critical" }
| { type: "compactStarted"; reason: "threshold" | "manual"; tokensBefore: number }
| { type: "compactEnded"; tokensAfter: number; tokensFreed: number; messagesReplaced: number }
| { type: "costUpdate"; turnCostUsdMicros: number; runCostUsdMicros: number;
    estimated: boolean; budgetPct?: number }
| { type: "retrievalUsed"; chunks: number; files: number; tokens: number; latencyMs: number }
| { type: "memoryRecalled"; hits: { id: string; kind: string; text: string }[] }
| { type: "memoryProposed"; proposalId: string; kind: string; text: string; confidence: number }
```

### Sub-agents

```typescript
| { type: "subagentSpawned"; agentRunId: string; role: SubagentRole; goal: string;
    toolClasses: ToolClass[]; worktree?: string }
| { type: "subagentProgress"; agentRunId: string; turn: number; note: string }
| { type: "subagentCompleted"; agentRunId: string; ok: boolean; summary: string;
    usage: TokenUsage }
```

### Diagnostics

```typescript
| { type: "notice"; level: "info" | "warn" | "error"; code?: string; message: string }
| { type: "injectionSuspected"; toolCallId: string; source: string; severity: "low" | "medium" | "high" }
| { type: "heartbeat"; elapsedMs: number }     // every 10 s while non-terminal
```

`heartbeat` exists so the UI can distinguish "the model is thinking" from "the core is wedged" without polling.

## Shared payload types

```typescript
type RunPhase = "queued" | "assembling" | "streaming" | "toolRunning"
              | "awaitingApproval" | "paused" | "compacting"
              | "complete" | "failed" | "limited" | "cancelled";

type GrantScope = "once" | "run" | "session" | "always";
type GrantDecision = "allow" | "deny";
type Risk = "low" | "medium" | "elevated" | "critical";
type FileAction = "created" | "modified" | "deleted" | "moved";

interface TokenUsage {
  promptTokens: number; completionTokens: number;
  cachedTokens: number; reasoningTokens: number;
}

interface ErrorPayload {
  code: string;                  // see error taxonomy
  message: string;               // already localized key + params on the UI side
  retryable: boolean;
  details?: Record<string, unknown>;
  docsUrl?: string;
}

interface ApprovalPayload {
  command?: { argv: string[]; shell: boolean; cwd: string; explanation: string };
  diff?: { path: string; hunks: DiffHunk[]; linesAdded: number; linesRemoved: number };
  paths?: string[];
  url?: string;
  mcp?: { server: string; tool: string; args: object };
}

interface Citation {
  kind: "file" | "url" | "memory";
  path?: string; url?: string; title?: string;
  range?: { startLine: number; endLine: number };
}
```

`GrantScope` here is the same vocabulary used by the policy engine, the IPC layer, and the approval UI. There is exactly one grant vocabulary in the product.

## Batching and rate rules

| Event | Coalescing |
|---|---|
| `textDelta`, `reasoningDelta` | Concatenated and flushed at most every 33 ms (≈30 fps), or immediately when ≥ 4 KB accumulates |
| `toolOutputDelta` | Flushed every 50 ms or 8 KB, per stream, per tool call |
| `contextUsage`, `costUpdate` | At most once per second, plus once on every threshold crossing |
| `writePreview` | At most every 100 ms; the final emit always has `complete: true` |
| `heartbeat` | Every 10 s while non-terminal |
| Everything else | Emitted immediately, never coalesced |

Emission **MUST NOT** block the agent loop. If the channel is full, deltas may be merged further, but no non-delta event may be dropped.

## UI handling rules

| Event | Required UI behavior |
|---|---|
| `runQueued` | Show queue position; keep the composer enabled for editing |
| `runStarted` | Create the turn container; show model and route badge |
| `textDelta` | Append to the streaming buffer; render at most once per animation frame |
| `reasoningDelta` | Append to the collapsible reasoning block (collapsed by default) |
| `toolCallProposed` | Insert a tool card in `queued` state |
| `approvalRequired` | Render the approval card; focus it; `Esc` denies |
| `toolOutputDelta` | Append to the row's output pane with a virtualized log view |
| `writePreview` | Update the diff panel in place; never scroll-jump the chat |
| `fileChanged` | Update the Changes panel counters |
| `contextUsage` | Update the meter; at `critical`, surface the compact action |
| `policyDenied` | Show a non-dismissible inline notice with the rule and its scope |
| `injectionSuspected` | Badge the row and require approval for any follow-on privileged action |
| `runCompleted` / `runFailed` / `runCancelled` / `runLimited` | Finalize the turn, enable retry affordances, stop the timer |

## Recovery

The UI keeps `lastSeq` per run.

| Condition | Action |
|---|---|
| `seq == lastSeq + 1` | Normal apply |
| `seq <= lastSeq` | Duplicate; ignore silently |
| `seq > lastSeq + 1` | Gap: mark the run "resyncing", call `agent_run_snapshot`, replace local state, resume from the snapshot's `seq` |
| Channel closed without a terminal event | Call `agent_run_snapshot`; if the core reports no active run, mark the run interrupted and offer resume or revert |
| App reload mid-run | Rehydrate from `agent_run_snapshot` and re-subscribe |

`agent_run_snapshot` returns a complete, self-consistent run state, so recovery never requires replaying history from event storage.

## Versioning

| Change | Requirement |
|---|---|
| New event type | Minor bump; UI ignores unknown types |
| New optional field | Minor bump |
| Removed or renamed field, changed semantics, changed enum meaning | Major bump |
| Major bump | The core supports the previous major for one release; the UI declares `minSupportedSpec` at startup |

Mismatch handling: if the core's `specVersion` major exceeds the UI's supported major, the app shows a blocking "restart to finish updating" state rather than rendering a partially understood stream.

## Relationship to the cloud stream

The managed gateway's NDJSON stream (`meta`, `delta`, `reasoning`, `toolCall`, `usage`, `done`, `error`) is a **different, narrower protocol** ([cloud API contract](06-cloud-api-contract.md)). The core translates it:

| Gateway frame | Local event |
|---|---|
| `meta` | absorbed into `runStarted` |
| `delta` | `textDelta` |
| `reasoning` | `reasoningDelta` |
| `toolCall` | `toolCallProposed` |
| `usage` | `turnCompleted.usage` + `costUpdate` |
| `done` | drives the turn loop, not emitted directly |
| `error` | `runFailed` or a retry, depending on the code |

The UI never sees gateway frames directly. This keeps BYOK and managed routes visually identical.

## Conformance tests

| ID | Test |
|---|---|
| T1 | Golden fixture: one recorded run's full event array round-trips Rust → JSON → TS types with no loss |
| T2 | Every event variant appears in at least one fixture, enforced by an exhaustiveness test |
| T3 | `seq` is gapless across a 10 000-event run under load |
| T4 | Duplicate and out-of-order injected events leave the UI state identical (idempotency) |
| T5 | A forced gap triggers exactly one snapshot call and full recovery |
| T6 | 60 s of 5 MB/s tool output keeps the UI above 30 fps and memory flat |
| T7 | An unknown event type and unknown fields are ignored without a console error |
| T8 | The terminal event is always last; nothing is emitted after it |
| T9 | Redacted inputs never contain seeded secrets |
| T10 | Gateway-frame to local-event translation matches a golden mapping fixture |

## Related documents

- [IPC commands](04-ipc-command-spec.md)
- [Agent runtime](01-agent-runtime-spec.md)
- [Chat and streaming UX](../05-ui/03-chat-and-streaming-ux.md)
- [ADR-0008](../02-architecture/adr/0008-event-protocol-over-tauri-channels.md)
