import { useEffect, useState, useCallback } from 'react';
import { listSkills, listAgents, reloadPlugins, activateSkill, startAgent } from '../../lib/tauri';
import type { Skill, AgentDefinition, AgentStep } from '../../lib/types';
import styles from './SkillsPanel.module.css';

interface SkillsPanelProps {
  onSkillActivated?: (prompt: string, skillName: string) => void;
  onAgentStarted?: (steps: AgentStep[], agentName: string) => void;
}

type Tab = 'skills' | 'agents';

export default function SkillsPanel({ onSkillActivated, onAgentStarted }: SkillsPanelProps) {
  const [tab, setTab] = useState<Tab>('skills');
  const [skills, setSkills] = useState<Skill[]>([]);
  const [agents, setAgents] = useState<AgentDefinition[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activatingSkill, setActivatingSkill] = useState<string | null>(null);
  const [startingAgent, setStartingAgent] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadPlugins = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [loadedSkills, loadedAgents] = await Promise.all([listSkills(), listAgents()]);
      setSkills(loadedSkills);
      setAgents(loadedAgents);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleReload = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      await reloadPlugins();
      const [loadedSkills, loadedAgents] = await Promise.all([listSkills(), listAgents()]);
      setSkills(loadedSkills);
      setAgents(loadedAgents);
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
          onClick={handleReload}
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
