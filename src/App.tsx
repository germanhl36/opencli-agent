import { useState, useEffect, useCallback } from 'react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import './styles/globals.css';

import ChatPanel from './components/ChatPanel';
import DiffViewer from './components/DiffViewer';
import ApprovalDialog from './components/ApprovalDialog';
import ModelPicker from './components/ModelPicker';
import SkillsPanel from './components/SkillsPanel';
import Settings from './components/Settings';

import { useSession } from './hooks/useSession';
import { useApproval } from './hooks/useApproval';
import { useStream } from './hooks/useStream';

import { loadConfig, saveConfig, readFileContent } from './lib/tauri';
import type { AppConfig, UnifiedDiff } from './lib/types';
import styles from './App.module.css';

type Panel = 'chat' | 'skills' | 'model' | 'settings';

interface SelectedFile {
  name: string;
  content: string;
}

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [activePanel, setActivePanel] = useState<Panel>('chat');
  const [activeDiff, setActiveDiff] = useState<UnifiedDiff | null>(null);
  const [workspacePath, setWorkspacePath] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<SelectedFile | null>(null);
  // Lifted chat input so skills can read + clear it
  const [chatInput, setChatInput] = useState('');

  const { messages, isLoading, error, send, reset, start } = useSession();
  const { approve, reject, currentRequest } = useApproval();
  const { streamingContent, isStreaming } = useStream();

  useEffect(() => {
    loadConfig().then((cfg) => {
      setConfig(cfg);
      applyTheme(cfg.theme);
    }).catch(console.error);
  }, []);

  useEffect(() => {
    if (config) {
      document.documentElement.style.setProperty('--font-size', `${config.fontSize}px`);
      applyTheme(config.theme);
    }
  }, [config?.fontSize, config?.theme]);

  const applyTheme = (theme: string) => {
    const root = document.documentElement;
    if (theme === 'light') root.setAttribute('data-theme', 'light');
    else if (theme === 'dark') root.setAttribute('data-theme', 'dark');
    else root.removeAttribute('data-theme');
  };

  const handleConfigChange = useCallback(async (newConfig: AppConfig) => {
    setConfig(newConfig);
    try { await saveConfig(newConfig); } catch (err) { console.error(err); }
  }, []);

  // Send from chat input — clears input and selected file after sending
  const handleSend = useCallback(async (content: string) => {
    await send(content);
    setChatInput('');
    setSelectedFile(null);
  }, [send]);

  const handleOpenFolder = useCallback(async () => {
    const selected = await openDialog({ directory: true, multiple: false, title: 'Select workspace folder' });
    if (!selected) return;
    const folder = Array.isArray(selected) ? selected[0] : selected;
    setWorkspacePath(folder);
    await start(undefined, undefined, folder);
    setActivePanel('chat');
  }, [start]);

  // File selection now stores content rather than auto-sending
  const handleOpenFile = useCallback(async () => {
    const selected = await openDialog({ directory: false, multiple: false, title: 'Select a file' });
    if (!selected) return;
    const filePath = Array.isArray(selected) ? selected[0] : selected;
    setActivePanel('chat');
    try {
      const content = await readFileContent(filePath);
      const name = filePath.replace(/\\/g, '/').split('/').pop() ?? filePath;
      setSelectedFile({ name, content });
    } catch (err) {
      await send(`⚠️ Failed to read file: ${String(err)}`);
    }
  }, [send]);

  // Build the full message for a skill: skill prompt + typed text + file content
  const handleSkillActivated = useCallback(async (skillPrompt: string) => {
    const parts: string[] = [skillPrompt];

    if (chatInput.trim()) {
      parts.push(chatInput.trim());
    }

    if (selectedFile) {
      parts.push(`\`\`\`${selectedFile.name}\n${selectedFile.content}\n\`\`\``);
    }

    const fullMessage = parts.join('\n\n');
    setChatInput('');
    setSelectedFile(null);
    setActivePanel('chat');
    await send(fullMessage);
  }, [chatInput, selectedFile, send]);

  const handleAgentStarted = useCallback(async (steps: { prompt: string }[]) => {
    if (steps.length === 0) return;
    setActivePanel('chat');
    await handleSkillActivated(steps[0].prompt);
  }, [handleSkillActivated]);

  return (
    <div className={styles.appRoot}>
      {/* Sidebar */}
      <nav className={styles.sidebar} aria-label="Main navigation">
        <div className={styles.sidebarLogo}>
          <span className={styles.logoText}>OpenCLI</span>
          <span className={styles.logoVersion}>Agent</span>
        </div>

        <div className={styles.navItems}>
          {(['chat', 'skills', 'model', 'settings'] as Panel[]).map((panel) => (
            <button
              key={panel}
              className={`${styles.navItem} ${activePanel === panel ? styles.navItemActive : ''}`}
              onClick={() => setActivePanel(panel)}
              aria-label={panel}
              title={panel}
            >
              <span className={styles.navIcon}>
                {panel === 'chat' ? '💬' : panel === 'skills' ? '⚡' : panel === 'model' ? '🧠' : '⚙️'}
              </span>
              <span className={styles.navLabel}>{panel === 'model' ? 'Model' : panel.charAt(0).toUpperCase() + panel.slice(1)}</span>
            </button>
          ))}
        </div>

        {config && (
          <div className={styles.sidebarFooter}>
            <span className={styles.providerBadge}>{config.activeProvider}</span>
            <span className={styles.modelBadge} title={config.activeModel}>
              {config.activeModel.length > 12 ? config.activeModel.slice(0, 12) + '…' : config.activeModel}
            </span>
          </div>
        )}
      </nav>

      {/* Main content */}
      <main className={styles.mainContent}>
        <div className={`${styles.panel} ${activePanel === 'chat' ? styles.panelVisible : styles.panelHidden}`}>
          <ChatPanel
            messages={messages}
            isLoading={isLoading || isStreaming}
            streamingContent={streamingContent || undefined}
            inputValue={chatInput}
            onInputChange={setChatInput}
            onSend={handleSend}
            onReset={reset}
            workspacePath={workspacePath}
            selectedFile={selectedFile}
            onClearFile={() => setSelectedFile(null)}
            onOpenFolder={handleOpenFolder}
            onOpenFile={handleOpenFile}
          />
        </div>

        {activePanel === 'skills' && (
          <div className={styles.panel}>
            <SkillsPanel
              onSkillActivated={handleSkillActivated}
              onAgentStarted={handleAgentStarted}
            />
          </div>
        )}

        {activePanel === 'model' && config && (
          <div className={styles.panel}>
            <ModelPicker config={config} onConfigChange={handleConfigChange} />
          </div>
        )}

        {activePanel === 'settings' && (
          <div className={styles.panel}>
            <Settings onClose={() => setActivePanel('chat')} />
          </div>
        )}

        {activeDiff && (
          <div className={styles.diffOverlay}>
            <div className={styles.diffPanel}>
              <div className={styles.diffHeader}>
                <h3 className={styles.diffTitle}>File Changes</h3>
                <button className={styles.diffClose} onClick={() => setActiveDiff(null)} aria-label="Close diff viewer">✕</button>
              </div>
              <div className={styles.diffContent}>
                <DiffViewer diff={activeDiff} />
              </div>
            </div>
          </div>
        )}

        {error && <div className={styles.errorBar} role="alert">{error}</div>}
      </main>

      <ApprovalDialog request={currentRequest} onApprove={approve} onReject={reject} />
    </div>
  );
}
