# Desktop Runtime

Spec-Version: 1.0.0

## Process model

Eury Agent runs as a **single Tauri process** on desktop:

| Part | Technology | Thread model |
|------|------------|--------------|
| WebView | React 19 | Main UI thread |
| Tauri runtime | Rust | Main + async Tokio pool |
| Agent runs | Cersei on Tokio | One active run default; queue optional |
| Indexer | Rust background | Dedicated worker thread(s) |
| PTY terminal | Rust + native PTY | Separate thread per session |

No separate agent daemon. No Node subprocess for the agent loop.

## Directory layout (planned)

```
agent/
  apps/
    desktop/
      src/                    # React frontend
      src-tauri/
        src/
          main.rs
          lib.rs
          commands/           # Tauri command handlers
          state.rs            # AppState
        tauri.conf.json
        Cargo.toml
  crates/
    agent-core/
    agent-tools/
    agent-sandbox/
    agent-policy/
    agent-store/
    agent-index/
    agent-memory/
    agent-types/              # Shared types (no heavy deps)
  Cargo.toml                  # Workspace root
  package.json                # pnpm workspace for desktop UI
```

## Crate dependency graph

```
agent-types (leaf)
    ↑
agent-sandbox, agent-policy, agent-store, agent-index, agent-memory
    ↑
agent-tools
    ↑
agent-core (Cersei adapter)
    ↑
desktop src-tauri (commands only — thin)
```

**Rule:** `apps/desktop` MUST NOT depend on `cersei` directly. Only `agent-core` imports Cersei.

## Tauri integration

### Commands (UI → Rust)

Synchronous or async invoke. See [../04-specs/04-ipc-command-spec.md](../04-specs/04-ipc-command-spec.md).

Examples: `workspace_open`, `agent_run_start`, `agent_run_cancel`, `approval_respond`.

### Events (Rust → UI)

Unidirectional streaming over two paths:

- Run stream: a `Channel<AgentEvent>` passed into `agent_run_start`, one per run, closed on the terminal event
- App topic: the global event `agent://app` for update, policy, auth, and trust changes

Payload: JSON matching the [event protocol](../04-specs/03-event-protocol-spec.md). The UI subscribes per run rather than demultiplexing a global stream, and recovers from a gap or reload via `agent_run_snapshot` ([ADR-0008](adr/0008-event-protocol-over-tauri-channels.md)).

## App data directories

| OS | Base path |
|----|-----------|
| macOS | `~/Library/Application Support/com.eury.agent/` |
| Windows | `%APPDATA%\com.eury.agent\` |
| Linux | `~/.local/share/com.eury.agent/` |

| Subpath | Contents |
|---------|----------|
| `db/agent.sqlite` | Encrypted SQLite (SQLCipher or equivalent) |
| `memory/<workspace_hash>/` | Graph DB files |
| `index/<workspace_hash>/` | Index segments |
| `logs/` | Rotating structured logs |
| `cache/` | Ephemeral (safe to delete) |
| `checkpoints/<run_id>/` | Rollback snapshots |

Override via env `EURY_AGENT_DATA_DIR`.

## Workspace layout

Per opened workspace:

```
<workspace>/
  .eury/
    plans/              # Plan markdown files
    agent.json          # Workspace metadata (agent version, last index)
  EURY.md               # Project instructions (optional)
  .eury.local.md        # Gitignored local overrides (optional)
```

## Startup sequence

1. Parse CLI args (`--workspace`, `--deep-link`).
2. Initialize logging + crash handler.
3. Open SQLite; run migrations.
4. Load settings + keychain probe.
5. Show UI (splash → main window).
6. Background: check update manifest, validate session.
7. If workspace arg: open workspace + warm index.

## Single active run

Default: **one agent run at a time** per window. Rationale: simpler approval UX, predictable resource use. Phase 21 may add background runs with explicit UI surfacing.

## Resource limits

| Resource | Default limit |
|----------|----------------|
| Concurrent index workers | `min(4, CPU cores)` |
| Max open editor tabs | 50 |
| Max terminal sessions | 5 |
| Agent run wall clock | 30 min (configurable) |
| Max turns per run | 50 (configurable) |

## Platform-specific notes

| Platform | Sandbox | Packaging |
|----------|---------|-----------|
| macOS | Seatbelt profile | .dmg, notarized |
| Windows | Job objects + ACL | .msi, Authenticode |
| Linux | Landlock + namespaces where available | .deb, .AppImage |

See [../03-security/02-sandbox-model.md](../03-security/02-sandbox-model.md).

## Related documents

- [04-agent-engine-abstraction.md](04-agent-engine-abstraction.md)
- [../04-specs/04-ipc-command-spec.md](../04-specs/04-ipc-command-spec.md)
- [../04-specs/05-local-data-model.md](../04-specs/05-local-data-model.md)
