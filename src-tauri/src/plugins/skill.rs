use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::OpenCLIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub prompt: String,
    #[serde(default)]
    pub context_files: Vec<String>,
}

pub fn load_skills_from_dir(dir: &Path) -> Result<Vec<Skill>, OpenCLIError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();
    let read_dir = std::fs::read_dir(dir)?;

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            match load_skill_from_file(&path) {
                Ok(skill) => skills.push(skill),
                Err(e) => eprintln!("Failed to load skill {:?}: {}", path, e),
            }
        }
    }

    Ok(skills)
}

pub fn load_skill_from_file(path: &Path) -> Result<Skill, OpenCLIError> {
    let content = std::fs::read_to_string(path)?;
    let skill: Skill = serde_yaml::from_str(&content)
        .map_err(|e| OpenCLIError::Config(format!("Invalid skill file {:?}: {}", path, e)))?;
    Ok(skill)
}

pub fn build_skill_prompt(skill: &Skill, user_input: &str, context: Option<&str>) -> String {
    let mut prompt = skill.prompt.clone();

    if let Some(ctx) = context {
        prompt = format!("{}\n\nContext:\n{}", prompt, ctx);
    }

    if !user_input.is_empty() {
        prompt = format!("{}\n\n{}", prompt, user_input);
    }

    prompt
}
