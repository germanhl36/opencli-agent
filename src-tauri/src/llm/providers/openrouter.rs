use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use crate::error::OpenCLIError;
use crate::llm::provider::{LLMProvider, LLMRequest, LLMResponse, Message, ModelInfo, StopReason, TokenEvent, TokenUsage, ToolCall};

pub struct OpenRouterProvider {
    api_key: String,
    client: Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn complete(&self, req: LLMRequest) -> Result<LLMResponse, OpenCLIError> {
        let messages = self.format_messages(&req.messages);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
        });

        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", "https://opencli.agent")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OpenCLIError::Llm(format!("OpenRouter error {}: {}", status, text)));
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

        let prompt_tokens = json.get("usage").and_then(|u| u.get("prompt_tokens")).and_then(|t| t.as_u64()).unwrap_or(0) as u32;
        let completion_tokens = json.get("usage").and_then(|u| u.get("completion_tokens")).and_then(|t| t.as_u64()).unwrap_or(0) as u32;

        let stop_reason_str = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(|r| r.as_str())
            .unwrap_or("stop");

        let stop_reason = match stop_reason_str {
            "tool_calls" => StopReason::ToolCall,
            "length" => StopReason::MaxTokens,
            _ => StopReason::EndTurn,
        };

        // Extract tool calls if present
        let tool_calls = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|m| m.get("tool_calls"))
            .and_then(|tc| tc.as_array())
            .map(|arr| {
                arr.iter().filter_map(|tc| self.parse_tool_call(tc)).collect()
            })
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            tool_calls,
            stop_reason,
            usage: Some(TokenUsage { prompt_tokens, completion_tokens }),
        })
    }

    async fn stream_tokens(&self, req: LLMRequest, tx: Sender<TokenEvent>) -> Result<(), OpenCLIError> {
        use tokio_stream::StreamExt;

        let messages = self.format_messages(&req.messages);
        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "temperature": req.temperature.unwrap_or(0.7),
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": true,
        });

        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", "https://opencli.agent")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let _ = tx.send(TokenEvent::Error { message: format!("OpenRouter error {}: {}", status, text) }).await;
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
        let resp = self.client
            .get("https://openrouter.ai/api/v1/models")
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| OpenCLIError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let json: Value = resp.json().await.map_err(|e| OpenCLIError::Llm(e.to_string()))?;
        let data = json.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
        Ok(data.into_iter().filter_map(|m| {
            let id = m.get("id")?.as_str()?.to_string();
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or(&id).to_string();
            let context_length = m.get("context_length").and_then(|c| c.as_u64()).map(|c| c as u32);
            Some(ModelInfo {
                id,
                name,
                provider: "openrouter".to_string(),
                context_length,
            })
        }).collect())
    }

    fn format_messages(&self, msgs: &[Message]) -> Value {
        serde_json::json!(msgs.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>>())
    }

    fn parse_tool_call(&self, raw: &Value) -> Option<ToolCall> {
        let id = raw.get("id")?.as_str()?.to_string();
        let name = raw.get("function")?.get("name")?.as_str()?.to_string();
        let args_str = raw.get("function")?.get("arguments")?.as_str().unwrap_or("{}");
        let arguments = serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));
        Some(ToolCall { id, name, arguments })
    }

    async fn health_check(&self) -> bool {
        self.client
            .get("https://openrouter.ai/api/v1/models")
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
