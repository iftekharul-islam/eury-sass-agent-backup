# Agent Engine Abstraction

Spec-Version: 1.0.0

## Purpose

Isolate the product from any single agent SDK. **Cersei** is the v1 implementation behind `AgentEngine`; UI, storage, and tests depend only on our trait and event types.

## Layer diagram

```
┌─────────────────────────────────────┐
│  React UI                           │
└──────────────┬──────────────────────┘
               │ IPC + AgentEvent
┌──────────────▼──────────────────────┐
│  agent-core (orchestration)         │
│  - RunController                    │
│  - Context assembly                 │
│  - Policy gate                      │
│  - Event fan-out                    │
└──────────────┬──────────────────────┘
               │ AgentEngine trait
┌──────────────▼──────────────────────┐
│  cersei_adapter                     │
│  - Agent::builder()                 │
│  - run_stream()                     │
│  - PermissionPolicy bridge          │
│  - Hook registration                │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Cersei SDK                         │
└─────────────────────────────────────┘
```

## AgentEngine trait (normative sketch)

```rust
#[async_trait]
pub trait AgentEngine: Send + Sync {
    /// Start a streaming run. Events delivered via callback/channel.
    async fn run_stream(
        &self,
        request: RunRequest,
        events: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<RunOutcome, AgentError>;

    /// List tools available for current mode + policy.
    fn tool_definitions(&self) -> Vec<ToolDefinition>;

    /// Engine capabilities for UI (streaming, thinking, sub-agents).
    fn capabilities(&self) -> EngineCapabilities;
}
```

`RunRequest` includes: `run_id`, `mode`, `prompt`, `context_messages`, `workspace_root`, `model_config`, `max_turns`, `plan_context`, `project_id`.

## Cersei adapter responsibilities

| Responsibility | Cersei API |
|----------------|------------|
| Build agent | `Agent::builder().provider().tools().permission_policy().hooks()` |
| Stream | `agent.run_stream(prompt)` |
| Map events | 26 `AgentEvent` variants → our stable `AgentEvent` |
| Permissions | `InteractivePolicy` → UI approval flow |
| Cost guard | `Hook` implementing budget check |
| Memory inject | `MemoryManager::build_context()` in system prompt |
| Sub-agents | `SubAgentSpawned` / `SubAgentComplete` → our multi-agent UI |

## Event mapping table

| Cersei `AgentEvent` | Our `AgentEvent` |
|---------------------|------------------|
| `TextDelta` | `text_delta` |
| `ThinkingDelta` | `thinking_delta` (optional UI) |
| `ToolStart` | `tool_start` |
| `ToolEnd` | `tool_end` |
| `PermissionRequired` | `approval_required` |
| `TurnComplete` | `turn_complete` |
| `TokenWarning` | `context_warning` |
| `CompactStart/End` | `compact_start` / `compact_end` |
| `SubAgentSpawned` | `subagent_spawned` |
| `SubAgentComplete` | `subagent_complete` |
| `CostUpdate` | `cost_update` |
| `Complete` | `run_complete` |
| `Error` | `run_error` |

Unmapped Cersei variants MUST be logged at `debug` and ignored in UI v1.

## Tool execution path

Cersei dispatches tools in-process. Our custom tools wrap `agent-sandbox`:

```rust
#[async_trait]
impl Tool for SandboxReadFile {
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let path = parse_path(&input)?;
        sandbox::read_file(ctx.workspace_root(), path).into_tool_result()
    }
}
```

Built-in Cersei filesystem tools MAY be disabled in favor of our sandboxed implementations (decision: **use custom sandbox tools** for consistent policy enforcement).

## Testing strategy

- **Unit tests:** mock `AgentEngine` returning canned event streams.
- **Integration tests:** Cersei adapter against fixture workspace (no network) with stub provider.
- **Contract tests:** golden files for event JSON schema.

## Migration off Cersei (hypothetical)

If Cersei is replaced:

1. Implement `AgentEngine` for new SDK.
2. Swap DI in `agent-core` factory.
3. UI unchanged if event protocol stable.

## Related documents

- [ADR-0001](adr/0001-embed-cersei-in-desktop.md)
- [ADR-0003](adr/0003-agent-engine-trait-boundary.md)
- [../04-specs/01-agent-runtime-spec.md](../04-specs/01-agent-runtime-spec.md)
- [../04-specs/03-event-protocol-spec.md](../04-specs/03-event-protocol-spec.md)
