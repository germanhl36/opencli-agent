use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenCLIError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Invalid project config: {0}")]
    InvalidProjectConfig(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Approval denied")]
    ApprovalDenied,
    #[error("Agent permission denied: {0}")]
    AgentPermissionDenied(String),
    #[error("Keychain error: {0}")]
    Keychain(String),
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    #[error("Shell error: {0}")]
    Shell(String),
    #[error("Command timeout")]
    CommandTimeout,
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("YAML error: {0}")]
    Yaml(String),
    #[error("Reqwest error: {0}")]
    Reqwest(String),
}

impl From<OpenCLIError> for String {
    fn from(e: OpenCLIError) -> String {
        e.to_string()
    }
}
