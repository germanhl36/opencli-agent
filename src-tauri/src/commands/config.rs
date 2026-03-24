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
    // Query all known providers in parallel and merge results.
    // Providers that fail (no key, not running) are silently skipped.
    let all_providers = ["ollama", "openrouter", "huggingface"];

    let mut handles = Vec::new();
    for provider_type in all_providers {
        let api_key = keychain::get_api_key(provider_type).ok().flatten();
        let pt = provider_type.to_string();
        handles.push(tokio::spawn(async move {
            let provider = create_provider(ProviderConfig {
                provider_type: pt,
                base_url: None,
                api_key,
                provider_name: None,
            })
            .ok()?;
            provider.list_models().await.ok()
        }));
    }

    let mut models: Vec<ModelInfo> = Vec::new();
    for handle in handles {
        if let Ok(Some(mut provider_models)) = handle.await {
            models.append(&mut provider_models);
        }
    }

    // For Ollama: also include popular models that aren't pulled yet,
    // so users can see what's available and pull them.
    let ollama_pulled: std::collections::HashSet<String> = models
        .iter()
        .filter(|m| m.provider == "ollama")
        .map(|m| m.id.split(':').next().unwrap_or(&m.id).to_string())
        .collect();

    let popular_ollama = [
        ("llama3.2:latest", "Llama 3.2 (3B)", 128_000u32),
        ("llama3.2:3b", "Llama 3.2 3B", 128_000),
        ("llama3.1:8b", "Llama 3.1 8B", 128_000),
        ("llama3.1:70b", "Llama 3.1 70B", 128_000),
        ("mistral:latest", "Mistral 7B", 32_000),
        ("mistral-nemo:latest", "Mistral Nemo 12B", 128_000),
        ("gemma3:4b", "Gemma 3 4B", 128_000),
        ("gemma3:12b", "Gemma 3 12B", 128_000),
        ("qwen2.5:7b", "Qwen 2.5 7B", 128_000),
        ("qwen2.5-coder:7b", "Qwen 2.5 Coder 7B", 128_000),
        ("deepseek-coder-v2:latest", "DeepSeek Coder V2", 128_000),
        ("codellama:latest", "Code Llama", 16_000),
        ("phi4:latest", "Phi-4 14B", 16_000),
        ("phi3:latest", "Phi-3 Medium", 128_000),
        ("nomic-embed-text:latest", "Nomic Embed Text", 8_192),
    ];

    for (id, name, ctx) in popular_ollama {
        let base = id.split(':').next().unwrap_or(id);
        if !ollama_pulled.contains(base) {
            models.push(ModelInfo {
                id: id.to_string(),
                name: format!("{} ↓ pull to use", name),
                provider: "ollama".to_string(),
                context_length: Some(ctx),
            });
        }
    }

    Ok(models)
}

#[tauri::command]
pub async fn store_api_key(provider: String, key: String) -> Result<(), String> {
    keychain::store_api_key(&provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reload_plugins(
    state: State<'_, AppState>,
    _app: tauri::AppHandle,
) -> Result<(), String> {
    // Always reload from the built-in skills directory so the registry
    // is never left empty after a reload call.
    #[cfg(debug_assertions)]
    let builtin_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    #[cfg(not(debug_assertions))]
    let builtin_root = _app
        .path()
        .resource_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let skills_dir = builtin_root.join("skills");
    let agents_dir = builtin_root.join("agents");
    let commands_file = builtin_root.join("skills").join("commands.yaml");

    let mut registry = state.plugin_registry.write().await;
    registry.load_all(&skills_dir, &agents_dir, &commands_file);

    Ok(())
}

#[tauri::command]
pub async fn list_skills(
    state: State<'_, AppState>,
) -> Result<Vec<crate::plugins::skill::Skill>, String> {
    let registry = state.plugin_registry.read().await;
    Ok(registry.skills.clone())
}

#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>,
) -> Result<Vec<crate::plugins::agent::Agent>, String> {
    let registry = state.plugin_registry.read().await;
    Ok(registry.agents.clone())
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
