mod application;
mod commands;
mod domain;
mod infrastructure;

#[cfg(test)]
mod tests;

use tauri::Manager;

/// Starts the local preview; WeChat acquisition and sending are intentionally absent.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            let state = infrastructure::storage::AppState::open(&directory)?;
            app.manage(state);
            infrastructure::platform::install(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_workspace,
            commands::profile_action,
            commands::get_provider,
            commands::save_provider,
            commands::remove_provider,
            commands::test_provider,
            commands::generate_reply,
            commands::cancel_reply,
            commands::copy_reply,
            commands::window_action,
            commands::tone_templates,
            commands::sync_settings
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Echo desktop application");
}
