pub mod agent;
pub mod commands;
pub mod skill;

use crate::plugins::agent::Agent;
use crate::plugins::commands::CommandAlias;
use crate::plugins::skill::Skill;

#[derive(Debug, Default)]
pub struct PluginRegistry {
    pub skills: Vec<Skill>,
    pub agents: Vec<Agent>,
    pub commands: Vec<CommandAlias>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_all(
        &mut self,
        skills_dir: &std::path::Path,
        agents_dir: &std::path::Path,
        commands_file: &std::path::Path,
    ) {
        self.skills = skill::load_skills_from_dir(skills_dir).unwrap_or_default();
        self.agents = agent::load_agents_from_dir(agents_dir).unwrap_or_default();
        self.commands = commands::load_commands_from_file(commands_file).unwrap_or_default();
    }

    pub fn find_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }

    pub fn find_agent(&self, name: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.name == name)
    }
}
