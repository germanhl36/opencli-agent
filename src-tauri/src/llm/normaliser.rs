use crate::llm::provider::{LLMRequest, LLMResponse, Message, StopReason, ToolCall};

/// Normalise provider-specific response quirks to our standard LLMResponse
pub fn normalise_response(raw_content: &str, raw_tool_calls: Vec<ToolCall>) -> LLMResponse {
    let content = raw_content.trim().to_string();
    let stop_reason = if raw_tool_calls.is_empty() {
        StopReason::EndTurn
    } else {
        StopReason::ToolCall
    };

    LLMResponse {
        content,
        tool_calls: raw_tool_calls,
        stop_reason,
        usage: None,
    }
}

/// Sanitise messages: strip empty messages, merge consecutive same-role messages
pub fn sanitise_messages(messages: Vec<Message>) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();

    for msg in messages {
        if msg.content.trim().is_empty() {
            continue;
        }
        // Merge consecutive messages with same role
        if let Some(last) = result.last_mut() {
            if last.role == msg.role {
                last.content.push('\n');
                last.content.push_str(&msg.content);
                continue;
            }
        }
        result.push(msg);
    }

    result
}

/// Truncate request messages to fit within a token budget
pub fn truncate_to_budget(req: LLMRequest, max_tokens: u32) -> LLMRequest {
    const CHARS_PER_TOKEN: usize = 4;
    let budget_chars = (max_tokens as usize) * CHARS_PER_TOKEN;

    let mut total_chars: usize = req.messages.iter().map(|m| m.content.len()).sum();
    if total_chars <= budget_chars {
        return req;
    }

    let mut messages = req.messages.clone();
    // Keep system messages and the last user message, truncate middle
    let i: usize = 1; // Always remove at index 1 (skip system message at 0)
    while total_chars > budget_chars && i < messages.len().saturating_sub(1) {
        let removed_len = messages[i].content.len();
        messages.remove(i);
        total_chars -= removed_len;
    }

    LLMRequest { messages, ..req }
}

/// Build a context system prompt from a snapshot
pub fn build_context_prompt(snapshot: &crate::core::context::ContextSnapshot) -> String {
    let mut prompt = String::from("You are OpenCLI Agent, an AI coding assistant. You have access to the following project files:\n\n");

    for file in &snapshot.files {
        prompt.push_str(&format!("### {}\n", file.path));
        if let Some(excerpt) = &file.excerpt {
            prompt.push_str("```\n");
            prompt.push_str(excerpt);
            if !excerpt.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    if snapshot.truncated {
        prompt.push_str("\n[Note: Some files were omitted due to context length limits.]\n");
    }

    prompt
}
