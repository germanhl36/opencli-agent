use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use crate::error::OpenCLIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TokenEvent {
    Text { delta: String },
    Tool { call: ToolCall },
    Stop { reason: StopReason },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<ToolSchema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub stop_reason: StopReason,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    ToolCall,
    Cancelled,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, req: LLMRequest) -> Result<LLMResponse, OpenCLIError>;
    async fn stream_tokens(&self, req: LLMRequest, tx: Sender<TokenEvent>) -> Result<(), OpenCLIError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, OpenCLIError>;
    fn format_messages(&self, msgs: &[Message]) -> serde_json::Value;
    fn parse_tool_call(&self, raw: &serde_json::Value) -> Option<ToolCall>;
    async fn health_check(&self) -> bool;
}
