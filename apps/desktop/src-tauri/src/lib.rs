//! Thin Tauri wiring for the Eury Agent desktop shell.
//!
//! Agent runtime, tools, storage, and policy are deliberately deferred beyond
//! Phase 0 and will be provided by workspace crates.

pub mod commands;

use tauri::Manager;

/// Starts the desktop shell.
///
/// # Errors
///
/// Returns the Tauri runtime error when the application window cannot start.
pub fn run() -> Result<(), tauri::Error> {
    // The encrypted store lives in the OS app-data dir for `com.eury.agent`,
    // resolved from Tauri's own path API rather than hardcoded, so it lands
    // in the right place per platform. Resolving it needs an `App`, so state
    // is built inside `setup` rather than before `Builder::default()`.
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("eury-agent.db");

            // Failing to open the encrypted store is an environment-level
            // failure with no graceful degradation (the alternative would be
            // silently running unpersisted), so abort startup with a clear
            // message rather than surface a misleading tauri::Error.
            let state = match commands::AppState::new(&db_path) {
                Ok(state) => state,
                Err(err) => {
                    eprintln!("fatal: failed to initialize application state: {err}");
                    std::process::exit(1);
                }
            };
            app.manage(state);

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<commands::AppState>();
                state.restore_trusted_workspaces().await;
                // Re-arms the engine with the stored session, so a signed-in
                // user stays signed in across restarts without a prompt.
                commands::restore_session(&state).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace_pick_folder,
            commands::workspace_open,
            commands::workspace_close,
            commands::workspace_info,
            commands::workspace_recent,
            commands::workspace_trust_set,
            commands::workspace_list_tree,
            commands::workspace_read_file,
            commands::settings_get,
            commands::settings_set,
            commands::window_state_save,
            commands::window_state_load,
            commands::window_close,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_toggle_fullscreen,
            commands::window_is_maximized,
            commands::app_get_platform,
            commands::capabilities_get,
            commands::run_start,
            commands::run_cancel,
            commands::run_approve,
            commands::run_steer,
            commands::agent_auth_get_tokens,
            commands::agent_auth_set_tokens,
            commands::agent_auth_clear_tokens,
            commands::terminal_create,
            commands::terminal_write,
            commands::terminal_resize,
            commands::terminal_close,
            commands::terminal_list,
            commands::terminal_capture,
        ])
        .build(tauri::generate_context!())?;

    app.run(|app_handle, event| {
        // Terminal sessions hold real child processes; without this hook
        // there is no app-quit cleanup at all and closing the window would
        // orphan every open shell.
        if matches!(event, tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit) {
            let state = app_handle.state::<commands::AppState>();
            let terminals = state.terminals.clone();
            // Stop the store actor too, so its SQLite connection closes
            // cleanly rather than being torn down mid-write.
            let store = state.store.clone();
            tauri::async_runtime::block_on(async move {
                terminals.shutdown_all(std::time::Duration::from_secs(3)).await;
                store.shutdown().await;
            });
        }
    });

    Ok(())
}
