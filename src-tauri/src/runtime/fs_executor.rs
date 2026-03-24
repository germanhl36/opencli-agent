use crate::core::approval::{
    classify_risk, ActionRequest, ActionType, ApprovalGate, ApprovalOutcome,
};
use crate::error::OpenCLIError;
use crate::runtime::audit::{AuditLogger, AuditStatus};
use crate::runtime::diff::{generate_diff, UnifiedDiff};
use crate::runtime::undo::{ReversePatch, UndoStack};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct FsExecutor {
    approval_gate: Arc<ApprovalGate>,
    undo_stack: Arc<Mutex<UndoStack>>,
    audit_logger: Arc<Mutex<AuditLogger>>,
    session_id: Uuid,
    app_handle: tauri::AppHandle,
}

impl FsExecutor {
    pub fn new(
        approval_gate: Arc<ApprovalGate>,
        undo_stack: Arc<Mutex<UndoStack>>,
        audit_logger: Arc<Mutex<AuditLogger>>,
        session_id: Uuid,
        app_handle: tauri::AppHandle,
    ) -> Self {
        Self {
            approval_gate,
            undo_stack,
            audit_logger,
            session_id,
            app_handle,
        }
    }

    pub async fn apply_patch(
        &self,
        path: &str,
        new_content: &str,
    ) -> Result<UnifiedDiff, OpenCLIError> {
        let file_path = Path::new(path);
        let old_content = if file_path.exists() {
            std::fs::read_to_string(file_path)?
        } else {
            String::new()
        };

        let diff = generate_diff(path, &old_content, new_content);

        let req = ActionRequest {
            id: Uuid::new_v4(),
            action: ActionType::FileWrite,
            target_path: path.to_string(),
            args: serde_json::json!({ "hunkCount": diff.hunks.len() }),
            description: format!("Write {} ({} hunks)", path, diff.hunks.len()),
            risk: classify_risk(&ActionType::FileWrite, path),
        };

        let outcome = self
            .approval_gate
            .request_approval(req, &self.app_handle)
            .await?;

        match outcome {
            ApprovalOutcome::Approved => {
                // Atomic write: tmp then rename
                let tmp_path = format!("{}.opencli_tmp", path);
                std::fs::write(&tmp_path, new_content)?;
                std::fs::rename(&tmp_path, path)?;

                // Undo entry
                self.undo_stack.lock().await.push(ReversePatch {
                    path: path.to_string(),
                    original_content: if old_content.is_empty() {
                        None
                    } else {
                        Some(old_content)
                    },
                });

                // Audit
                self.audit_logger.lock().await.log(
                    self.session_id,
                    "file_write",
                    path,
                    AuditStatus::Success,
                    false,
                )?;

                Ok(diff)
            }
            ApprovalOutcome::Rejected => {
                self.audit_logger.lock().await.log(
                    self.session_id,
                    "file_write",
                    path,
                    AuditStatus::Rejected,
                    false,
                )?;
                Err(OpenCLIError::ApprovalDenied)
            }
        }
    }

    pub async fn delete_file(&self, path: &str) -> Result<(), OpenCLIError> {
        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(OpenCLIError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path),
            )));
        }

        let original_content = std::fs::read_to_string(file_path).ok();

        let req = ActionRequest {
            id: Uuid::new_v4(),
            action: ActionType::FileDelete,
            target_path: path.to_string(),
            args: serde_json::json!({}),
            description: format!("Delete file: {}", path),
            risk: classify_risk(&ActionType::FileDelete, path),
        };

        let outcome = self
            .approval_gate
            .request_approval(req, &self.app_handle)
            .await?;

        match outcome {
            ApprovalOutcome::Approved => {
                std::fs::remove_file(file_path)?;

                self.undo_stack.lock().await.push(ReversePatch {
                    path: path.to_string(),
                    original_content,
                });

                self.audit_logger.lock().await.log(
                    self.session_id,
                    "file_delete",
                    path,
                    AuditStatus::Success,
                    false,
                )?;

                Ok(())
            }
            ApprovalOutcome::Rejected => {
                self.audit_logger.lock().await.log(
                    self.session_id,
                    "file_delete",
                    path,
                    AuditStatus::Rejected,
                    false,
                )?;
                Err(OpenCLIError::ApprovalDenied)
            }
        }
    }
}
