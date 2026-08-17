//! End-to-end exercise of the agent loop against a scripted gateway.
//!
//! Everything below the model is real: the prompt assembler, the tool-call
//! extractor, the policy engine, the tool registry and the filesystem. Only
//! the model's replies are scripted, so these are the behaviours a user
//! actually gets when they ask the agent to do something — not a mock of them.

use agent_core::agent_loop::AgentLoopEngine;
use agent_core::approval::{ApprovalOutcome, ApprovalWaiter};
use agent_core::auth::{AuthStore, AuthTokens};
use agent_core::engine::AgentEngine;
use agent_sandbox::path::PathGuard;
use agent_sandbox::workspace::{TrustState, TrustStore, Workspace};
use agent_types::events::AgentEvent;
use agent_types::requests::{ModelConfig, RunMode, RunRequest};
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::sync::CancellationToken;

/// Serves one scripted NDJSON body per connection, in order.
///
/// The loop opens a fresh request per round, so a two-element script is a
/// two-round conversation: tool call, then the model's answer.
async fn scripted_gateway(script: Vec<String>) -> Result<String, Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}/stream", listener.local_addr()?);

    tokio::spawn(async move {
        for turn in script {

            let Ok((mut socket, _)) = listener.accept().await else { return };

            let mut scratch = vec![0u8; 16384];
            let _ = socket.read(&mut scratch).await;
            let mut body = String::new();
            for line in turn.lines() {
                let text = serde_json::to_string(&format!("{line}\n")).unwrap_or_default();
                let _ = writeln!(body, "{{\"type\":\"delta\",\"text\":{text}}}");
            }
            let _ = writeln!(body, "{{\"type\":\"done\",\"finish_reason\":\"stop\"}}");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;

        }
    });

    Ok(url)
}

fn tool_call(name: &str, arguments: &serde_json::Value) -> String {
    format!(
        "```tool_call\n{}\n```",
        serde_json::json!({ "name": name, "arguments": arguments })
    )
}

struct Harness {
    workspace: PathBuf,
    events: Vec<AgentEvent>,
}

impl Harness {
    fn text(&self) -> String {
        self.events
            .iter()
            .filter_map(|e| match e {
                AgentEvent::TextDelta { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn tools_started(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|e| match e {
                AgentEvent::ToolStart { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn tool_results(&self) -> Vec<serde_json::Value> {
        self.events
            .iter()
            .filter_map(|e| match e {
                AgentEvent::ToolEnd { result, .. } => Some(result.clone()),
                _ => None,
            })
            .collect()
    }

    fn approvals_requested(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|e| match e {
                AgentEvent::ApprovalRequired { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn completed(&self) -> bool {
        self.events.iter().any(|e| matches!(e, AgentEvent::RunComplete { .. }))
    }

    /// Reads workspace content the way everything else does — through
    /// `PathGuard` — rather than reaching around the boundary.
    fn read(&self, relative: &str) -> Option<String> {
        use std::io::Read as _;
        let workspace = Workspace {
            id: uuid::Uuid::now_v7(),
            root_path: self.workspace.clone(),
            trust_state: TrustState::Trusted,
        };
        let mut file = PathGuard::open_read(&workspace, Path::new(relative)).ok()?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).ok()?;
        Some(buffer)
    }
}

/// How the harness answers an approval prompt, standing in for the user.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Approvals {
    Allow,
    AllowSession,
    Deny,
}

/// Runs `script` as the model's replies and returns everything the UI saw.
async fn run_with(
    script: Vec<String>,
    mode: RunMode,
    trusted: bool,
    approvals: Approvals,
) -> Result<Harness, Box<dyn Error>> {
    // Cache-only: never touches the OS keychain.
    AuthStore::set_cached_tokens(AuthTokens {
        access_token: "test-token".into(),
        refresh_token: "test-refresh".into(),
        device_id: None,
        expires_at: None,
    });

    let workspace = tempdir()?;

    let trust = Arc::new(TrustStore::new());
    if trusted {
        trust.set_trust(&workspace, true);
    }

    let url = scripted_gateway(script).await?;
    let waiter = Arc::new(ApprovalWaiter::new());
    let engine = AgentLoopEngine::with_gateway_url(waiter.clone(), trust, &url)?;

    let request = RunRequest {
        run_id: uuid::Uuid::now_v7(),
        conversation_id: uuid::Uuid::now_v7(),
        mode,
        prompt: "do the thing".into(),
        history: vec![],
        attachments: vec![],
        workspace_id: None,
        workspace_root: Some(workspace.clone()),
        model: ModelConfig {
            provider: "OpenAI".into(),
            model: "gpt-5.6".into(),
            temperature: 0.0,
            max_tokens: None,
        },
        plan_context: None,
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);

    // Answer approval prompts as the user would. Without someone answering,
    // the run waits forever — which is what the desktop does too if the
    // approval card never gets a click.
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<AgentEvent>(256);
    let answering = tokio::spawn(async move {
        let mut seen = Vec::new();
        while let Some(event) = rx.recv().await {
            if let AgentEvent::ApprovalRequired { tool_call_id, .. } = &event {
                let outcome = match approvals {
                    Approvals::Allow => ApprovalOutcome::Once,
                    Approvals::AllowSession => ApprovalOutcome::Session,
                    Approvals::Deny => ApprovalOutcome::Denied,
                };
                waiter.resolve(tool_call_id, outcome);
            }
            seen.push(event.clone());
            let _ = event_tx.send(event).await;
        }
        seen
    });

    let _ = engine.run_stream(request, tx, CancellationToken::new()).await;
    let events = answering.await.unwrap_or_default();
    while event_rx.try_recv().is_ok() {}

    Ok(Harness { workspace, events })
}

async fn run(script: Vec<String>, mode: RunMode, trusted: bool) -> Result<Harness, Box<dyn Error>> {
    run_with(script, mode, trusted, Approvals::Allow).await
}

/// Fixture plumbing: this crate's own temp directory, never workspace content
/// supplied by an agent or tool, so `PathGuard` does not apply here.
#[allow(clippy::disallowed_methods)]
fn tempdir() -> Result<PathBuf, Box<dyn Error>> {
    let dir = std::env::temp_dir().join(format!("eury-e2e-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir)?;
    Ok(dir.canonicalize()?)
}

#[allow(clippy::disallowed_methods)]
fn cleanup(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}

#[tokio::test]
async fn creates_a_file_when_asked_to() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            format!(
                "I'll create it.\n{}",
                tool_call(
                    "write_file",
                    &serde_json::json!({"path": "hello.txt", "content": "Hello world\n"}),
                )
            ),
            "Created `hello.txt` with the text you asked for.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    assert_eq!(h.tools_started(), vec!["write_file"], "events: {:?}", h.events);
    assert_eq!(h.read("hello.txt").as_deref(), Some("Hello world\n"));
    assert!(h.completed(), "the run must end with a terminal event");
    assert!(h.text().contains("Created `hello.txt`"));
    cleanup(&h.workspace);
    Ok(())
}

#[tokio::test]
async fn reads_then_edits_across_rounds() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("write_file", &serde_json::json!({"path": "a.txt", "content": "old value\n"})),
            tool_call("read_file", &serde_json::json!({"path": "a.txt"})),
            tool_call(
                "edit_file",
                // The schema is camelCase; snake_case is covered separately.
                &serde_json::json!({"path": "a.txt", "oldString": "old", "newString": "new"}),
            ),
            "Updated `a.txt`.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    assert_eq!(h.tools_started(), vec!["write_file", "read_file", "edit_file"]);
    assert_eq!(h.read("a.txt").as_deref(), Some("new value\n"));
    cleanup(&h.workspace);
    Ok(())
}

#[tokio::test]
async fn refuses_to_write_outside_the_workspace() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call(
                "write_file",
                &serde_json::json!({"path": "../escaped.txt", "content": "nope"}),
            ),
            "I could not write outside the project.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    let escaped = h.workspace.parent().map(|p| p.join("escaped.txt"));
    assert!(
        escaped.is_none_or(|p| !p.exists()),
        "a path escape must not land on disk: {:?}",
        h.events
    );
    cleanup(&h.workspace);
    Ok(())
}

#[tokio::test]
async fn an_untrusted_workspace_stays_read_only() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("write_file", &serde_json::json!({"path": "x.txt", "content": "nope"})),
            "This project is untrusted, so I did not change it.".to_string(),
        ],
        RunMode::Agent,
        false,
    )
    .await?;

    assert_eq!(h.read("x.txt"), None, "untrusted workspaces must not be written to");

    // The refusal has to say why and how to fix it, and the run must finish
    // normally rather than dying with a bare "denied by policy".
    let refusal = h.tool_results().iter().map(ToString::to_string).collect::<String>();
    assert!(refusal.contains("not trusted"), "got: {refusal}");
    assert!(refusal.contains("Trust the project"), "got: {refusal}");
    assert!(h.completed(), "a refusal must not abort the run");
    assert!(
        !h.events.iter().any(|e| matches!(e, AgentEvent::RunError { .. })),
        "a policy refusal is not a run failure",
    );
    cleanup(&h.workspace);
    Ok(())
}

#[tokio::test]
async fn an_unknown_tool_is_reported_not_executed() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("delete_everything", &serde_json::json!({"path": "/"})),
            "That tool does not exist.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    assert!(h.tools_started().is_empty(), "an unregistered name must never run");
    cleanup(&h.workspace);
    Ok(())
}

#[tokio::test]
async fn tool_output_is_reported_back_to_the_user() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("write_file", &serde_json::json!({"path": "b.txt", "content": "hi"})),
            tool_call("list_dir", &serde_json::json!({"path": "."})),
            "The project contains b.txt.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    let listed = h.tool_results().iter().any(|r| r.to_string().contains("b.txt"));
    assert!(listed, "list_dir must report the file it just wrote: {:?}", h.tool_results());
    cleanup(&h.workspace);
    Ok(())
}

/// A model that guesses `snake_case` argument names (a very common habit) must
/// get a visible error, not a silent no-op that looks like success.
#[tokio::test]
async fn wrong_argument_names_surface_as_an_error() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("write_file", &serde_json::json!({"path": "c.txt", "content": "old\n"})),
            tool_call(
                "edit_file",
                &serde_json::json!({"path": "c.txt", "old_string": "old", "new_string": "new"}),
            ),
            "I could not apply that edit.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    assert_eq!(h.read("c.txt").as_deref(), Some("old\n"), "the file must be untouched");
    let edit_result = h.tool_results().last().cloned().unwrap_or_default();
    assert!(
        edit_result.to_string().to_lowercase().contains("error")
            || edit_result.to_string().to_lowercase().contains("invalid"),
        "a rejected call must report why, got: {edit_result}",
    );
    Ok(())
}

/// Running a command is `Execute` class, which the standard policy gates
/// behind the user's approval — the desktop shows an approval card for it.
#[tokio::test]
async fn a_command_asks_before_running() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("run_command", &serde_json::json!({"command": "echo hello-from-eury"})),
            "Ran it.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    assert_eq!(h.approvals_requested(), vec!["run_command"]);
    let output = h.tool_results().iter().map(ToString::to_string).collect::<String>();
    assert!(output.contains("hello-from-eury"), "approved command must run: {output}");
    Ok(())
}

/// After a diagnostic command runs, the model must answer with the result — not
/// claim no task was given.
#[tokio::test]
async fn answers_after_a_version_command_runs() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call("run_command", &serde_json::json!({"command": "pnpm --version"})),
            "Your pnpm version is 11.5.1.".to_string(),
        ],
        RunMode::Ask,
        true,
    )
    .await?;

    assert!(h.text().contains("11.5.1"));
    assert!(
        !h.text().to_lowercase().contains("no coding task"),
        "must not dismiss the user's question after tools ran: {}",
        h.text()
    );
    Ok(())
}

/// Denying that approval must stop the command and tell the model why.
#[tokio::test]
async fn denying_an_approval_stops_the_command() -> Result<(), Box<dyn Error>> {
    let h = run_with(
        vec![
            tool_call("run_command", &serde_json::json!({"command": "echo should-not-run"})),
            "You denied that, so I stopped.".to_string(),
        ],
        RunMode::Agent,
        true,
        Approvals::Deny,
    )
    .await?;

    assert_eq!(h.approvals_requested(), vec!["run_command"]);
    assert!(h.tools_started().is_empty(), "a denied command must never start");
    assert!(h.completed());
    Ok(())
}

/// "Allow for this session" must auto-approve later shell commands too.
#[tokio::test]
async fn session_approval_covers_later_commands() -> Result<(), Box<dyn Error>> {
    let h = run_with(
        vec![
            tool_call("run_command", &serde_json::json!({"command": "echo one"})),
            tool_call("run_command", &serde_json::json!({"command": "echo two"})),
            "Both ran.".to_string(),
        ],
        RunMode::Agent,
        true,
        Approvals::AllowSession,
    )
    .await?;

    assert_eq!(h.approvals_requested(), vec!["run_command"]);
    assert_eq!(h.tools_started(), vec!["run_command", "run_command"]);
    Ok(())
}

/// Editing a file that does not exist must say which tool would work, so the
/// model corrects course instead of retrying.
#[tokio::test]
async fn editing_a_missing_file_points_at_write_file() -> Result<(), Box<dyn Error>> {
    let h = run(
        vec![
            tool_call(
                "edit_file",
                &serde_json::json!({"path": "hello.txt", "oldString": "a", "newString": "b"}),
            ),
            "That file does not exist yet.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    let error = h.tool_results().iter().map(ToString::to_string).collect::<String>();
    assert!(error.contains("does not exist"), "got: {error}");
    assert!(error.contains("write_file"), "the error must name the tool that works: {error}");
    cleanup(&h.workspace);
    Ok(())
}

/// A model that repeats the identical failing call — the observed loop — gets
/// cut off rather than burning every remaining round on it.
#[tokio::test]
async fn an_identical_failing_call_is_not_run_twice() -> Result<(), Box<dyn Error>> {
    let failing = tool_call(
        "edit_file",
        &serde_json::json!({"path": "nope.txt", "oldString": "a", "newString": "b"}),
    );

    let h = run(
        vec![
            failing.clone(),
            failing.clone(),
            failing.clone(),
            "I cannot edit a file that does not exist.".to_string(),
        ],
        RunMode::Agent,
        true,
    )
    .await?;

    // The tool ran once; the repeats were answered from the guard.
    assert_eq!(
        h.tools_started().iter().filter(|n| *n == "edit_file").count(),
        1,
        "a repeated failing call must not execute again: {:?}",
        h.tools_started(),
    );
    assert!(h.completed());
    cleanup(&h.workspace);
    Ok(())
}
