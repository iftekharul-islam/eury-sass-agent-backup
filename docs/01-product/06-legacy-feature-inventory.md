# Deprecated-App Feature Inventory

Spec-Version: 1.0.0

This is the authoritative disposition record for meaningful behavior implemented
in `code-old/`. It is evidence, not an implementation dependency: new Agent code
MUST NOT import, execute, or copy security-sensitive behavior from the deprecated
Python application.

**Coverage: 100%** of feature-bearing source areas classified as of 2026-08-16.
Pure icon, color, font, and layout utilities are represented by their owning UI
feature rather than repeated line by line.

## Disposition vocabulary

| Disposition | Meaning |
|---|---|
| preserve | Product behavior remains with equivalent semantics |
| improve | User value remains, but UX, reliability, or controls are strengthened |
| replace | Need remains, but the old architecture/implementation is discarded |
| drop | Behavior is intentionally excluded and has no replacement |

## Inventory

| ID | Deprecated behavior | Disposition | Target | Phase | Evidence | Rationale |
|---|---|---|---|---|---|---|
| L-001 | Persistent Home/Code areas | preserve | F-001 | 3 | `code-old/src/eury_code/state.py`, `components/segmented_control.py` | Core product separation remains |
| L-002 | Project-scoped conversation lists | improve | F-008 | 3–9 | `code-old/README.md`, `components/sidebar.py` | Keep grouping; use encrypted normalized storage and progressive history |
| L-003 | Rename, duplicate, pin, clear, delete session | improve | F-008 | 3–9 | `code-old/components/sidebar.py` | Preserve actions with confirmation, audit, and transactional persistence |
| L-004 | Collapsible sidebar | preserve | F-003 | 3 | `code-old/components/sidebar.py` | Proven space-saving desktop behavior |
| L-005 | Light/dark theme, accents, font preference | improve | F-002, F-007 | 3 | `code-old/theme.py`, `components/settings_dialog.py` | Keep personalization through semantic tokens and accessible defaults |
| L-006 | Native About, Update, and sidebar menu actions | preserve | F-005 | 3 | `code-old/src/eury_code/app.py`, `components/about_dialog.py` | Native discoverability remains |
| L-007 | Searchable categorized settings dialog | improve | F-007 | 3 | `code-old/components/settings_dialog.py` | Rebuild as the documented modal with policy source visibility |
| L-008 | Home cloud conversations | replace | F-020, F-070 | 8, 23 | `code-old/conversation_sync.py`, `api_client.py` | Agent-owned sync contracts replace legacy `/code` coupling |
| L-009 | Cloud project create/select and project metadata | replace | F-071 | 23 | `code-old/components/projects_dialog.py`, `api_client.py` | Keep jobs through Agent-owned project references |
| L-010 | Profile, organization, billing, notification settings | improve | F-007, F-073, F-074 | 3, 10, 24 | `code-old/components/settings_dialog.py` | Preserve surfaces with stable entitlements and self-contained Agent APIs |
| L-011 | Browser PKCE login gate | replace | F-010 | 10 | `code-old/auth_flow.py`, `components/auth_gate.py` | Use Agent-owned device auth rather than legacy IDE routes |
| L-012 | Plaintext local auth session | replace | F-011 | 10 | `code-old/auth_store.py` | Secrets move to OS keychain with rotation and revoke |
| L-013 | Model catalog and picker | improve | F-022 | 11 | `code-old/model_catalog.py`, `components/model_picker.py` | Add capability, policy, cost, and disabled-reason metadata |
| L-014 | Streaming assistant response | preserve | F-020 | 8 | `code-old/workers.py`, `components/main_content.py` | Core responsiveness requirement |
| L-015 | Markdown prose and code-block rendering | improve | F-020 | 8 | `code-old/components/message_markdown.py`, `markdown_render.py` | Preserve readable output with strict sanitization and streaming boundaries |
| L-016 | Copy, show-more, and message controls | improve | F-034 | 8 | `code-old/components/main_content.py`, `message_markdown.py` | Use consistent accessible turn actions |
| L-017 | File-path attachments in composer | improve | F-035 | 8 | `code-old/components/chat_input.py`, `state.py` | Generalize to typed references with encrypted attachment lifecycle |
| L-018 | Thinking and tool-activity timeline | improve | F-028 | 8 | `code-old/components/tool_activity.py`, `agent/run_events.py` | Keep emission order; replace ad hoc rows with structured events |
| L-019 | Generated-image gallery | improve | F-032 | 11 | `code-old/components/image_gallery_dialog.py`, `state.py` | Add provider/cost metadata, accessibility, encrypted storage, explicit save |
| L-020 | Web source list | improve | F-033 | 11 | `code-old/state.py`, `components/main_content.py` | Render verified citation chips and untrusted-content boundaries |
| L-021 | Agent mode with local tools | replace | F-025 | 4–7 | `code-old/agent/runner.py`, `agent/run.py` | Embedded Cersei runtime replaces Python loop |
| L-022 | Read-only Ask mode | preserve | F-023 | 17 | `code-old/README.md`, `agent/permission_policy.py` | Read-only construction remains canonical |
| L-023 | Read-only Plan mode and Markdown plan | improve | F-024, F-064 | 17 | `code-old/plan_parse.py`, `components/plan_card.py` | Keep plan artifact with normative schema and trusted plan-store |
| L-024 | Tool batching and scheduler | replace | F-025 | 4 | `code-old/agent/tool_batch.py`, `agent/scheduler.py` | Cersei orchestration and deterministic event order replace Python threads |
| L-025 | Run cancellation | preserve | F-021 | 4 | `code-old/agent/run.py`, `agent/runner.py` | Required safety and steering control |
| L-026 | Directory listing and file reading | improve | F-040 | 6 | `code-old/agent/tool_registry.py`, `sandbox/fs_tools.py` | Keep capability with stronger path, limits, redaction, and schemas |
| L-027 | File writing | improve | F-041 | 6 | `code-old/agent/tool_registry.py`, `sandbox/fs_tools.py` | Add stale-write protection, atomicity, checkpoint, and full diff |
| L-028 | Delete and directory creation | improve | F-042 | 6 | `code-old/agent/tool_registry.py`, `sandbox/fs_tools.py` | Add reversible deletion, limits, and elevated approval |
| L-029 | Shell command tool | replace | F-044 | 12 | `code-old/sandbox/shell_tool.py`, `terminal_bridge.py` | Native sandboxed process/PTY layer replaces Python subprocess path |
| L-030 | Workspace path guard | improve | F-067 | 5 | `code-old/sandbox/path_guard.py` | Preserve boundary with symlink, TOCTOU, platform, and deny-list defenses |
| L-031 | Command deny guard | improve | F-044 | 12 | `code-old/sandbox/command_guard.py` | Use parsed argv, OS sandbox, policy allowlists, and typed errors |
| L-032 | Live write preview | preserve | F-029 | 6 | `code-old/agent/write_preview.py`, `components/file_viewer.py` | High-value transparency behavior |
| L-033 | Tool approval card/dialog | improve | F-030 | 7 | `code-old/components/tool_approval_card.py`, `tool_permission_dialog.py` | Normalize risk, scope, queueing, and inline approval UX |
| L-034 | Global auto-approve-writes toggle | replace | F-030, F-082 | 7, 25 | `code-old/state.py`, `agent/permission_policy.py` | Replace broad boolean with normalized scoped grants and organization policy |
| L-035 | Run checkpoint journal | improve | F-065 | 18 | `code-old/agent/run_checkpoint.py` | Replace ad hoc journal with content-addressed transactional rollback |
| L-036 | Post-run verification | improve | F-044 | 12 | `code-old/agent/verification.py` | Keep visible verification through approved sandboxed commands |
| L-037 | Plan step executor | replace | F-026 | 17 | `code-old/agent/plan_executor.py` | Build mode enforces approved step scope and durable state |
| L-038 | File viewer/editor and syntax highlighting | improve | F-050, F-051, F-052 | 13 | `code-old/components/file_viewer.py`, `code_highlighter.py` | Move to CodeMirror with large-file and diff behavior |
| L-039 | Integrated terminal panel | replace | F-053 | 12 | `code-old/components/terminal_panel.py`, `terminal_bridge.py` | Native PTY sessions replace blocking process bridge |
| L-040 | Embedded local web preview | improve | F-054 | 22 | `code-old/components/web_browser.py` | Keep local preview with explicit origin and no authenticated automation |
| L-041 | Project detection, Node setup, port discovery | improve | F-054 | 22 | `code-old/preview/project_detector.py`, `node_setup.py`, `port_extractor.py` | Preserve convenience with bounded process ownership and URL validation |
| L-042 | Whole-state JSON session persistence | replace | F-068 | 9 | `code-old/session_store.py` | Encrypted SQLite, migrations, WAL, and corruption recovery replace monolithic JSON |
| L-043 | JSON preferences store | replace | F-069 | 9 | `code-old/preferences_store.py` | Typed transactional settings replace unversioned file state |
| L-044 | Conversation cache and active-session cache | improve | F-068, F-070 | 9, 23 | `code-old/conversation_cache.py`, `conversation_sync.py` | Separate local source of truth from optional sync |
| L-045 | In-app update check/download/open installer | replace | F-075 | 27 | `code-old/updater.py`, `components/update_dialog.py` | Signed manifest, staged rollout, integrity verification, rollback |
| L-046 | Briefcase multi-platform packaging | replace | F-075 | 27 | `code-old/pyproject.toml`, `README.md` | Tauri-native signed installers and provenance replace Python packaging |
| L-047 | Markdown/regex tool-call fabrication | drop | NG-014 | — | `code-old/agent/parse_tool_calls.py`, `tests/test_parse_tool_calls.py` | Fabricated calls are ambiguous and unsafe |
| L-048 | Regex/prose output surgery | drop | NG-014 | — | `code-old/markdown_render.py`, `agent/parse_tool_calls.py` | Never silently alter model output to infer control data |
| L-049 | Shared `CODE_API_TOKEN` development bypass | drop | security baseline | 2 | `code-old/config.py`, `api_client.py` | Agent uses scoped user/device identity; no production bypass |
| L-050 | Rebinding `HOME` to the workspace for shell execution | drop | F-044 | 12 | `code-old/sandbox/shell_tool.py` | Breaks toolchains and hides boundary errors; use explicit cwd/env allowlist |
| L-051 | First-class Zotero settings integration | replace | F-046 | 19 | `code-old/components/settings_dialog.py`, `api_client.py` | Domain integrations belong behind approved MCP servers |
| L-052 | Cloud memory CRUD | replace | F-062, F-063 | 16 | `code-old/api_client.py`, `components/settings_dialog.py` | Local-first governed memory replaces generic cloud memory calls |
| L-053 | Plan cards with step status and stop | improve | F-024, F-026 | 17 | `code-old/components/plan_card.py`, `state.py` | Preserve legibility with normative plan lifecycle |
| L-054 | Syntax-aware code highlighting | preserve | F-052 | 13 | `code-old/components/code_highlighter.py` | Essential editor readability |
| L-055 | Compact run outcome and verification summary | improve | F-028 | 8 | `code-old/state.py`, `components/tool_activity.py` | Keep summary while deriving it from structured journal events |
| L-056 | Generated images stored as data URLs in messages | replace | F-032 | 11 | `code-old/state.py` | Encrypted attachment records replace inline base64 persistence |
| L-057 | Minimum-version forced update | improve | F-075 | 27 | `code-old/updater.py`, `README.md` | Keep emergency control with signed manifests and compatibility policy |
| L-058 | Arbitrary URL entry in embedded browser | improve | F-054 | 22 | `code-old/components/web_browser.py` | Restrict to validated local preview origins and explicit external-open flow |
| L-059 | Automatic session title from first prompt | preserve | F-008 | 3 | `code-old/state.py` | Useful compact history behavior |
| L-060 | Project instructions and permissions in app state | improve | F-063, F-082 | 16, 25 | `code-old/state.py`, `components/project_settings_dialog.py` | Split trusted `EURY.md` instructions from signed organization policy |
| L-061 | Animated theme and project transitions | drop | optional polish | — | `code-old/components/animated_widgets.py` | Not a product requirement; any future motion must honor reduced-motion settings |
| L-062 | Full-window authentication gate with browser-handoff state | preserve | F-010 | 10 | `code-old/components/auth_gate.py` | Clear progress and recovery states remain useful |
| L-063 | Local-only logout and refresh | improve | F-012, F-013 | 10 | `code-old/src/eury_code/app.py` | Revoke the cloud refresh chain instead of only deleting local state |
| L-064 | Project file upload and indexing status | improve | F-071 | 23 | `code-old/components/project_settings_dialog.py` | Add validation, retention, cancellation, and explicit indexing failures |
| L-065 | Shared-project members, permissions, and author labels | improve | F-074 | 24 | `code-old/state.py`, `components/project_settings_dialog.py` | Preserve permission-aware UI with event-driven updates |
| L-066 | Move and pin cloud conversations | preserve | F-070, F-071 | 23 | `code-old/conversation_sync.py` | Keep optimistic behavior with idempotency and rollback |
| L-067 | Local notification toggles with no delivery effect | drop | no target (stub) | — | `code-old/preferences_store.py`, `components/settings_dialog.py` | A setting that cannot affect delivery is misleading |
| L-068 | Usage-analytics toggle without telemetry plumbing | drop | telemetry consent contract | 26 | `code-old/preferences_store.py` | Reintroduce only with real opt-in collection and default-off behavior |
| L-069 | Saved effort and streaming toggles ignored by requests | replace | F-020, F-022 | 8, 11 | `code-old/components/settings_dialog.py`, `preferences_store.py` | Only expose settings with an enforced runtime contract |
| L-070 | Clear all cloud history | improve | F-070 | 23 | `code-old/components/settings_dialog.py` | Preserve confirmation and include attachment/retention semantics |
| L-071 | Unverified runtime font download/cache | replace | F-002 | 3 | `code-old/font_loader.py` | Bundle approved fonts or integrity-pin every asset |
| L-072 | Fabricated tool calls after a model refusal | drop | NG-014 | — | `code-old/agent/runner.py` | The client must never invent a model-requested operation |
| L-073 | Rendered retry control without a handler | replace | F-034 | 8 | `code-old/components/main_content.py` | Retry must create a first-class linked run |
| L-074 | Background timer mutating conversation state | replace | F-070 | 23 | `code-old/conversation_sync.py` | Serialize updates through the runtime and resolve conflicts explicitly |
| L-075 | Preview silently running dependency installation | replace | F-044, F-054 | 12, 22 | `code-old/preview/process_manager.py` | Every package install requires an execute grant and visible command |
| L-076 | Duplicate chat-move signal causing repeated mutation | drop | F-070 | 23 | `code-old/src/eury_code/app.py` | Remove the bug and require idempotent sync tests |
| L-077 | Synchronous settings/project API calls on UI thread | replace | F-007, F-071 | 3, 23 | `code-old/components/settings_dialog.py`, `project_settings_dialog.py` | Use cancellable async commands and explicit loading/error states |
| L-078 | Single-file viewer with direct edit/save | improve | F-051 | 13 | `code-old/components/file_viewer.py` | Preserve direct editing within tabbed editor dirty-state and conflict rules |
| L-079 | Grep implemented through shell and no native glob | replace | F-043 | 6 | `code-old/agent/scheduler.py` | Ship bounded typed search tools rather than shell synthesis |
| L-080 | Cloud project description and AI instructions | preserve | F-071, F-072 | 10, 23 | `code-old/components/project_settings_dialog.py` | Keep cloud project context clearly separate from local `EURY.md` |

## Source-area coverage

| Source area | Reviewed evidence | Inventory IDs |
|---|---|---|
| Application shell and state | `app.py`, `state.py`, `session_store.py`, `preferences_store.py` | L-001–L-007, L-034, L-042–L-043, L-059–L-060 |
| Conversation and UI components | `components/` feature widgets | L-002–L-020, L-032–L-040, L-051, L-053–L-058 |
| Agent runtime and planning | `agent/` | L-018, L-021–L-025, L-033–L-037, L-047–L-048, L-053, L-055 |
| Sandbox and terminal | `sandbox/`, `terminal_bridge.py` | L-026–L-031, L-039, L-050 |
| Preview runtime | `preview/`, `components/web_browser.py` | L-040–L-041, L-058 |
| Cloud/auth/model integration | `api_client.py`, `auth_*`, `conversation_*`, `model_catalog.py` | L-008–L-013, L-020, L-044, L-049, L-051–L-052 |
| Release/packaging | `updater.py`, `pyproject.toml`, release docs | L-045–L-046, L-057 |
| Tests and deprecated design docs | `tests/`, `docs/technical/`, `docs/phases/` | Used to validate behavior and anti-pattern classification above |

## Catalog capabilities with no legacy implementation

The following are new work, not migration-parity claims: F-004 command palette,
F-006 multi-window, F-027 review workflow, F-046 MCP, F-047/F-055 Git tools and
panel, F-048/F-049 visual tools, F-050 explorer, F-056 file search,
F-060/F-061 indexing and semantic search, F-063 `EURY.md`, F-066 skills, and
F-080–F-086 enterprise controls.

## Related documents

- [Feature catalog](02-feature-catalog.md)
- [Non-goals](05-non-goals.md)
- [Competitive landscape](../00-overview/03-competitive-landscape.md)
- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
