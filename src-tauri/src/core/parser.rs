use super::approval::{ActionRequest, ActionType, RiskLevel};
use crate::error::OpenCLIError;
use serde_json::Value;
use uuid::Uuid;

pub fn parse_action_request(raw: &Value) -> Result<ActionRequest, OpenCLIError> {
    let action_str = raw
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpenCLIError::Parse("Missing 'action' field".to_string()))?;

    let action = match action_str {
        "file_write" => ActionType::FileWrite,
        "file_delete" => ActionType::FileDelete,
        "dir_create" => ActionType::DirCreate,
        "shell_run" => ActionType::ShellRun,
        other => {
            return Err(OpenCLIError::Parse(format!(
                "Unknown action type: {}",
                other
            )))
        }
    };

    let target_path = raw
        .get("target_path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let args = raw
        .get("args")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    let description = raw
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description provided")
        .to_string();

    let risk = classify_risk(&action, &target_path, &args);

    Ok(ActionRequest {
        id: Uuid::new_v4(),
        action,
        target_path,
        args,
        description,
        risk,
    })
}

fn classify_risk(action: &ActionType, target_path: &str, args: &Value) -> RiskLevel {
    match action {
        ActionType::ShellRun => {
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            // Dangerous commands = high risk
            let dangerous = [
                "rm -rf",
                "sudo",
                "chmod 777",
                "dd if=",
                "mkfs",
                ":(){:|:&};:",
            ];
            if dangerous.iter().any(|d| cmd.contains(d)) {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            }
        }
        ActionType::FileDelete => RiskLevel::High,
        ActionType::FileWrite => {
            // Writing to system paths = high risk
            if target_path.starts_with("/etc/")
                || target_path.starts_with("/usr/")
                || target_path.starts_with("/System/")
            {
                RiskLevel::High
            } else {
                RiskLevel::Low
            }
        }
        ActionType::DirCreate => RiskLevel::Low,
    }
}

pub fn parse_tool_call_from_llm(
    tool_name: &str,
    arguments: &Value,
) -> Result<ActionRequest, OpenCLIError> {
    let mut combined = serde_json::json!({
        "action": tool_name,
    });
    if let Value::Object(map) = arguments {
        for (k, v) in map {
            combined[k] = v.clone();
        }
    }
    parse_action_request(&combined)
}
