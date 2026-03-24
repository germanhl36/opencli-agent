use crate::error::OpenCLIError;
use crate::llm::provider::{
    LLMProvider, LLMRequest, LLMResponse, Message, ModelInfo, StopReason, TokenEvent, ToolCall,
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc::Sender;

/// Custom provider for any OpenAI-compatible endpoint
pub struct CustomProvider {
    base_url: String,
    api_key: Option<String>,
    provider_name: String,
    client: Client,
}

impl CustomProvider {
    pub fn new(base_url: String, api_key: Option<String>, provider_name: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            provider_name: provider_name.unwrap_or_else(|| "custom".to_string()),
            client: Client::new(),
        }
    }

    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key {
            builder.bearer_auth(key)
        } else {
            builder
        }
    }
}

#[async_trait]
impl LLMProvider for CustomProvider {
    async fn complete(&self, req: LLMRequest) -> Result<LLMResponse, OpenCLIError> {
        let messages = self.format_messages(&req.messages);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
        });

        let request = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body);
        let resp = self
            .add_auth(request)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OpenCLIError::Llm(format!(
                "Custom provider error {}: {}",
                status, text
            )));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;
        let content = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let tool_calls = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|m| m.get("tool_calls"))
            .and_then(|tc| tc.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| self.parse_tool_call(tc))
                    .collect()
            })
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            tool_calls,
            stop_reason: StopReason::EndTurn,
            usage: None,
        })
    }

    async fn stream_tokens(
        &self,
        req: LLMRequest,
        tx: Sender<TokenEvent>,
    ) -> Result<(), OpenCLIError> {
        use tokio_stream::StreamExt;

        let messages = self.format_messages(&req.messages);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": true,
        });

        let request = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body);
        let resp = self
            .add_auth(request)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let _ = tx
                .send(TokenEvent::Error {
                    message: format!("Custom provider error {}: {}", status, text),
                })
                .await;
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
                        let _ = tx
                            .send(TokenEvent::Stop {
                                reason: StopReason::EndTurn,
                            })
                            .await;
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
                        let _ = tx
                            .send(TokenEvent::Text {
                                delta: delta.to_string(),
                            })
                            .await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, OpenCLIError> {
        let request = self.client.get(format!("{}/v1/models", self.base_url));
        let resp = self
            .add_auth(request)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;
        let data = json
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        let provider = self.provider_name.clone();
        Ok(data
            .into_iter()
            .filter_map(move |m| {
                let id = m.get("id")?.as_str()?.to_string();
                Some(ModelInfo {
                    id: id.clone(),
                    name: id,
                    provider: provider.clone(),
                    context_length: None,
                })
            })
            .collect())
    }

    fn format_messages(&self, msgs: &[Message]) -> Value {
        serde_json::json!(msgs
            .iter()
            .map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content,
            }))
            .collect::<Vec<_>>())
    }

    fn parse_tool_call(&self, raw: &Value) -> Option<ToolCall> {
        let id = raw.get("id")?.as_str()?.to_string();
        let name = raw.get("function")?.get("name")?.as_str()?.to_string();
        let args_str = raw
            .get("function")?
            .get("arguments")?
            .as_str()
            .unwrap_or("{}");
        let arguments = serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));
        Some(ToolCall {
            id,
            name,
            arguments,
        })
    }

    async fn health_check(&self) -> bool {
        let request = self.client.get(format!("{}/v1/models", self.base_url));
        self.add_auth(request)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
