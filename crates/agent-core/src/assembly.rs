//! System-prompt assembly.
//!
//! # Why this exists
//!
//! The agent loop recovers tool calls from ` ```tool_call ` fences in the
//! assistant's own text ([`crate::tool_calls`]). That only works if the model
//! is *told* which tools exist, what their arguments look like, and how to
//! emit a call — nothing else in the stack tells it. Without this prompt the
//! model in Code mode behaves like a plain chat window: it answers "which
//! folder do you mean?" while sitting inside an open workspace it could have
//! listed itself.

use agent_sandbox::workspace::{TrustState, Workspace};
use agent_types::capabilities::ToolDefinition;
use agent_types::requests::{RunMode, RunRequest};
use std::fmt::Write as _;

/// Tools that mutate files on disk.
const FILE_WRITE_TOOLS: [&str; 2] = ["write_file", "edit_file"];
/// Tools that run shell commands.
const EXECUTE_TOOLS: [&str; 1] = ["run_command"];

pub struct PromptAssembler;

impl PromptAssembler {
    /// Tool definitions for a run, with write/execute tools removed when the
    /// workspace is not trusted so the prompt does not advertise tools policy
    /// will refuse anyway.
    #[must_use]
    pub fn tools_for_run(
        tools: &agent_tools::registry::ToolRegistry,
        mode: &RunMode,
        workspace: Option<&Workspace>,
    ) -> Vec<ToolDefinition> {
        let mut defs = tools.get_definitions_for_mode(mode);
        if workspace.is_some_and(|ws| !ws.is_trusted()) {
            defs.retain(|tool| {
                !FILE_WRITE_TOOLS.contains(&tool.name.as_str())
                    && !EXECUTE_TOOLS.contains(&tool.name.as_str())
            });
        }
        defs
    }

    /// Builds the system prompt for a run: who the agent is, what workspace it
    /// is standing in, which tools it may call, and the exact call syntax the
    /// extractor understands.
    #[must_use]
    pub fn system_prompt(
        request: &RunRequest,
        workspace: Option<&Workspace>,
        tools: &[ToolDefinition],
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str(
            "You are Eury Agent, a coding assistant running on the user's own machine \
             inside the Eury desktop app.\n",
        );
        let _ = writeln!(prompt, "Current mode: {}.", Self::mode_label(&request.mode));
        prompt.push('\n');

        prompt.push_str(&Self::workspace_section(workspace));

        if tools.is_empty() {
            prompt.push_str(
                "## Tools\n\nYou have no workspace tools in this mode: you cannot read, write \
                 or run anything in the user's project, and must not claim to have inspected a \
                 file. Any other tools the platform provides you still apply.\n\n",
            );
            if workspace.is_some() {
                prompt.push_str(&Self::mode_limit_note(&request.mode));
            }
        } else {
            prompt.push_str(&Self::tools_section(tools));
            prompt.push_str(&Self::protocol_section());
            let has_file_writes =
                tools.iter().any(|t| FILE_WRITE_TOOLS.contains(&t.name.as_str()));
            let has_execute = tools.iter().any(|t| EXECUTE_TOOLS.contains(&t.name.as_str()));
            if !has_file_writes {
                prompt.push_str(&Self::file_write_limit_note(&request.mode));
            }
            if !has_execute {
                if workspace.is_some_and(|ws| !ws.is_trusted()) {
                    prompt.push_str(&Self::untrusted_limit_note());
                } else {
                    prompt.push_str(&Self::execute_limit_note(&request.mode));
                }
            }
        }

        prompt.push_str(&Self::behavior_section(workspace, !tools.is_empty()));

        prompt
    }

    /// What to say when this mode cannot change files on disk.
    fn file_write_limit_note(mode: &RunMode) -> String {
        format!(
            "## What this mode cannot do\n\nYou cannot create, edit or delete files in {} mode. \
             If the user asks for a file change, say plainly that this mode is read-only for \
             writes and that switching the composer to Agent mode will let you make the change — \
             then stop. Do not print the file you would have written as a substitute for writing \
             it.\n\n",
            Self::mode_name(mode),
        )
    }

    /// What to say when this project is not trusted yet.
    fn untrusted_limit_note() -> String {
        "## What this project cannot do yet\n\nThis project is not trusted, so shell \
         commands and file writes are blocked. If the user asks you to run a command \
         or change a file, tell them to click **Trust project** in the app — do not \
         paste shell commands for them to run manually.\n\n"
            .to_string()
    }

    /// What to say when this mode cannot run shell commands.
    fn execute_limit_note(mode: &RunMode) -> String {
        format!(
            "## What this mode cannot do\n\nYou cannot run shell commands in {} mode. If the \
             user asks to check a version, environment variable, or other machine state, say \
             that switching the composer to Agent mode will let you run the command — then \
             stop.\n\n",
            Self::mode_name(mode),
        )
    }

    /// Legacy combined note for modes with no workspace tools at all.
    fn mode_limit_note(mode: &RunMode) -> String {
        format!(
            "{}{}",
            Self::file_write_limit_note(mode),
            Self::execute_limit_note(mode)
        )
    }

    fn mode_name(mode: &RunMode) -> &'static str {
        match mode {
            RunMode::Chat => "Chat",
            RunMode::Ask => "Ask",
            RunMode::Plan => "Plan",
            RunMode::Agent => "Agent",
            RunMode::Build => "Build",
        }
    }

    fn mode_label(mode: &RunMode) -> &'static str {
        match mode {
            RunMode::Chat => "chat (conversation only)",
            RunMode::Ask => "ask (answer questions about the code, do not modify it)",
            RunMode::Plan => "plan (produce a step-by-step plan before any edit)",
            RunMode::Agent => "agent (inspect and change the workspace to complete the task)",
            RunMode::Build => "build (execute an approved plan)",
        }
    }

    fn workspace_section(workspace: Option<&Workspace>) -> String {
        let Some(workspace) = workspace else {
            return "## Workspace\n\nNo project folder is open, so you cannot read, write or \
                    run anything on this machine. If the user asks about their files, say that \
                    no project is open and ask them to open a project folder in the app.\n\n"
                .to_string();
        };

        let mut section = String::from("## Workspace\n\n");
        let _ = writeln!(section, "Root: {}", workspace.root_path.display());
        match workspace.trust_state {
            TrustState::Trusted => section.push_str(
                "Trust: trusted — you may read files, write files and run commands. \
                 Writes and commands may still pause for the user's approval.\n",
            ),
            TrustState::Untrusted => section.push_str(
                "Trust: UNTRUSTED — read-only. Reads are allowed; writes and commands are \
                 blocked until the user trusts this project. Do not promise to change files.\n",
            ),
        }
        section.push_str(
            "All tool paths are relative to this root. Paths outside it are rejected.\n\n",
        );
        section
    }

    fn tools_section(tools: &[ToolDefinition]) -> String {
        let mut section = String::from("## Tools\n\nYou can call these tools:\n\n");
        let mut ordered: Vec<&ToolDefinition> = tools.iter().collect();
        ordered.sort_by_key(|tool| {
            if tool.name == "run_command" {
                0
            } else if tool.name == "list_dir" {
                1
            } else {
                2
            }
        });
        for tool in ordered {
            let _ = writeln!(section, "### {}", tool.name);
            let _ = writeln!(section, "{}", tool.description);
            let schema = serde_json::to_string(&tool.parameters_schema)
                .unwrap_or_else(|_| "{}".to_string());
            let _ = writeln!(section, "Arguments schema: {schema}\n");
        }
        section
    }

    fn protocol_section() -> String {
        // Must stay in lockstep with `crate::tool_calls::extract_tool_calls`.
        "## Calling a tool\n\nTo call a tool, emit a fenced block exactly like this, on its \
         own lines:\n\n\
         ```tool_call\n\
         {\"name\": \"list_dir\", \"arguments\": {\"path\": \".\"}}\n\
         ```\n\n\
         Shell diagnostics (versions, env checks):\n\n\
         ```tool_call\n\
         {\"name\": \"run_command\", \"arguments\": {\"command\": \"node --version\"}}\n\
         ```\n\n\
         Rules:\n\
         - The fence language must be `tool_call`. A ```json fence, a bare object, or prose \
         describing the call does not run anything.\n\
         - `name` must be exactly one of the tool names above, and `arguments` must match its \
         schema. Unknown names are dropped.\n\
         - Shell commands (node --version, npm, find, etc.) always use `run_command` with a \
         `command` argument — never `read_file`.\n\
         - One call per block. To make several calls in a turn, emit several blocks.\n\
         - Emit the call and stop. Results come back in the next turn as `[tool_result ...]` \
         blocks; then answer the user's question using those results in plain language.\n\
         - Never show a `tool_call` fence as an example or illustration — every one you emit \
         is executed.\n\n"
            .to_string()
    }

    fn behavior_section(workspace: Option<&Workspace>, has_tools: bool) -> String {
        let has_workspace = workspace.is_some();
        let mut section = String::from("## How to work\n\n");
        if has_workspace && has_tools {
            section.push_str(
                "                 - Find things out yourself. If the user asks what is in a folder, call \
                 `list_dir` — do not ask them to paste a path, upload files or take a \
                 screenshot. Do not use `find`/`pwd` pipelines when `list_dir` or \
                 `read_file` can answer the question.\n\
                 - When the user asks what version of a tool is installed, what their \
                 environment is, or anything else you could answer by running a command, call \
                 `run_command` (for example `node --version` or `pnpm --version`) — never tell \
                 them to run it themselves while `run_command` is available.\n\
                 - Never say you only have \"file\" or \"read-only\" tools when `run_command` \
                 is listed above — use it for shell diagnostics.\n\
                 - Never say workspace, terminal, or file tools are unavailable when they appear \
                 in the Tools section — emit tool_call fences instead.\n\
                 - After `[tool_result]` blocks arrive, answer the user directly — report version \
                 numbers, file contents, or command output. Never say no task was given.\n\
                 - Before starting a dev server (`npm run dev`, `pnpm dev`, etc.), run the package \
                 manager install command if `node_modules` is missing. If a dev command fails with \
                 unresolved imports or missing packages, run `npm install` or `pnpm install`, then \
                 retry.\n\
                 - When a command fails (non-zero exit code or errors in stderr), diagnose the \
                 output and emit another tool call to fix it — do not tell the user to run commands \
                 manually or give up after one failure.\n\
                 - For action requests (run, start, create, install, build), keep emitting tool \
                 calls until the task is actually done — one `list_dir` or `read_file` is never \
                 enough; follow through with install and run commands as needed.\n\
                 - Work like an autonomous coding agent: understand → act → verify → act again \
                 until the user's request is fully complete. Never stop after planning text.\n\
                 - When an action task is truly finished, end your final message with \
                 `TASK_COMPLETE` on its own line, then summarize what you did.\n\
                 - To create a new file call `write_file`; it creates the file if it does not \
                 exist. `edit_file` only replaces text inside a file that already exists.\n\
                 - Never say you created, changed or ran something before the tool result comes \
                 back. Emit the call, wait, then report what the result actually says — including \
                 a refusal.\n\
                 - Read before you edit, and re-read after editing if the result matters.\n\
                 - Verify with a command when one is available (tests, build, linter).\n\
                 - Only describe file contents you actually read this run.\n\
                 - The latest user message is always the active task. Earlier turns are \
                 background only — do not repeat their shell diagnostics or commands unless \
                 the latest message asks for them again.\n",
            );
        } else {
            section.push_str(
                "- Answer from the conversation. Be explicit that you cannot see the user's \
                 files right now.\n",
            );
        }
        section.push_str(
            "- Be concise and concrete. Report what you did and what happened, including \
             failures.\n",
        );
        section
    }

    /// Marks third-party content so the model treats it as data, never as
    /// instructions.
    #[must_use]
    pub fn mark_untrusted(data: &str, source_name: &str) -> String {
        format!(
            "{{{{UNTRUSTED_CONTENT_START source=\"{source_name}\"}}}}\n{data}\n{{{{UNTRUSTED_CONTENT_END}}}}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_types::requests::{ModelConfig, RunMode, RunRequest};
    use std::path::PathBuf;
    use uuid::Uuid;

    fn request(mode: RunMode) -> RunRequest {
        RunRequest {
            run_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            mode,
            prompt: "list the files".to_string(),
            history: vec![],
            attachments: vec![],
            workspace_id: None,
            workspace_root: Some(PathBuf::from("/tmp/project")),
            model: ModelConfig {
                provider: "OpenAI".into(),
                model: "gpt-5.6".into(),
                temperature: 0.7,
                max_tokens: None,
            },
            plan_context: None,
        }
    }

    fn workspace(trust: TrustState) -> Workspace {
        Workspace {
            id: Uuid::new_v4(),
            root_path: PathBuf::from("/tmp/project"),
            trust_state: trust,
        }
    }

    fn tool() -> ToolDefinition {
        ToolDefinition {
            name: "list_dir".to_string(),
            description: "Lists entries in a directory.".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": { "path": { "type": "string" } }
            }),
        }
    }

    /// The real registry, not a stand-in: if a tool the user asks for is
    /// missing from the prompt, the model refuses the task and blames its
    /// tools ("`edit_file` only modifies existing files").
    #[test]
    fn lists_every_agent_mode_tool_from_the_real_registry() {
        let tools = agent_tools::registry_factory::default_registry()
            .get_definitions_for_mode(&RunMode::Agent);
        let prompt = PromptAssembler::system_prompt(
            &request(RunMode::Agent),
            Some(&workspace(TrustState::Trusted)),
            &tools,
        );

        for expected in ["write_file", "edit_file", "read_file", "list_dir", "run_command"] {
            assert!(prompt.contains(expected), "{expected} missing from the system prompt");
        }
    }

    /// A read-only mode must explain itself in terms the user can act on,
    /// instead of reporting that a tool "isn't available in this session".
    #[test]
    fn a_read_only_mode_points_at_agent_mode() {
        let tools = agent_tools::registry_factory::default_registry()
            .get_definitions_for_mode(&RunMode::Ask);
        let prompt = PromptAssembler::system_prompt(
            &request(RunMode::Ask),
            Some(&workspace(TrustState::Trusted)),
            &tools,
        );

        assert!(prompt.contains("read_file"), "Ask mode still reads");
        assert!(prompt.contains("### run_command"), "Ask mode may run diagnostics");
        assert!(!prompt.contains("### write_file"), "Ask mode must not offer writes");
        assert!(prompt.contains("switching the composer to Agent mode"));
        assert!(!prompt.contains("cannot run shell commands"), "Ask may run commands");
        assert!(prompt.contains("Do not print the file you would have written"));
    }

    #[test]
    fn agent_mode_carries_no_read_only_note() {
        let tools = agent_tools::registry_factory::default_registry()
            .get_definitions_for_mode(&RunMode::Agent);
        let prompt = PromptAssembler::system_prompt(
            &request(RunMode::Agent),
            Some(&workspace(TrustState::Trusted)),
            &tools,
        );

        assert!(prompt.contains("### write_file"));
        assert!(!prompt.contains("What this mode cannot do"));
    }

    #[test]
    fn names_the_workspace_and_its_tools() {
        let prompt = PromptAssembler::system_prompt(
            &request(RunMode::Agent),
            Some(&workspace(TrustState::Trusted)),
            &[tool()],
        );

        assert!(prompt.contains("/tmp/project"));
        assert!(prompt.contains("list_dir"));
        assert!(prompt.contains("```tool_call"));
        assert!(prompt.contains("do not ask them to paste a path"));
    }

    #[test]
    fn flags_an_untrusted_workspace_as_read_only() {
        let prompt = PromptAssembler::system_prompt(
            &request(RunMode::Agent),
            Some(&workspace(TrustState::Untrusted)),
            &[tool()],
        );

        assert!(prompt.contains("UNTRUSTED"));
        assert!(prompt.contains("read-only"));
    }

    #[test]
    fn untrusted_workspace_drops_execute_tools_from_the_prompt() {
        let registry = agent_tools::registry_factory::default_registry();
        let tools = PromptAssembler::tools_for_run(
            &registry,
            &RunMode::Agent,
            Some(&workspace(TrustState::Untrusted)),
        );
        let prompt = PromptAssembler::system_prompt(
            &request(RunMode::Agent),
            Some(&workspace(TrustState::Untrusted)),
            &tools,
        );

        assert!(!tools.iter().any(|t| t.name == "run_command"));
        assert!(!prompt.contains("### run_command"));
        assert!(prompt.contains("Trust project"));
        assert!(prompt.contains("do not paste shell commands"));
    }

    #[test]
    fn says_so_when_no_project_is_open() {
        let prompt = PromptAssembler::system_prompt(&request(RunMode::Chat), None, &[]);

        assert!(prompt.contains("No project folder is open"));
        assert!(!prompt.contains("```tool_call"));
    }
}
