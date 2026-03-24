use crate::error::OpenCLIError;
use crate::llm::provider::{LLMRequest, Message};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub tool_calls: Vec<serde_json::Value>,
}

impl SessionMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp(),
            tool_calls: Vec::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp(),
            tool_calls: Vec::new(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp(),
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct SessionState {
    pub id: Uuid,
    pub messages: Vec<SessionMessage>,
    pub provider: String,
    pub model: String,
    pub working_directory: Option<String>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            messages: Vec::new(),
            provider: "ollama".to_string(),
            model: "llama3.2:latest".to_string(),
            working_directory: None,
        }
    }

    pub fn reset(&mut self) {
        self.id = Uuid::new_v4();
        self.messages.clear();
    }

    pub fn to_llm_messages(&self) -> Vec<Message> {
        self.messages
            .iter()
            .map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SessionManager {
    state: Arc<RwLock<SessionState>>,
}

impl SessionManager {
    pub fn new(state: Arc<RwLock<SessionState>>) -> Self {
        Self { state }
    }

    pub async fn start(
        &self,
        provider: String,
        model: String,
        working_directory: Option<String>,
    ) -> Result<Uuid, OpenCLIError> {
        let mut state = self.state.write().await;
        state.id = Uuid::new_v4();
        state.messages.clear();
        state.provider = provider;
        state.model = model;
        state.working_directory = working_directory;
        Ok(state.id)
    }

    pub async fn reset(&self) -> Result<(), OpenCLIError> {
        let mut state = self.state.write().await;
        let provider = state.provider.clone();
        let model = state.model.clone();
        let wd = state.working_directory.clone();
        state.reset();
        state.provider = provider;
        state.model = model;
        state.working_directory = wd;
        Ok(())
    }

    pub async fn get_messages(&self) -> Vec<SessionMessage> {
        let state = self.state.read().await;
        state.messages.clone()
    }

    pub async fn add_user_message(&self, content: String) -> Result<(), OpenCLIError> {
        let mut state = self.state.write().await;
        state.messages.push(SessionMessage::user(content));
        Ok(())
    }

    pub async fn add_assistant_message(&self, content: String) -> Result<(), OpenCLIError> {
        let mut state = self.state.write().await;
        state.messages.push(SessionMessage::assistant(content));
        Ok(())
    }

    pub async fn build_llm_request(
        &self,
        context_prompt: Option<String>,
    ) -> Result<LLMRequest, OpenCLIError> {
        let state = self.state.read().await;
        let mut messages = state.to_llm_messages();

        // Prepend context as system message if provided
        if let Some(ctx) = context_prompt {
            messages.insert(
                0,
                Message {
                    role: "system".to_string(),
                    content: ctx,
                },
            );
        }

        Ok(LLMRequest {
            messages,
            model: state.model.clone(),
            temperature: Some(0.7),
            max_tokens: Some(4096),
            tools: None,
        })
    }
}
