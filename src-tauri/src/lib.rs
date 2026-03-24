mod commands;
mod config;
mod core;
mod error;
mod llm;
mod plugins;
mod runtime;

use tauri::Manager;

pub struct AppState {
    pub session: std::sync::Arc<tokio::sync::RwLock<core::session::SessionState>>,
    pub config: std::sync::Arc<tokio::sync::RwLock<config::schema::AppConfig>>,
    pub approval_gate: std::sync::Arc<core::approval::ApprovalGate>,
    pub plugin_registry: std::sync::Arc<tokio::sync::RwLock<plugins::PluginRegistry>>,
    pub audit_logger: std::sync::Arc<tokio::sync::Mutex<runtime::audit::AuditLogger>>,
    pub undo_stack: std::sync::Arc<tokio::sync::Mutex<runtime::undo::UndoStack>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");

            // Load persisted config
            let loader = config::loader::ConfigLoader::new(app_data_dir.clone());
            let app_config = loader.load().unwrap_or_default();

            let state = AppState {
                session: std::sync::Arc::new(tokio::sync::RwLock::new(
                    core::session::SessionState::new(),
                )),
                config: std::sync::Arc::new(tokio::sync::RwLock::new(app_config)),
                approval_gate: std::sync::Arc::new(core::approval::ApprovalGate::new()),
                plugin_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
                    plugins::PluginRegistry::new(),
                )),
                audit_logger: std::sync::Arc::new(tokio::sync::Mutex::new(
                    runtime::audit::AuditLogger::new(app_data_dir.join("audit.log")),
                )),
                undo_stack: std::sync::Arc::new(tokio::sync::Mutex::new(
                    runtime::undo::UndoStack::new(),
                )),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::start_session,
            commands::session::send_message,
            commands::session::reset_session,
            commands::session::get_history,
            commands::session::get_context,
            commands::session::resolve_approval,
            commands::session::undo_last,
            commands::fs::read_dir,
            commands::fs::apply_patch,
            commands::fs::delete_file,
            commands::shell::run_command,
            commands::shell::cancel_command,
            commands::config::load_config,
            commands::config::save_config,
            commands::config::list_models,
            commands::config::store_api_key,
            commands::config::reload_plugins,
            commands::config::activate_skill,
            commands::config::start_agent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
