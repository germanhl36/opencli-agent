use crate::runtime::diff::UnifiedDiff;
use crate::runtime::fs_executor::FsExecutor;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

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
            size_bytes: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
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

    executor
        .apply_patch(&path, &new_content)
        .await
        .map_err(|e| e.to_string())
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

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }
    if p.is_dir() {
        return Err("Path is a directory, not a file".to_string());
    }
    // Refuse files larger than 2 MB to avoid overwhelming the LLM context
    let size = std::fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .len();
    if size > 2 * 1024 * 1024 {
        return Err(format!(
            "File is too large ({} KB). Maximum is 2 MB.",
            size / 1024
        ));
    }
    // Try UTF-8 first; fall back to lossy decoding for Latin-1 / other encodings
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(s),
        Err(_) => {
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            // Reject true binary files (high proportion of null bytes)
            let null_count = bytes.iter().filter(|&&b| b == 0).count();
            if null_count > bytes.len() / 10 {
                return Err("File appears to be binary and cannot be analysed as text.".to_string());
            }
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }
}
