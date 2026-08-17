use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use agent_core::agent_loop::AgentLoopEngine;
use agent_core::approval::ApprovalWaiter;
use agent_core::run_manager::RunManager;
use agent_core::tool_executor::{ToolExecutor, ToolExecuteRequest};
use agent_types::events::AgentEvent;
use agent_types::requests::RunRequest;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_trusted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum IpcError {
    Internal(String),
    NotFound(String),
    NotAuthorized(String),
}

impl From<agent_types::errors::AgentError> for IpcError {
    fn from(err: agent_types::errors::AgentError) -> Self {
        IpcError::Internal(err.to_string())
    }
}

pub struct AppState {
    pub run_manager: Arc<RunManager>,
    pub tool_executor: Arc<ToolExecutor>,
    pub approvals: Arc<ApprovalWaiter>,
    pub workspace_path: std::sync::Mutex<Option<PathBuf>>,
    pub terminals: Arc<agent_sandbox::pty::PtyManager>,
    pub trust: Arc<agent_sandbox::workspace::TrustStore>,
    /// Encrypted local store (`SQLCipher`). Settings and workspace trust are
    /// persisted here rather than in plaintext `localStorage`.
    pub store: Arc<agent_store::actor::StoreActorHandle>,
    terminal_rate: TerminalRateLimiter,
}

impl AppState {
    /// Opens the encrypted store at `db_path` and builds the app state,
    /// restoring any previously trusted workspaces from it.
    ///
    /// # Errors
    ///
    /// Returns [`agent_types::errors::AgentError::Internal`] if the agent
    /// engine's grant store or the encrypted local store cannot be
    /// initialized.
    pub fn new(db_path: &std::path::Path) -> Result<Self, agent_types::errors::AgentError> {
        let approvals = Arc::new(ApprovalWaiter::new());
        let trust = Arc::new(agent_sandbox::workspace::TrustStore::new());
        let engine = Arc::new(AgentLoopEngine::new(approvals.clone(), trust.clone())?);
        let tool_executor = Arc::new(ToolExecutor::new(approvals.clone(), trust.clone())?);

        let store = agent_store::actor::spawn_actor(&db_path.to_string_lossy())
            .map_err(|e| agent_types::errors::AgentError::Internal(format!("store: {e}")))?;

        Ok(Self {
            run_manager: Arc::new(RunManager::new(engine)),
            tool_executor,
            approvals,
            workspace_path: std::sync::Mutex::new(None),
            terminals: Arc::new(agent_sandbox::pty::PtyManager::new(env!("CARGO_PKG_VERSION"))),
            trust,
            store: Arc::new(store),
            terminal_rate: TerminalRateLimiter::new(20),
        })
    }

    /// Repopulates the in-memory [`agent_sandbox::workspace::TrustStore`] from
    /// the `workspaces` table, so trust survives a restart instead of
    /// silently re-prompting.
    pub async fn restore_trusted_workspaces(&self) {
        let Ok(rows) = self
            .store
            .query("SELECT path FROM workspaces WHERE trust_state = 'trusted'".to_string(), vec![])
            .await
        else {
            return;
        };
        for row in rows {
            if let Some(rusqlite::types::Value::Text(path)) = row.first() {
                self.trust.set_trust(std::path::Path::new(path), true);
            }
        }
    }
}

/// Fixed-window limiter for `terminal_create` (20/min per the IPC spec).
/// Deliberately small — this isn't meant to be a precise leaky bucket, just
/// enough to blunt runaway session-spawn loops.
struct TerminalRateLimiter {
    capacity: u32,
    state: std::sync::Mutex<(u32, std::time::Instant)>,
}

impl TerminalRateLimiter {
    fn new(capacity: u32) -> Self {
        Self { capacity, state: std::sync::Mutex::new((capacity, std::time::Instant::now())) }
    }

    fn try_acquire(&self) -> bool {
        let mut guard = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let (remaining, window_start) = &mut *guard;
        if window_start.elapsed() >= std::time::Duration::from_mins(1) {
            *remaining = self.capacity;
            *window_start = std::time::Instant::now();
        }
        if *remaining == 0 {
            return false;
        }
        *remaining -= 1;
        true
    }
}

/// A stable id for "the currently open workspace", derived deterministically
/// from its path. The app only tracks one open workspace today (a single
/// `workspace_path: Option<PathBuf>`, not a `WorkspaceRegistry`), so this
/// exists purely to give `PtyManager`'s per-workspace session cap something
/// to key on without changing that existing single-workspace contract.
fn workspace_uuid(path: &std::path::Path) -> uuid::Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, path.to_string_lossy().as_bytes())
}

fn map_pty_err(err: agent_sandbox::pty::PtyError) -> IpcError {
    match err {
        agent_sandbox::pty::PtyError::NotFound => {
            IpcError::NotFound("Terminal session not found".into())
        }
        agent_sandbox::pty::PtyError::SessionLimit { .. } => {
            IpcError::NotAuthorized(err.to_string())
        }
        other => IpcError::Internal(other.to_string()),
    }
}

fn open_workspace_path(state: &State<'_, AppState>) -> Result<PathBuf, IpcError> {
    let guard = state.workspace_path.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.clone().ok_or_else(|| IpcError::NotFound("No workspace open".into()))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalCreated {
    pub terminal_id: uuid::Uuid,
    pub degraded: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalCapture {
    pub text: String,
}

/// `W` `T` `!` — requires an open **and trusted** workspace; always audited.
/// An untrusted workspace is read-only (no shell, no network, no MCP), so
/// spawning a terminal in one is refused.
///
/// # Errors
///
/// Returns an error if no workspace is open, the workspace is untrusted, the
/// rate limit is exceeded, or the PTY session fails to spawn.
#[tauri::command]
pub async fn terminal_create(
    cwd: Option<String>,
    shell: Option<String>,
    cols: u16,
    rows: u16,
    on_frame: tauri::ipc::Channel<agent_types::terminal::TerminalFrame>,
    state: State<'_, AppState>,
) -> Result<TerminalCreated, IpcError> {
    let workspace_path = open_workspace_path(&state)?;

    if !state.trust.is_trusted(&workspace_path) {
        return Err(IpcError::NotAuthorized(
            "This workspace is untrusted and runs read-only: no shell, no network, no MCP.".into(),
        ));
    }

    if !state.terminal_rate.try_acquire() {
        return Err(IpcError::NotAuthorized(
            "Rate limit exceeded for terminal_create (20/min)".into(),
        ));
    }

    let workspace_id = workspace_uuid(&workspace_path);
    let (tx, mut rx) = mpsc::channel::<agent_sandbox::pty::PtyEvent>(256);

    let info = state
        .terminals
        .create(
            workspace_id,
            &workspace_path,
            cwd.map(PathBuf::from),
            shell.as_deref(),
            cols,
            rows,
            tx,
        )
        .await
        .map_err(map_pty_err)?;

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let frame = match event {
                agent_sandbox::pty::PtyEvent::Data(f) => {
                    agent_types::terminal::TerminalFrame::Data {
                        seq: f.seq,
                        dropped_before: f.dropped_before,
                        bytes: f.bytes,
                    }
                }
                agent_sandbox::pty::PtyEvent::Exited { code } => {
                    agent_types::terminal::TerminalFrame::Exited { code }
                }
                agent_sandbox::pty::PtyEvent::Killed => {
                    agent_types::terminal::TerminalFrame::Killed
                }
            };
            if on_frame.send(frame).is_err() {
                break;
            }
        }
    });

    Ok(TerminalCreated { terminal_id: info.terminal_id, degraded: info.degraded })
}

/// # Errors
///
/// Returns an error if the terminal session does not exist.
#[tauri::command]
pub async fn terminal_write(
    terminal_id: uuid::Uuid,
    data: String,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    state.terminals.write(terminal_id, data.as_bytes()).await.map_err(map_pty_err)
}

/// # Errors
///
/// Returns an error if the terminal session does not exist.
#[tauri::command]
pub async fn terminal_resize(
    terminal_id: uuid::Uuid,
    cols: u16,
    rows: u16,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    state.terminals.resize(terminal_id, cols, rows).await.map_err(map_pty_err)
}

/// # Errors
///
/// Returns an error if the terminal session does not exist.
#[tauri::command]
pub async fn terminal_close(
    terminal_id: uuid::Uuid,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    state.terminals.close(terminal_id).await.map_err(map_pty_err)
}

/// # Errors
///
/// This command does not currently fail; it returns `Ok` even if no workspace is open.
#[tauri::command]
pub async fn terminal_list(
    state: State<'_, AppState>,
) -> Result<Vec<agent_sandbox::pty::PtySessionInfo>, IpcError> {
    let workspace_id = open_workspace_path(&state).ok().map(|p| workspace_uuid(&p));
    Ok(state.terminals.list(workspace_id).await)
}

/// Returns ANSI/OSC/CSI-stripped plain text only, exactly `{ text }` per
/// the IPC spec — the trust envelope (`ContextBlock`, `trust: untrusted`)
/// is built later, at the point the user's "share output" action attaches
/// this text to a prompt, not here.
///
/// # Errors
///
/// Returns an error if the terminal session does not exist.
#[tauri::command]
pub async fn terminal_capture(
    terminal_id: uuid::Uuid,
    lines: Option<usize>,
    state: State<'_, AppState>,
) -> Result<TerminalCapture, IpcError> {
    let raw = state.terminals.capture(terminal_id, lines).await.map_err(map_pty_err)?;
    let stripped = agent_sandbox::pty::ansi::strip_ansi(&raw);
    Ok(TerminalCapture { text: String::from_utf8_lossy(&stripped).into_owned() })
}

/// # Errors
///
/// This command does not fail synchronously; run failures are reported
/// asynchronously via `agent://app` events. A run that fails before or during
/// streaming still ends with a terminal `run_error`, so the UI never waits on
/// a run that is no longer happening.
#[tauri::command]
pub async fn run_start(
    request: RunRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    // The engine authenticates from the cached session, so renew before the
    // run rather than letting it fail mid-stream with a 401.
    ensure_fresh_session(&state).await?;

    // Runs carry their own workspace root; mirror it into the open-workspace
    // slot so trust lookups and terminals target the same folder the UI shows.
    if let Some(root) = request.workspace_root.clone() {
        let mut guard = state
            .workspace_path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(root);
    }

    let (tx, mut rx) = mpsc::channel::<AgentEvent>(100);
    let run_id = request.run_id;

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app.emit("agent://app", event);
        }
    });

    let rm = state.run_manager.clone();
    let error_tx = tx.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = rm.start_run(request, tx).await {
            // Swallowing this is what leaves the UI spinning forever on a run
            // that never started — a rejected concurrent run, an auth failure,
            // an unreachable gateway.
            let _ = error_tx
                .send(AgentEvent::RunError {
                    run_id,
                    code: error_code(&err),
                    message: err.to_string(),
                })
                .await;
        }
    });

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRunRegisterRequest {
    pub run_id: uuid::Uuid,
    pub conversation_id: uuid::Uuid,
}

/// Registers a foreground run for the desktop AI SDK path (no Rust inference loop).
#[tauri::command]
pub async fn ai_run_register(
    request: AiRunRegisterRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    ensure_fresh_session(&state).await?;
    state.run_manager.register_run(request.conversation_id).await?;
    let _ = app.emit(
        "agent://app",
        AgentEvent::Meta {
            run_id: request.run_id,
            status: "streaming".to_string(),
        },
    );
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRunCompleteRequest {
    pub run_id: uuid::Uuid,
    pub conversation_id: uuid::Uuid,
    pub stop_reason: String,
}

/// Marks an AI SDK run complete and releases the foreground run slot.
#[tauri::command]
pub async fn ai_run_complete(
    request: AiRunCompleteRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    state.run_manager.unregister_run(request.conversation_id).await;
    let _ = app.emit(
        "agent://app",
        AgentEvent::RunComplete {
            run_id: request.run_id,
            stop_reason: request.stop_reason,
        },
    );
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExecuteIpcRequest {
    pub run_id: uuid::Uuid,
    pub conversation_id: uuid::Uuid,
    pub tool_call_id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub mode: agent_types::requests::RunMode,
    pub workspace_root: Option<PathBuf>,
}

/// Executes one local tool with policy, approvals, and live output streaming.
#[tauri::command]
pub async fn tool_execute(
    request: ToolExecuteIpcRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, IpcError> {
    if let Some(root) = request.workspace_root.clone() {
        let mut guard = state
            .workspace_path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(root);
    }

    let cancel = state
        .run_manager
        .cancel_token_for(request.conversation_id)
        .await
        .unwrap_or_else(CancellationToken::new);

    let (tx, mut rx) = mpsc::channel::<AgentEvent>(32);
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app_handle.emit("agent://app", event);
        }
    });

    let result = state.tool_executor.execute(
        ToolExecuteRequest {
            run_id: request.run_id,
            conversation_id: request.conversation_id,
            tool_call_id: request.tool_call_id,
            name: request.name,
            arguments: request.arguments,
            mode: request.mode,
            workspace_root: request.workspace_root,
        },
        &tx,
        &cancel,
    )
    .await;

    Ok(result)
}

/// The error's canonical `EURY_*` code, read from its own serde tag so this
/// cannot drift from [`agent_types::errors::AgentError`].
fn error_code(err: &agent_types::errors::AgentError) -> String {
    serde_json::to_value(err)
        .ok()
        .and_then(|value| value.get("code")?.as_str().map(str::to_string))
        .unwrap_or_else(|| "EURY_AGENT_INTERNAL".to_string())
}

/// # Errors
///
/// Returns an error if no active run exists for the given conversation.
#[tauri::command]
pub async fn run_cancel(
    conversation_id: uuid::Uuid,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    state.run_manager.cancel_run(conversation_id).await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if no pending approval exists for the given tool call.
#[tauri::command]
pub async fn run_approve(
    tool_call_id: String,
    approved: bool,
    scope: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    use agent_core::approval::ApprovalOutcome;

    let outcome = match (approved, scope.as_deref()) {
        (false, _) | (_, Some("denied")) => ApprovalOutcome::Denied,
        (true, Some("allow_session")) => ApprovalOutcome::Session,
        (true, _) => ApprovalOutcome::Once,
    };

    if !state.approvals.resolve(&tool_call_id, outcome) {
        return Err(IpcError::NotFound(format!("No pending approval for {tool_call_id}")));
    }
    Ok(())
}

/// # Errors
///
/// This command does not currently fail; steering is not yet implemented
/// (see Phase 8 gap: it only logs the instruction).
#[tauri::command]
pub async fn run_steer(
    conversation_id: uuid::Uuid,
    instruction: String,
    _state: State<'_, AppState>,
) -> Result<(), IpcError> {
    println!("Steering run {conversation_id} with instruction: {instruction}");
    Ok(())
}

/// # Errors
///
/// This command does not currently fail; a cancelled or errored file dialog
/// resolves to `Ok(None)`.
#[tauri::command]
pub async fn workspace_pick_folder(app: AppHandle) -> Result<Option<String>, IpcError> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_folder();
    Ok(picked.and_then(|p| p.into_path().ok().map(|pb| pb.to_string_lossy().to_string())))
}

/// # Errors
///
/// This command does not currently fail; an unresolvable `path` falls back
/// to the current working directory.
#[tauri::command]
pub async fn workspace_open(
    path: Option<PathBuf>,
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, IpcError> {
    let resolved =
        path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let name = resolved
        .file_name()
        .map_or_else(|| "workspace".to_string(), |n| n.to_string_lossy().to_string());

    let mut guard = state.workspace_path.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(resolved.clone());
    drop(guard);

    let is_trusted = state.trust.is_trusted(&resolved);
    Ok(WorkspaceInfo { path: resolved, name, is_trusted })
}

/// # Errors
///
/// This command does not currently fail.
#[tauri::command]
pub async fn workspace_close(state: State<'_, AppState>) -> Result<(), IpcError> {
    let mut guard = state.workspace_path.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
    Ok(())
}

/// # Errors
///
/// Returns an error if no workspace is currently open.
#[tauri::command]
pub async fn workspace_info(state: State<'_, AppState>) -> Result<WorkspaceInfo, IpcError> {
    let path = state
        .workspace_path
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .ok_or_else(|| IpcError::NotFound("No workspace open".into()))?;
    let name = path
        .file_name()
        .map_or_else(|| "workspace".to_string(), |n| n.to_string_lossy().to_string());
    let is_trusted = state.trust.is_trusted(&path);
    Ok(WorkspaceInfo { path, name, is_trusted })
}

/// # Errors
///
/// This command does not currently fail; it returns an empty list if no
/// workspace is open.
#[tauri::command]
pub async fn workspace_recent(state: State<'_, AppState>) -> Result<Vec<WorkspaceInfo>, IpcError> {
    let current =
        state.workspace_path.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    if let Some(path) = current {
        let name = path
            .file_name()
            .map_or_else(|| "workspace".to_string(), |n| n.to_string_lossy().to_string());
        let is_trusted = state.trust.is_trusted(&path);
        return Ok(vec![WorkspaceInfo { path, name, is_trusted }]);
    }
    Ok(vec![])
}

/// Records the user's answer to the workspace trust prompt. Passing
/// `trust: false` revokes trust, returning the workspace to read-only.
///
/// Trust is keyed by canonical absolute path, applied immediately in memory,
/// and persisted to the encrypted store's `workspaces` table so it survives a
/// restart.
///
/// # Errors
///
/// Returns an error if the trust state cannot be persisted.
#[tauri::command]
pub async fn workspace_trust_set(
    path: PathBuf,
    trust: bool,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    state.trust.set_trust(&path, trust);

    let canonical =
        std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone()).to_string_lossy().to_string();
    let name = path
        .file_name()
        .map_or_else(|| "workspace".to_string(), |n| n.to_string_lossy().to_string());
    let trust_state = if trust { "trusted" } else { "untrusted" };

    state
        .store
        .execute(
            "INSERT INTO workspaces (id, path, name, trust_state)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET
                 trust_state = excluded.trust_state,
                 updated_at = datetime('now')"
                .to_string(),
            vec![
                rusqlite::types::Value::Text(workspace_uuid(&path).to_string()),
                rusqlite::types::Value::Text(canonical),
                rusqlite::types::Value::Text(name),
                rusqlite::types::Value::Text(trust_state.to_string()),
            ],
        )
        .await
        .map_err(|e| IpcError::Internal(format!("failed to persist workspace trust: {e}")))?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEntry {
    /// Path relative to the workspace root, using `/` separators.
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    /// 0 for entries directly under the root.
    pub depth: usize,
}

/// Cap on entries returned by one tree listing, so a huge repository can't
/// stall the UI or balloon the IPC payload.
const MAX_TREE_ENTRIES: usize = 5_000;

/// Lists the workspace's real files for the file-tree UI.
///
/// This is a `Read`-class capability, so it deliberately works in an
/// **untrusted** workspace — the user can inspect a project before deciding to
/// trust it. The traversal root goes through [`agent_sandbox::path::PathGuard`],
/// so `..` segments and symlinks that resolve outside the workspace are
/// refused rather than followed, and `.gitignore`/`.git` are skipped.
///
/// # Errors
///
/// Returns an error if no workspace is open, or if `sub_path` resolves outside
/// the workspace root or isn't a directory.
#[tauri::command]
pub async fn workspace_list_tree(
    sub_path: Option<String>,
    max_depth: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<WorkspaceEntry>, IpcError> {
    let root = open_workspace_path(&state)?;
    let workspace = agent_sandbox::workspace::Workspace {
        id: workspace_uuid(&root),
        root_path: root.clone(),
        trust_state: state.trust.trust_state(&root),
    };

    let relative = sub_path.unwrap_or_else(|| ".".to_string());
    let (dir, canon_root) =
        agent_sandbox::path::PathGuard::resolve_dir(&workspace, std::path::Path::new(&relative))
            .map_err(|e| IpcError::NotAuthorized(e.to_string()))?;

    Ok(collect_workspace_entries(&dir, &canon_root, max_depth))
}

/// Walks `dir` and returns entries relative to `canon_root`.
///
/// Split out from [`workspace_list_tree`] so the traversal rules — gitignore
/// handling, `.git`/`node_modules` pruning, depth, the entry cap, and sort
/// order — are testable without constructing a Tauri `State`.
fn collect_workspace_entries(
    dir: &std::path::Path,
    canon_root: &std::path::Path,
    max_depth: Option<usize>,
) -> Vec<WorkspaceEntry> {
    let mut builder = ignore::WalkBuilder::new(dir);
    builder.hidden(false).git_ignore(true).git_global(false).filter_entry(|e| {
        e.file_name().to_str().is_none_or(|n| n != ".git" && n != "node_modules")
    });
    if let Some(depth) = max_depth {
        builder.max_depth(Some(depth));
    }

    let mut entries = Vec::new();
    for result in builder.build() {
        let Ok(entry) = result else { continue };
        if entry.path() == dir {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(canon_root) else { continue };

        let rel_str = rel.to_string_lossy().replace('\\', "/");
        entries.push(WorkspaceEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            depth: rel.components().count().saturating_sub(1),
            is_dir: entry.file_type().is_some_and(|t| t.is_dir()),
            path: rel_str,
        });

        if entries.len() >= MAX_TREE_ENTRIES {
            break;
        }
    }

    // Directories first, then alphabetical — the ordering a file tree expects.
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.path.cmp(&b.path)));
    entries
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    pub path: String,
    pub content: String,
    pub truncated: bool,
}

/// Largest file body sent to the viewer. Beyond this the content is cut and
/// flagged, rather than streaming megabytes through IPC.
const MAX_FILE_BYTES: usize = 512 * 1024;

/// Reads one workspace file for the viewer.
///
/// Like [`workspace_list_tree`] this is `Read`-class and works in an untrusted
/// workspace, and it goes through `PathGuard` so it cannot escape the root.
///
/// # Errors
///
/// Returns an error if no workspace is open, the path escapes the workspace,
/// the file is missing, or it isn't valid UTF-8 text.
#[tauri::command]
pub async fn workspace_read_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceFile, IpcError> {
    use std::io::Read as _;

    let root = open_workspace_path(&state)?;
    let workspace = agent_sandbox::workspace::Workspace {
        id: workspace_uuid(&root),
        root_path: root,
        trust_state: agent_sandbox::workspace::TrustState::Trusted,
    };

    let mut file =
        agent_sandbox::path::PathGuard::open_read(&workspace, std::path::Path::new(&path))
            .map_err(|e| IpcError::NotAuthorized(e.to_string()))?;

    // Read one byte past the cap so we can tell "exactly at the cap" from
    // "there was more".
    let mut buffer = Vec::new();
    file.by_ref()
        .take(MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut buffer)
        .map_err(|e| IpcError::Internal(format!("failed to read {path}: {e}")))?;

    let truncated = buffer.len() > MAX_FILE_BYTES;
    buffer.truncate(MAX_FILE_BYTES);

    let content = String::from_utf8(buffer)
        .map_err(|_| IpcError::Internal(format!("{path} is not UTF-8 text")))?;

    Ok(WorkspaceFile { path, content, truncated })
}

/// The core presentation settings the Rust side reasons about, plus every
/// other settings field carried opaquely.
///
/// The UI's settings object is much larger than these three fields (profile,
/// model, privacy, permissions, MCP servers, ...) and evolves independently.
/// Flattening the rest into `extra` lets the store round-trip the whole blob
/// without this struct having to track a schema it does not own — and without
/// silently dropping fields on save, which `deny_unknown_fields` would have
/// done.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub accent: String,
    pub density: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Key under which the app settings blob lives in the encrypted store's
/// `settings` table.
const SETTINGS_KEY: &str = "app_settings";

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            accent: "ember".to_string(),
            density: "default".to_string(),
            extra: serde_json::Map::new(),
        }
    }
}

/// Reads app settings from the encrypted store, falling back to defaults when
/// nothing has been saved yet.
///
/// # Errors
///
/// Returns an error if the store cannot be read.
#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, IpcError> {
    let rows = state
        .store
        .query(
            "SELECT value FROM settings WHERE key = ?1".to_string(),
            vec![rusqlite::types::Value::Text(SETTINGS_KEY.to_string())],
        )
        .await
        .map_err(|e| IpcError::Internal(format!("failed to read settings: {e}")))?;

    let Some(rusqlite::types::Value::Text(json)) = rows.first().and_then(|r| r.first()) else {
        return Ok(AppSettings::default());
    };
    // A settings row that no longer parses (e.g. written by a newer build)
    // falls back to defaults rather than failing the launch path.
    Ok(serde_json::from_str(json).unwrap_or_default())
}

/// Persists app settings to the encrypted store.
///
/// # Errors
///
/// Returns an error if the settings cannot be serialized or written.
#[tauri::command]
pub async fn settings_set(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), IpcError> {
    let json = serde_json::to_string(&settings)
        .map_err(|e| IpcError::Internal(format!("failed to serialize settings: {e}")))?;

    state
        .store
        .execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')"
                .to_string(),
            vec![
                rusqlite::types::Value::Text(SETTINGS_KEY.to_string()),
                rusqlite::types::Value::Text(json),
            ],
        )
        .await
        .map_err(|e| IpcError::Internal(format!("failed to persist settings: {e}")))?;

    Ok(())
}

/// # Errors
///
/// This command does not currently fail; window state is not yet persisted
/// (tracked separately).
#[tauri::command]
pub async fn window_state_save() -> Result<(), IpcError> {
    Ok(())
}

/// # Errors
///
/// This command does not currently fail; window state is not yet persisted
/// (tracked separately).
#[tauri::command]
pub async fn window_state_load() -> Result<(), IpcError> {
    Ok(())
}

/// # Errors
///
/// Returns an error if the window cannot be closed.
#[tauri::command]
pub async fn window_close(window: tauri::WebviewWindow) -> Result<(), IpcError> {
    window.close().map_err(|e| IpcError::Internal(e.to_string()))?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the window cannot be minimized.
#[tauri::command]
pub async fn window_minimize(window: tauri::WebviewWindow) -> Result<(), IpcError> {
    window.minimize().map_err(|e| IpcError::Internal(e.to_string()))?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the window cannot be maximized or unmaximized.
#[tauri::command]
pub async fn window_toggle_maximize(window: tauri::WebviewWindow) -> Result<(), IpcError> {
    let is_max = window.is_maximized().unwrap_or(false);
    if is_max {
        window.unmaximize().map_err(|e| IpcError::Internal(e.to_string()))?;
    } else {
        window.maximize().map_err(|e| IpcError::Internal(e.to_string()))?;
    }
    Ok(())
}

/// # Errors
///
/// Returns an error if the window's fullscreen state cannot be changed.
#[tauri::command]
pub async fn window_toggle_fullscreen(window: tauri::WebviewWindow) -> Result<(), IpcError> {
    let is_full = window.is_fullscreen().unwrap_or(false);
    window.set_fullscreen(!is_full).map_err(|e| IpcError::Internal(e.to_string()))?;
    Ok(())
}

/// # Errors
///
/// This command does not currently fail; a query error is treated as not maximized.
#[tauri::command]
pub async fn window_is_maximized(window: tauri::WebviewWindow) -> Result<bool, IpcError> {
    Ok(window.is_maximized().unwrap_or(false))
}

/// # Errors
///
/// This command does not currently fail.
#[tauri::command]
pub async fn app_get_platform() -> Result<String, IpcError> {
    #[cfg(target_os = "macos")]
    return Ok("macos".to_string());
    #[cfg(target_os = "windows")]
    return Ok("windows".to_string());
    #[cfg(target_os = "linux")]
    return Ok("linux".to_string());
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return Ok("unknown".to_string());
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub os_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxInfo {
    pub available: bool,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitsInfo {
    pub max_file_size_mb: u32,
    pub max_terminals: u32,
    pub max_parallel_tools: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub ipc_api_version: u32,
    pub event_spec_version: String,
    pub app_version: String,
    pub build_sha: String,
    pub channel: String,
    pub platform: PlatformInfo,
    pub features: std::collections::HashMap<String, bool>,
    pub sandbox: SandboxInfo,
    pub limits: LimitsInfo,
    pub offline: bool,
    pub air_gapped: bool,
}

/// Maps the sandbox crate's `mechanism` (schema: `seatbelt` | `landlock_seccomp`
/// | `restricted_token_job_broker`) to the simplified `sandbox.kind` enum the
/// IPC spec's `Capabilities.sandbox` exposes to the UI
/// (`seatbelt` | `landlock` | `job_object` | `none`).
fn sandbox_kind(mechanism: Option<&str>) -> &'static str {
    match mechanism {
        Some("seatbelt") => "seatbelt",
        Some("landlock_seccomp") => "landlock",
        Some("restricted_token_job_broker") => "job_object",
        _ => "none",
    }
}

/// Real, per-call capability probing — see [`agent_sandbox::os::probe_capabilities`]
/// for what's actually verified per platform (macOS spawns real `sandbox-exec`
/// probes; Linux/Windows honestly report unimplemented rather than an
/// unverified `true`). The UI calls this once at startup and shows a
/// persistent security banner when `sandbox.available` is false.
///
/// # Errors
///
/// This command does not currently fail.
#[tauri::command]
pub async fn capabilities_get() -> Result<Capabilities, IpcError> {
    let sandbox_caps = agent_sandbox::os::probe_capabilities();

    let mut features = std::collections::HashMap::new();
    features.insert("mcp".to_string(), false);
    features.insert("index".to_string(), false);
    features.insert("subagents".to_string(), false);
    features.insert("sync".to_string(), false);
    features.insert("terminal".to_string(), true);

    Ok(Capabilities {
        ipc_api_version: 1,
        event_spec_version: "1.0.0".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        build_sha: option_env!("EURY_AGENT_BUILD_SHA").unwrap_or("unknown").to_string(),
        channel: option_env!("EURY_AGENT_CHANNEL").unwrap_or("dev").to_string(),
        platform: PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            os_version: sandbox_caps.os_version.clone().unwrap_or_else(|| "unknown".to_string()),
        },
        features,
        sandbox: SandboxInfo {
            // Fail closed: only claim the sandbox is "available" once every
            // probe (filesystem, process, egress, privilege escalation) is
            // independently verified passing, not just the ones we happen to
            // have implemented — this is the exact distinction the mocked
            // `probe_capabilities()` used to erase.
            available: sandbox_caps.privileged_tools_enabled,
            kind: sandbox_kind(sandbox_caps.mechanism.as_deref()).to_string(),
        },
        limits: LimitsInfo { max_file_size_mb: 25, max_terminals: 20, max_parallel_tools: 1 },
        offline: false,
        air_gapped: false,
    })
}

use agent_core::auth::{AuthStore, AuthTokens};

/// Key under which the session tokens live in the encrypted store's `settings`
/// table.
///
/// Tokens live here rather than in the OS keychain on purpose. The keychain
/// binds an item's ACL to the calling binary's code signature, and every
/// rebuild of an ad-hoc-signed app is a different binary — so macOS asks for
/// the login password on every launch, and a cancelled prompt reads as
/// "signed out". The encrypted store is already protected by a keychain-held
/// database key, so this is one guarded door instead of two.
const AUTH_TOKENS_KEY: &str = "auth_tokens";

/// Where the platform's agent API lives. Mirrors the frontend's
/// `getAgentApiUrl()`; both read the same `.env`.
fn agent_api_url() -> String {
    std::env::var("VITE_EURY_AGENT_API_URL")
        .or_else(|_| std::env::var("EURY_AGENT_API_URL"))
        .unwrap_or_else(|_| "http://localhost:3001".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Access tokens live ~15 minutes; refresh a little early so a request that
/// takes a moment to start doesn't race the expiry.
const REFRESH_SKEW_SECONDS: i64 = 60;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshResponse {
    access_token: String,
    refresh_token: String,
    device_id: Option<String>,
    expires_in: Option<i64>,
}

fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Refreshes the session if the access token is expired or about to be.
///
/// Without this the desktop used the token it was handed at sign-in forever:
/// the platform expires it after 15 minutes, so every later call came back
/// `401 EURY_AUTH_UNAUTHORIZED` while the app still believed it was signed in.
/// A refresh the platform rejects (expired, revoked, reused) clears the stored
/// session so the user is sent back to sign-in instead of a dead one.
///
/// # Errors
///
/// Returns [`IpcError::NotAuthorized`] when the platform refuses to renew the
/// session, or [`IpcError::Internal`] if the request or store access fails.
pub async fn ensure_fresh_session(state: &AppState) -> Result<(), IpcError> {
    let Some(tokens) = read_stored_tokens(&state.store).await? else { return Ok(()) };

    let (Some(expires_at), Some(device_id)) = (tokens.expires_at, tokens.device_id.clone()) else {
        // Signed in before this app knew about expiry; nothing to refresh
        // against, so leave it and let the 401 path prompt a fresh sign-in.
        return Ok(());
    };

    if now_seconds() < expires_at - REFRESH_SKEW_SECONDS {
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| IpcError::Internal(e.to_string()))?;

    let response = client
        .post(format!("{}/agent/v1/auth/refresh", agent_api_url()))
        .json(&serde_json::json!({
            "refreshToken": tokens.refresh_token,
            "deviceId": device_id,
        }))
        .send()
        .await
        .map_err(|e| IpcError::Internal(format!("refresh request failed: {e}")))?;

    if !response.status().is_success() {
        // The platform will not renew this session: drop it rather than
        // leaving the user "signed in" against a token nothing accepts.
        clear_stored_session(state).await;
        return Err(IpcError::NotAuthorized("Session expired — sign in again".to_string()));
    }

    let refreshed: RefreshResponse = response
        .json()
        .await
        .map_err(|e| IpcError::Internal(format!("malformed refresh response: {e}")))?;

    let next = AuthTokens {
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token,
        device_id: refreshed.device_id.or(Some(device_id)),
        expires_at: Some(now_seconds() + refreshed.expires_in.unwrap_or(900)),
    };
    write_stored_tokens(state, &next).await?;
    Ok(())
}

async fn write_stored_tokens(state: &AppState, tokens: &AuthTokens) -> Result<(), IpcError> {
    let json = serde_json::to_string(tokens)
        .map_err(|e| IpcError::Internal(format!("failed to serialize tokens: {e}")))?;
    state
        .store
        .execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"
                .to_string(),
            vec![
                rusqlite::types::Value::Text(AUTH_TOKENS_KEY.to_string()),
                rusqlite::types::Value::Text(json),
            ],
        )
        .await
        .map_err(|e| IpcError::Internal(format!("failed to store tokens: {e}")))?;
    AuthStore::set_cached_tokens(tokens.clone());
    Ok(())
}

async fn clear_stored_session(state: &AppState) {
    AuthStore::clear_cached_tokens();
    let _ = state
        .store
        .execute(
            "DELETE FROM settings WHERE key = ?1".to_string(),
            vec![rusqlite::types::Value::Text(AUTH_TOKENS_KEY.to_string())],
        )
        .await;
}

async fn read_stored_tokens(
    store: &agent_store::actor::StoreActorHandle,
) -> Result<Option<AuthTokens>, IpcError> {
    let rows = store
        .query(
            "SELECT value FROM settings WHERE key = ?1".to_string(),
            vec![rusqlite::types::Value::Text(AUTH_TOKENS_KEY.to_string())],
        )
        .await
        .map_err(|e| IpcError::Internal(format!("failed to read tokens: {e}")))?;

    let Some(rusqlite::types::Value::Text(json)) = rows.first().and_then(|r| r.first()) else {
        return Ok(None);
    };
    Ok(serde_json::from_str(json).ok())
}

/// Loads any stored session into the engine's cache at startup, so a signed-in
/// user is still signed in after a restart without any prompt.
pub async fn restore_session(state: &AppState) {
    if let Ok(Some(tokens)) = read_stored_tokens(&state.store).await {
        AuthStore::set_cached_tokens(tokens);
    }
}

/// # Errors
///
/// Returns an error if no session is cached or stored.
#[tauri::command]
pub async fn agent_auth_get_tokens(state: State<'_, AppState>) -> Result<AuthTokens, String> {
    // Renew first, so callers never receive a token that is already dead.
    if let Err(e) = ensure_fresh_session(&state).await {
        return Err(format!("{e:?}"));
    }

    if let Ok(tokens) = AuthStore::get_tokens() {
        return Ok(tokens);
    }

    match read_stored_tokens(&state.store).await {
        Ok(Some(tokens)) => {
            AuthStore::set_cached_tokens(tokens.clone());
            Ok(tokens)
        }
        Ok(None) => Err("No token found".to_string()),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// # Errors
///
/// Returns an error if the tokens cannot be serialized or written to the store.
///
/// `tokens` is taken by value rather than by reference because Tauri's IPC
/// deserializer only implements `CommandArg` for owned types.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn agent_auth_set_tokens(
    tokens: AuthTokens,
    state: State<'_, AppState>,
) -> Result<(), String> {
    write_stored_tokens(&state, &tokens).await.map_err(|e| format!("{e:?}"))
}

/// # Errors
///
/// Returns an error if the stored session cannot be deleted.
#[tauri::command]
pub async fn agent_auth_clear_tokens(state: State<'_, AppState>) -> Result<(), String> {
    clear_stored_session(&state).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::collect_workspace_entries;
    use std::path::PathBuf;

    /// `PathGuard` containment for the traversal *root* is covered by
    /// `agent-tools`' `search_path_guard_test`; these cover the walk rules
    /// layered on top of it.
    #[allow(clippy::disallowed_methods)]
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!("eury-tree-test-{}", uuid::Uuid::now_v7()));
        let _ = std::fs::create_dir_all(root.join("src/nested"));
        let _ = std::fs::create_dir_all(root.join(".git/objects"));
        let _ = std::fs::create_dir_all(root.join("node_modules/left-pad"));
        let _ = std::fs::create_dir_all(root.join("build"));
        let _ = std::fs::write(root.join(".gitignore"), "build/\n*.log\n");
        let _ = std::fs::write(root.join("README.md"), "hi");
        let _ = std::fs::write(root.join("src/main.rs"), "fn main() {}");
        let _ = std::fs::write(root.join("src/nested/deep.rs"), "// deep");
        let _ = std::fs::write(root.join("build/output.bin"), "x");
        let _ = std::fs::write(root.join("debug.log"), "x");
        let _ = std::fs::write(root.join(".git/objects/abc"), "x");
        let _ = std::fs::write(root.join("node_modules/left-pad/index.js"), "x");
        root
    }

    #[allow(clippy::disallowed_methods)]
    fn cleanup(root: &PathBuf) {
        let _ = std::fs::remove_dir_all(root);
    }

    fn paths(entries: &[super::WorkspaceEntry]) -> Vec<String> {
        entries.iter().map(|e| e.path.clone()).collect()
    }

    #[test]
    fn lists_real_files_relative_to_the_workspace_root() {
        let root = fixture();
        let entries = collect_workspace_entries(&root, &root, None);
        let found = paths(&entries);

        assert!(found.contains(&"README.md".to_string()), "got {found:?}");
        assert!(found.contains(&"src/main.rs".to_string()), "got {found:?}");
        assert!(found.contains(&"src/nested/deep.rs".to_string()), "got {found:?}");

        cleanup(&root);
    }

    #[test]
    fn prunes_git_and_node_modules_and_honors_gitignore() {
        let root = fixture();
        let found = paths(&collect_workspace_entries(&root, &root, None));

        // Pruned outright — these dominate a real repo and are never useful
        // in a file tree. Note `.gitignore` itself must survive; only the
        // `.git/` directory is pruned.
        assert!(!found.iter().any(|p| p == ".git" || p.starts_with(".git/")), "got {found:?}");
        assert!(found.contains(&".gitignore".to_string()), "got {found:?}");
        assert!(!found.iter().any(|p| p.starts_with("node_modules")), "got {found:?}");
        // Respecting .gitignore is what keeps build output out of the tree.
        assert!(!found.iter().any(|p| p.starts_with("build")), "got {found:?}");
        assert!(!found.contains(&"debug.log".to_string()), "got {found:?}");

        cleanup(&root);
    }

    #[test]
    fn depth_limit_stops_descending() {
        let root = fixture();
        let found = paths(&collect_workspace_entries(&root, &root, Some(1)));

        assert!(found.contains(&"README.md".to_string()), "got {found:?}");
        assert!(found.contains(&"src".to_string()), "got {found:?}");
        assert!(!found.contains(&"src/main.rs".to_string()), "depth 1 must not descend");

        cleanup(&root);
    }

    #[test]
    fn reports_depth_and_sorts_directories_first() {
        let root = fixture();
        let entries = collect_workspace_entries(&root, &root, None);

        let readme = entries.iter().find(|e| e.path == "README.md");
        assert!(matches!(readme, Some(e) if e.depth == 0 && !e.is_dir), "got {entries:?}");

        let deep = entries.iter().find(|e| e.path == "src/nested/deep.rs");
        assert!(matches!(deep, Some(e) if e.depth == 2), "got {entries:?}");

        // Every directory sorts ahead of every file.
        let first_file = entries.iter().position(|e| !e.is_dir).unwrap_or(0);
        assert!(
            entries.iter().skip(first_file).all(|e| !e.is_dir),
            "directories must all precede files"
        );

        cleanup(&root);
    }
}
