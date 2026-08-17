# Editor, Terminal, Preview

Spec-Version: 1.2.0

These three surfaces make the agent's work inspectable. None of them is a full IDE — Eury Agent is an agent client, and the IDE is a separate product ([non-goals](../01-product/05-non-goals.md)).

## Editor

CodeMirror 6. Chosen over Monaco for bundle size (~350 KB vs ~2.5 MB gzipped), faster cold mount, and simpler decoration API for diff overlays.

| Capability | v1 | Notes |
|---|---|---|
| Syntax highlighting | Yes | 20 bundled languages + plain fallback |
| Read-only mode | Yes | Default for untrusted workspaces |
| Editing + save | Yes | `Cmd/Ctrl+S`; writes go through the sandbox path guard |
| Diff decorations | Yes | Live write preview ([tool activity](04-tool-activity-and-diff-ux.md)) |
| Search in file | Yes | `Cmd/Ctrl+F`, regex toggle |
| Go to line | Yes | `Cmd/Ctrl+G` |
| Multi-cursor | Yes | CodeMirror default |
| Format on save | Optional | Runs the workspace formatter as an execute-class tool (needs grant) |
| LSP (completion, hover, rename) | No | Deferred; tracked in [open questions](../09-roadmap/open-questions.md) |
| Debugger | No | Out of scope |

| Rule | Value |
|---|---|
| Max file size opened | 8 MB; larger opens read-only, highlight disabled |
| Minified/one-line files | Highlight disabled above 5000 columns |
| Binary files | Not opened; hex summary only |
| Encoding | Detected; preserved on save (`UTF-8`, `UTF-8 BOM`, `UTF-16LE`) |
| Line endings | Preserved per file; shown in the tab's status chip |
| External change | Watcher detects; unmodified buffer reloads silently, modified buffer prompts |
| Agent change while open | Buffer reloads and shows a "changed by agent" chip with jump-to-hunk |
| Unsaved on quit | Blocking prompt listing dirty files |

Files are read and written only via Rust commands, never `fetch` or direct FS from the webview ([IPC commands](../04-specs/04-ipc-command-spec.md)).

## Terminal

xterm.js in the UI, PTY in Rust (`portable-pty`).

| Aspect | Rule |
|---|---|
| Shell | `$SHELL` on macOS/Linux, PowerShell 7 → `cmd.exe` fallback on Windows |
| Sessions | Up to 4 concurrent per workspace; each its own tab |
| cwd | Workspace root, or the tool's cwd when promoted from a tool card |
| Data path | PTY bytes → Rust ring buffer → Tauri channel → xterm write |
| Throughput | ≥ 10 MB/s without UI stall; output coalesced per frame, 4 MB scrollback per session |
| Resize | Debounced 50 ms, forwarded as `SIGWINCH` |
| Env | Inherits a sanitized environment; secrets from the keychain are never injected |
| Agent-visible? | Yes when the user attaches it: the agent can read the last N lines only after an explicit "share terminal output" action |
| User input | Always allowed; a human-typed command is not an agent tool call and is not audited as one |
| Kill | Tab close sends `SIGHUP` then `SIGKILL` after 2 s |
| Windows ConPTY | Required; degraded warning if unavailable |

Agent-initiated commands run through the `run_command` tool with policy and sandbox enforcement — never by typing into a user terminal. Promoting a tool's execution into a terminal tab is a view change, not a permission change.

## Preview

Webview panel for local dev servers and static output.

| Aspect | Rule |
|---|---|
| Allowed origins | `http://localhost:*`, `http://127.0.0.1:*`, `file://` inside the workspace |
| Other origins | Blocked with an "open in browser" fallback |
| Isolation | Separate webview with its own session; no access to app IPC, no shared storage |
| Detection | Offers a preview when a tool's output announces a listening port |
| Controls | Reload, hard reload, address field (validated), device-width presets, open-in-browser |
| Console capture | Errors and warnings only, forwarded to the Activity panel; can be shared to the agent explicitly |
| Screenshot | Captures the preview to a temp file the agent may read after approval |
| DevTools | Available in dev builds only |

The preview is untrusted content. Anything it produces (console text, screenshots, DOM extracts) is marked untrusted before it can enter a prompt ([prompt injection defense](../03-security/05-prompt-injection-defense.md)).

## Cross-surface actions

| From | Action |
|---|---|
| Chat path reference | Open file at line |
| Diff hunk | Open file at hunk, or apply hunk |
| Tool card (`run_command`) | Promote to a terminal tab |
| Terminal selection | Send to composer as an attachment |
| Editor selection | "Ask about selection" / "Refactor selection" |
| Preview console error | "Fix this error" prefilled prompt |
| Search result | Open file at match |

## Performance targets

| Action | Target |
|---|---|
| Open 1000-line file | ≤ 120 ms to interactive |
| Open 100k-line file | ≤ 600 ms, virtualized |
| Terminal first prompt | ≤ 300 ms after tab open |
| Preview first paint | Bounded by the dev server, ≤ 100 ms overhead |
| Editor keystroke latency | ≤ 16 ms p95 while a run streams |

## Accessibility

The editor exposes CodeMirror's accessible textarea; the terminal exposes xterm's screen-reader mode (opt-in, announces last line). The preview is an `iframe`-equivalent region with a labeled landmark. All three are reachable and closable by keyboard alone.

## Related documents

- [App shell and navigation](02-app-shell-and-navigation.md)
- [Tool catalog](../04-specs/02-tool-catalog-spec.md)
- [Sandbox model](../03-security/02-sandbox-model.md)
