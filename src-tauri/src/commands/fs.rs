use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::runtime::diff::UnifiedDiff;
use crate::runtime::fs_executor::FsExecutor;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_at: i64,
}

#[tauri::command]
pub async fn read_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let dir_path = std::path::Path::new(&path);
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let entries = std::fs::read_dir(dir_path).map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        result.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size_bytes: if metadata.is_file() { metadata.len() } else { 0 },
            modified_at,
        });
    }

    result.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(result)
}

#[tauri::command]
pub async fn apply_patch(
    path: String,
    new_content: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<UnifiedDiff, String> {
    let session_id = {
        let session = state.session.read().await;
        session.id
    };

    let executor = FsExecutor::new(
        state.approval_gate.clone(),
        state.undo_stack.clone(),
        state.audit_logger.clone(),
        session_id,
        app_handle,
    );

    executor.apply_patch(&path, &new_content).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_file(
    path: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let session_id = {
        let session = state.session.read().await;
        session.id
    };

    let executor = FsExecutor::new(
        state.approval_gate.clone(),
        state.undo_stack.clone(),
        state.audit_logger.clone(),
        session_id,
        app_handle,
    );

    executor.delete_file(&path).await.map_err(|e| e.to_string())
}
