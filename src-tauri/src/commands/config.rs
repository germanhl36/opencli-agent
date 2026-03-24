use crate::config::keychain;
use crate::config::loader::ConfigLoader;
use crate::config::schema::AppConfig;
use crate::llm::factory::{create_provider, ProviderConfig};
use crate::llm::provider::ModelInfo;
use crate::AppState;
use tauri::{Manager, State};

#[tauri::command]
pub async fn load_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    ConfigLoader::validate(&config).map_err(|e| e.to_string())?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    let loader = ConfigLoader::new(app_data_dir);
    loader.save(&config).map_err(|e| e.to_string())?;

    let mut state_config = state.config.write().await;
    *state_config = config;

    Ok(())
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let config = state.config.read().await;
    let provider_type = config.active_provider.clone();
    drop(config);

    // Get API key from keychain (never from config)
    let api_key = keychain::get_api_key(&provider_type).ok().flatten();

    let provider = create_provider(ProviderConfig {
        provider_type,
        base_url: None,
        api_key,
        provider_name: None,
    })
    .map_err(|e| e.to_string())?;

    provider.list_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn store_api_key(provider: String, key: String) -> Result<(), String> {
    keychain::store_api_key(&provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reload_plugins(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use std::path::PathBuf;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;

    // Look for skills/agents in app_data_dir and working directory
    let config = state.config.read().await;
    let working_dir = config
        .working_directory
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| app_data_dir.clone());
    drop(config);

    let skills_dir = working_dir.join("skills");
    let agents_dir = working_dir.join("agents");
    let commands_file = working_dir.join("commands.yaml");

    let mut registry = state.plugin_registry.write().await;
    registry.load_all(&skills_dir, &agents_dir, &commands_file);

    Ok(())
}

#[tauri::command]
pub async fn activate_skill(
    skill_name: String,
    user_input: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let registry = state.plugin_registry.read().await;
    let skill = registry
        .find_skill(&skill_name)
        .ok_or_else(|| format!("Skill '{}' not found", skill_name))?
        .clone();
    drop(registry);

    let prompt = crate::plugins::skill::build_skill_prompt(&skill, &user_input, None);

    // Add as a user message and return the prompt
    let session_state = state.session.write().await;
    drop(session_state);

    Ok(prompt)
}

#[tauri::command]
pub async fn start_agent(
    agent_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let registry = state.plugin_registry.read().await;
    let agent = registry
        .find_agent(&agent_name)
        .ok_or_else(|| format!("Agent '{}' not found", agent_name))?
        .clone();
    drop(registry);

    let steps: Vec<serde_json::Value> = agent
        .steps
        .iter()
        .map(|step| {
            serde_json::json!({
                "goal": step.goal,
                "prompt": step.prompt,
                "allowedTools": step.allowed_tools,
            })
        })
        .collect();

    Ok(steps)
}
