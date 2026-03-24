use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::error::OpenCLIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub ts: String,
    pub session_id: String,
    pub action: String,
    pub target: String,
    pub status: AuditStatus,
    pub auto: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Success,
    Rejected,
    Error,
}

pub struct AuditLogger {
    path: PathBuf,
    file: Option<File>,
}

impl AuditLogger {
    pub fn new(path: PathBuf) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok();
        Self { path, file }
    }

    pub fn log(
        &mut self,
        session_id: Uuid,
        action: &str,
        target: &str,
        status: AuditStatus,
        auto: bool,
    ) -> Result<(), OpenCLIError> {
        let entry = AuditEntry {
            ts: Utc::now().to_rfc3339(),
            session_id: session_id.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            status,
            auto,
        };

        let line = serde_json::to_string(&entry)? + "\n";

        if let Some(file) = &mut self.file {
            file.write_all(line.as_bytes())?;
            file.flush()?;
        } else {
            // Try to reopen the file if it was not available at init
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&self.path) {
                f.write_all(line.as_bytes())?;
            }
        }

        Ok(())
    }
}
