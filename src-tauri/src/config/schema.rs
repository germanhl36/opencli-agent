use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub active_provider: String,
    pub active_model: String,
    pub theme: String,
    pub font_size: u32,
    pub command_timeout_s: u64,
    pub sandbox_enabled: bool,
    pub sandbox_image: Option<String>,
    pub working_directory: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_provider: "ollama".to_string(),
            active_model: "llama3.2:latest".to_string(),
            theme: "system".to_string(),
            font_size: 14,
            command_timeout_s: 30,
            sandbox_enabled: false,
            sandbox_image: None,
            working_directory: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub model_override: Option<String>,
    pub provider_override: Option<String>,
    #[serde(default)]
    pub auto_approve: Vec<AutoApproveRule>,
    pub sandbox_enabled: Option<bool>,
    pub sandbox_image: Option<String>,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoApproveRule {
    pub action: String,
    pub command_prefix: Option<String>,
    pub path_glob: Option<String>,
}
