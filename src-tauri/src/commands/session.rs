use tauri::State;
use uuid::Uuid;
use crate::AppState;
use crate::core::session::SessionMessage;
use crate::core::context::{ContextBuilder, ContextSnapshot};
use crate::core::approval::ApprovalOutcome;
use crate::llm::factory::{create_provider, ProviderConfig};
use crate::llm::normaliser::build_context_prompt;
use crate::config::keychain;

#[tauri::command]
pub async fn start_session(
    provider: Option<String>,
    model: Option<String>,
    working_directory: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.read().await;
    let p = provider.unwrap_or_else(|| config.active_provider.clone());
    let m = model.unwrap_or_else(|| config.active_model.clone());
    let wd = working_directory.or_else(|| config.working_directory.clone());
    drop(config);

    let mut session = state.session.write().await;
    session.id = Uuid::new_v4();
    session.messages.clear();
    session.provider = p;
    session.model = m;
    session.working_directory = wd;

    Ok(session.id.to_string())
}

#[tauri::command]
pub async fn send_message(
    content: String,
    state: State<'_, AppState>,
    _app: tauri::AppHandle,
) -> Result<String, String> {
    // Add user message
    {
        let mut session = state.session.write().await;
        session.messages.push(SessionMessage::user(content.clone()));
    }

    // Build context snapshot if working directory is set
    let context_prompt = {
        let session = state.session.read().await;
        if let Some(wd) = &session.working_directory {
            let wd = wd.clone();
            drop(session);
            let builder = ContextBuilder::new(std::path::PathBuf::from(&wd));
            builder.build_snapshot(None)
                .ok()
                .map(|snapshot| build_context_prompt(&snapshot))
        } else {
            None
        }
    };

    // Get provider and model from session
    let (provider_type, model) = {
        let session = state.session.read().await;
        (session.provider.clone(), session.model.clone())
    };

    // Get API key from keychain (never from config)
    let api_key = keychain::get_api_key(&provider_type)
        .ok()
        .flatten();

    let provider = create_provider(ProviderConfig {
        provider_type,
        base_url: None,
        api_key,
        provider_name: None,
    })
    .map_err(|e| e.to_string())?;

    // Build LLM request
    let messages_for_request = {
        let session = state.session.read().await;
        session.to_llm_messages()
    };

    let mut all_messages = Vec::new();
    if let Some(ctx) = context_prompt {
        all_messages.push(crate::llm::provider::Message {
            role: "system".to_string(),
            content: ctx,
        });
    }
    all_messages.extend(messages_for_request);

    let request = crate::llm::provider::LLMRequest {
        messages: all_messages,
        model,
        temperature: Some(0.7),
        max_tokens: Some(4096),
        tools: None,
    };

    let response = provider.complete(request).await.map_err(|e| e.to_string())?;

    // Store assistant response
    {
        let mut session = state.session.write().await;
        session.messages.push(SessionMessage::assistant(response.content.clone()));
    }

    Ok(response.content)
}

#[tauri::command]
pub async fn reset_session(state: State<'_, AppState>) -> Result<(), String> {
    let mut session = state.session.write().await;
    let provider = session.provider.clone();
    let model = session.model.clone();
    let wd = session.working_directory.clone();
    session.id = Uuid::new_v4();
    session.messages.clear();
    session.provider = provider;
    session.model = model;
    session.working_directory = wd;
    Ok(())
}

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<SessionMessage>, String> {
    let session = state.session.read().await;
    Ok(session.messages.clone())
}

#[tauri::command]
pub async fn get_context(state: State<'_, AppState>) -> Result<ContextSnapshot, String> {
    let session = state.session.read().await;
    let wd = session.working_directory.clone();
    drop(session);

    if let Some(wd) = wd {
        let builder = ContextBuilder::new(std::path::PathBuf::from(wd));
        builder.build_snapshot(None).map_err(|e| e.to_string())
    } else {
        Ok(ContextSnapshot {
            files: Vec::new(),
            total_tokens: 0,
            truncated: false,
        })
    }
}

#[tauri::command]
pub async fn resolve_approval(
    approval_id: String,
    approved: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&approval_id).map_err(|e| e.to_string())?;
    let outcome = if approved {
        ApprovalOutcome::Approved
    } else {
        ApprovalOutcome::Rejected
    };
    state.approval_gate.resolve(id, outcome).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn undo_last(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let mut undo_stack = state.undo_stack.lock().await;
    match undo_stack.apply_undo().map_err(|e| e.to_string())? {
        Some(patch) => Ok(Some(patch.path)),
        None => Ok(None),
    }
}
