use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tauri::Emitter;
use crate::error::OpenCLIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub id: Uuid,
    pub action: ActionType,
    pub target_path: String,
    pub args: serde_json::Value,
    pub description: String,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    FileWrite,
    FileDelete,
    DirCreate,
    ShellRun,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalOutcome {
    Approved,
    Rejected,
}

/// Payload emitted to the frontend when approval is needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    pub token: String,
    pub action: ActionRequest,
}

pub struct ApprovalGate {
    pending: Arc<Mutex<HashMap<Uuid, oneshot::Sender<ApprovalOutcome>>>>,
}

impl ApprovalGate {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Request approval from the frontend for an action.
    /// Emits "approval-requested" event and blocks until resolved.
    pub async fn request_approval(
        &self,
        action: ActionRequest,
        app_handle: &tauri::AppHandle,
    ) -> Result<ApprovalOutcome, OpenCLIError> {
        let (tx, rx) = oneshot::channel();
        let token = action.id;
        {
            let mut pending = self.pending.lock().await;
            pending.insert(token, tx);
        }

        let payload = ApprovalRequest {
            token: token.to_string(),
            action,
        };
        app_handle
            .emit("approval-requested", payload)
            .map_err(|e| OpenCLIError::Shell(e.to_string()))?;

        rx.await.map_err(|_| OpenCLIError::ApprovalDenied)
    }

    /// Resolve a pending approval by token UUID.
    pub async fn resolve(&self, token: Uuid, outcome: ApprovalOutcome) -> Result<(), OpenCLIError> {
        let mut pending = self.pending.lock().await;
        if let Some(tx) = pending.remove(&token) {
            let _ = tx.send(outcome);
            Ok(())
        } else {
            Err(OpenCLIError::Config(format!("No pending approval with id {}", token)))
        }
    }
}

impl Default for ApprovalGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify the risk level of an action.
pub fn classify_risk(action: &ActionType, path: &str) -> RiskLevel {
    match action {
        ActionType::DirCreate => RiskLevel::Low,
        ActionType::FileWrite => {
            if path.starts_with("/etc/")
                || path.starts_with("/usr/")
                || path.starts_with("/System/")
                || path.contains("..")
            {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            }
        }
        ActionType::FileDelete | ActionType::ShellRun => RiskLevel::High,
    }
}
