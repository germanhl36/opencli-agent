use crate::error::OpenCLIError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAlias {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsConfig {
    pub commands: Vec<CommandAlias>,
}

pub fn load_commands_from_file(path: &Path) -> Result<Vec<CommandAlias>, OpenCLIError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)?;
    let config: CommandsConfig = serde_yaml::from_str(&content)
        .map_err(|e| OpenCLIError::Config(format!("Invalid commands file: {}", e)))?;
    Ok(config.commands)
}

pub fn resolve_command(name: &str, aliases: &[CommandAlias]) -> Option<String> {
    aliases
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.command.clone())
}

pub fn substitute_args(command: &str, args: &HashMap<String, String>) -> String {
    let mut result = command.to_string();
    for (key, value) in args {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}
