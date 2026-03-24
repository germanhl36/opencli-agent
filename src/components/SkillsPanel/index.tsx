import { useEffect, useState, useCallback } from 'react';
import { reloadPlugins, activateSkill, startAgent } from '../../lib/tauri';
import styles from './SkillsPanel.module.css';

interface Skill {
  name: string;
  description: string;
  prompt: string;
  contextFiles: string[];
}

interface AgentStep {
  goal: string;
  prompt: string;
  allowedTools: string[];
}

interface Agent {
  name: string;
  description: string;
  steps: AgentStep[];
}

interface SkillsPanelProps {
  onSkillActivated?: (prompt: string, skillName: string) => void;
  onAgentStarted?: (steps: AgentStep[], agentName: string) => void;
}

type Tab = 'skills' | 'agents';

export default function SkillsPanel({ onSkillActivated, onAgentStarted }: SkillsPanelProps) {
  const [tab, setTab] = useState<Tab>('skills');
  const [skills, setSkills] = useState<Skill[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activatingSkill, setActivatingSkill] = useState<string | null>(null);
  const [startingAgent, setStartingAgent] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadPlugins = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      await reloadPlugins();
      // After reload, skills/agents are in the registry — we don't have a direct list endpoint
      // We use the hardcoded built-ins for the initial display
      setSkills([
        { name: 'refactor', description: 'Refactor selected code for clarity and maintainability', prompt: '', contextFiles: [] },
        { name: 'explain-code', description: 'Explain what the selected code does', prompt: '', contextFiles: [] },
        { name: 'write-tests', description: 'Write tests for the selected code', prompt: '', contextFiles: [] },
        { name: 'summarise-pr', description: 'Summarise changes in the current diff', prompt: '', contextFiles: [] },
        { name: 'debug', description: 'Debug the selected code or error', prompt: '', contextFiles: [] },
      ]);
      setAgents([
        { name: 'code-review', description: 'Multi-step code review workflow', steps: [] },
        { name: 'dependency-update', description: 'Update project dependencies safely', steps: [] },
        { name: 'scaffold-feature', description: 'Scaffold a new feature with boilerplate', steps: [] },
      ]);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPlugins();
  }, [loadPlugins]);

  const handleActivateSkill = async (skillName: string) => {
    setActivatingSkill(skillName);
    try {
      const prompt = await activateSkill(skillName, '');
      onSkillActivated?.(prompt, skillName);
    } catch (err) {
      setError(String(err));
    } finally {
      setActivatingSkill(null);
    }
  };

  const handleStartAgent = async (agentName: string) => {
    setStartingAgent(agentName);
    try {
      const steps = await startAgent(agentName) as AgentStep[];
      onAgentStarted?.(steps, agentName);
    } catch (err) {
      setError(String(err));
    } finally {
      setStartingAgent(null);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.tabs} role="tablist">
          <button
            role="tab"
            aria-selected={tab === 'skills'}
            className={`${styles.tab} ${tab === 'skills' ? styles.activeTab : ''}`}
            onClick={() => setTab('skills')}
          >
            Skills
          </button>
          <button
            role="tab"
            aria-selected={tab === 'agents'}
            className={`${styles.tab} ${tab === 'agents' ? styles.activeTab : ''}`}
            onClick={() => setTab('agents')}
          >
            Agents
          </button>
        </div>
        <button
          className={styles.reloadButton}
          onClick={loadPlugins}
          disabled={isLoading}
          aria-label="Reload plugins"
        >
          {isLoading ? '...' : 'Reload'}
        </button>
      </div>

      {error && <div className={styles.error}>{error}</div>}

      <div className={styles.list} role="tabpanel">
        {tab === 'skills' && skills.map((skill) => (
          <div key={skill.name} className={styles.card}>
            <div className={styles.cardInfo}>
              <span className={styles.cardName}>{skill.name}</span>
              <span className={styles.cardDesc}>{skill.description}</span>
            </div>
            <button
              className={styles.activateButton}
              onClick={() => handleActivateSkill(skill.name)}
              disabled={activatingSkill === skill.name}
              aria-label={`Activate skill ${skill.name}`}
            >
              {activatingSkill === skill.name ? '...' : 'Use'}
            </button>
          </div>
        ))}

        {tab === 'agents' && agents.map((agent) => (
          <div key={agent.name} className={styles.card}>
            <div className={styles.cardInfo}>
              <span className={styles.cardName}>{agent.name}</span>
              <span className={styles.cardDesc}>{agent.description}</span>
            </div>
            <button
              className={styles.activateButton}
              onClick={() => handleStartAgent(agent.name)}
              disabled={startingAgent === agent.name}
              aria-label={`Start agent ${agent.name}`}
            >
              {startingAgent === agent.name ? '...' : 'Start'}
            </button>
          </div>
        ))}

        {tab === 'skills' && skills.length === 0 && !isLoading && (
          <div className={styles.empty}>No skills found</div>
        )}
        {tab === 'agents' && agents.length === 0 && !isLoading && (
          <div className={styles.empty}>No agents found</div>
        )}
      </div>
    </div>
  );
}
