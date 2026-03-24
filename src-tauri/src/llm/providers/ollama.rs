use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use crate::error::OpenCLIError;
use crate::llm::provider::{LLMProvider, LLMRequest, LLMResponse, Message, ModelInfo, StopReason, TokenEvent, ToolCall};

pub struct OllamaProvider {
    base_url: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(&self, req: LLMRequest) -> Result<LLMResponse, OpenCLIError> {
        let messages = self.format_messages(&req.messages);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": req.temperature.unwrap_or(0.7),
                "num_predict": req.max_tokens.unwrap_or(4096),
            }
        });

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OpenCLIError::Llm(format!("Ollama error {}: {}", status, text)));
        }

        let json: Value = resp.json().await.map_err(|e| OpenCLIError::Llm(e.to_string()))?;
        let content = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        Ok(LLMResponse {
            content,
            tool_calls: Vec::new(),
            stop_reason: StopReason::EndTurn,
            usage: None,
        })
    }

    async fn stream_tokens(&self, req: LLMRequest, tx: Sender<TokenEvent>) -> Result<(), OpenCLIError> {
        use tokio_stream::StreamExt;
        use reqwest::header;

        let messages = self.format_messages(&req.messages);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "stream": true,
            "options": {
                "temperature": req.temperature.unwrap_or(0.7),
                "num_predict": req.max_tokens.unwrap_or(4096),
            }
        });

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let _ = tx.send(TokenEvent::Error { message: format!("Ollama error {}: {}", status, text) }).await;
            return Ok(());
        }

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| OpenCLIError::Llm(e.to_string()))?;
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                if let Ok(json) = serde_json::from_str::<Value>(line) {
                    let done = json.get("done").and_then(|d| d.as_bool()).unwrap_or(false);
                    if done {
                        let _ = tx.send(TokenEvent::Stop { reason: StopReason::EndTurn }).await;
                        break;
                    }
                    if let Some(delta) = json.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                        let _ = tx.send(TokenEvent::Text { delta: delta.to_string() }).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, OpenCLIError> {
        let resp = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let json: Value = resp.json().await.map_err(|e| OpenCLIError::Llm(e.to_string()))?;
        let models = json.get("models").and_then(|m| m.as_array()).cloned().unwrap_or_default();
        Ok(models.into_iter().filter_map(|m| {
            let id = m.get("name")?.as_str()?.to_string();
            Some(ModelInfo {
                id: id.clone(),
                name: id,
                provider: "ollama".to_string(),
                context_length: None,
            })
        }).collect())
    }

    fn format_messages(&self, msgs: &[Message]) -> Value {
        serde_json::json!(msgs.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>>())
    }

    fn parse_tool_call(&self, _raw: &Value) -> Option<ToolCall> {
        // Ollama doesn't natively support tool calls yet
        None
    }

    async fn health_check(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
