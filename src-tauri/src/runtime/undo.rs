use serde::{Deserialize, Serialize};
use crate::error::OpenCLIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReversePatch {
    pub path: String,
    pub original_content: Option<String>,
}

pub struct UndoStack {
    patches: Vec<ReversePatch>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self { patches: Vec::new() }
    }

    pub fn push(&mut self, patch: ReversePatch) {
        self.patches.push(patch);
    }

    pub fn pop(&mut self) -> Option<ReversePatch> {
        self.patches.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.patches.len()
    }

    pub fn apply_undo(&mut self) -> Result<Option<ReversePatch>, OpenCLIError> {
        if let Some(patch) = self.pop() {
            match &patch.original_content {
                Some(content) => {
                    std::fs::write(&patch.path, content)?;
                }
                None => {
                    // File was newly created, so delete it
                    if std::path::Path::new(&patch.path).exists() {
                        std::fs::remove_file(&patch.path)?;
                    }
                }
            }
            Ok(Some(patch))
        } else {
            Ok(None)
        }
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}
