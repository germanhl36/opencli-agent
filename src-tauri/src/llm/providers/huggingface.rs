use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use crate::error::OpenCLIError;
use crate::llm::provider::{LLMProvider, LLMRequest, LLMResponse, Message, ModelInfo, StopReason, TokenEvent, ToolCall};

pub struct HuggingFaceProvider {
    api_key: String,
    model_endpoint: String,
    client: Client,
}

impl HuggingFaceProvider {
    pub fn new(api_key: String, model_endpoint: Option<String>) -> Self {
        Self {
            api_key,
            model_endpoint: model_endpoint.unwrap_or_else(|| {
                "https://api-inference.huggingface.co/models".to_string()
            }),
            client: Client::new(),
        }
    }

    fn endpoint_for_model(&self, model: &str) -> String {
        if self.model_endpoint.contains(model) {
            self.model_endpoint.clone()
        } else {
            format!("{}/{}/v1/chat/completions", self.model_endpoint, model)
        }
    }
}

#[async_trait]
impl LLMProvider for HuggingFaceProvider {
    async fn complete(&self, req: LLMRequest) -> Result<LLMResponse, OpenCLIError> {
        let messages = self.format_messages(&req.messages);
        let endpoint = self.endpoint_for_model(&req.model);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
        });

        let resp = self.client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OpenCLIError::Llm(format!("HuggingFace error {}: {}", status, text)));
        }

        let json: Value = resp.json().await.map_err(|e| OpenCLIError::Llm(e.to_string()))?;
        let content = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
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

        let messages = self.format_messages(&req.messages);
        let endpoint = self.endpoint_for_model(&req.model);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": true,
        });

        let resp = self.client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let _ = tx.send(TokenEvent::Error { message: format!("HuggingFace error {}: {}", status, text) }).await;
            return Ok(());
        }

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| OpenCLIError::Llm(e.to_string()))?;
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line == "data: [DONE]" {
                    if line == "data: [DONE]" {
                        let _ = tx.send(TokenEvent::Stop { reason: StopReason::EndTurn }).await;
                    }
                    continue;
                }
                let data = line.strip_prefix("data: ").unwrap_or(line);
                if let Ok(json) = serde_json::from_str::<Value>(data) {
                    if let Some(delta) = json
                        .get("choices")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|choice| choice.get("delta"))
                        .and_then(|d| d.get("content"))
                        .and_then(|c| c.as_str())
                    {
                        let _ = tx.send(TokenEvent::Text { delta: delta.to_string() }).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, OpenCLIError> {
        // Return a curated list of popular HuggingFace inference models
        Ok(vec![
            ModelInfo {
                id: "meta-llama/Llama-3.1-8B-Instruct".to_string(),
                name: "Llama 3.1 8B Instruct".to_string(),
                provider: "huggingface".to_string(),
                context_length: Some(131072),
            },
            ModelInfo {
                id: "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
                name: "Mistral 7B Instruct v0.3".to_string(),
                provider: "huggingface".to_string(),
                context_length: Some(32768),
            },
            ModelInfo {
                id: "microsoft/Phi-3.5-mini-instruct".to_string(),
                name: "Phi-3.5 Mini Instruct".to_string(),
                provider: "huggingface".to_string(),
                context_length: Some(131072),
            },
        ])
    }

    fn format_messages(&self, msgs: &[Message]) -> Value {
        serde_json::json!(msgs.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>>())
    }

    fn parse_tool_call(&self, _raw: &Value) -> Option<ToolCall> {
        None
    }

    async fn health_check(&self) -> bool {
        self.client
            .get("https://api-inference.huggingface.co/models")
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
