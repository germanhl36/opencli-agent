use crate::error::OpenCLIError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub goal: String,
    pub prompt: String,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub steps: Vec<AgentStep>,
}

pub fn load_agents_from_dir(dir: &Path) -> Result<Vec<Agent>, OpenCLIError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut agents = Vec::new();
    let read_dir = std::fs::read_dir(dir)?;

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            match load_agent_from_file(&path) {
                Ok(agent) => agents.push(agent),
                Err(e) => eprintln!("Failed to load agent {:?}: {}", path, e),
            }
        }
    }

    Ok(agents)
}

pub fn load_agent_from_file(path: &Path) -> Result<Agent, OpenCLIError> {
    let content = std::fs::read_to_string(path)?;
    let agent: Agent = serde_yaml::from_str(&content)
        .map_err(|e| OpenCLIError::Config(format!("Invalid agent file {:?}: {}", path, e)))?;
    Ok(agent)
}

pub fn check_tool_allowed(tool_name: &str, allowed_tools: &[String]) -> bool {
    if allowed_tools.is_empty() {
        return false;
    }
    allowed_tools.iter().any(|t| t == tool_name || t == "*")
}
