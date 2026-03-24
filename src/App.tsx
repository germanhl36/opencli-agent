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

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [activePanel, setActivePanel] = useState<Panel>('chat');
  const [activeDiff, setActiveDiff] = useState<UnifiedDiff | null>(null);
  const [workspacePath, setWorkspacePath] = useState<string | null>(null);

  const { messages, isLoading, error, send, reset, start } = useSession();
  const { approve, reject, currentRequest } = useApproval();
  const { streamingContent, isStreaming } = useStream();

  // Load config on mount
  useEffect(() => {
    loadConfig().then((cfg) => {
      setConfig(cfg);
      // Apply theme
      applyTheme(cfg.theme);
    }).catch(console.error);
  }, []);

  // Apply font size when config changes
  useEffect(() => {
    if (config) {
      document.documentElement.style.setProperty('--font-size', `${config.fontSize}px`);
      applyTheme(config.theme);
    }
  }, [config?.fontSize, config?.theme]);

  const applyTheme = (theme: string) => {
    const root = document.documentElement;
    if (theme === 'light') {
      root.setAttribute('data-theme', 'light');
    } else if (theme === 'dark') {
      root.setAttribute('data-theme', 'dark');
    } else {
      root.removeAttribute('data-theme'); // system
    }
  };

  const handleConfigChange = useCallback(async (newConfig: AppConfig) => {
    setConfig(newConfig);
    try {
      await saveConfig(newConfig);
    } catch (err) {
      console.error('Failed to save config:', err);
    }
  }, []);

  const handleSend = useCallback(async (content: string) => {
    await send(content);
  }, [send]);

  const handleOpenFolder = useCallback(async () => {
    const selected = await openDialog({ directory: true, multiple: false, title: 'Select workspace folder' });
    if (!selected) return;
    const folder = Array.isArray(selected) ? selected[0] : selected;
    setWorkspacePath(folder);
    await start(undefined, undefined, folder);
    setActivePanel('chat');
  }, [start]);

  const handleOpenFile = useCallback(async () => {
    const selected = await openDialog({ directory: false, multiple: false, title: 'Select a file to analyse' });
    if (!selected) return;
    const filePath = Array.isArray(selected) ? selected[0] : selected;
    try {
      const content = await readFileContent(filePath);
      const fileName = filePath.split('/').pop() ?? filePath;
      setActivePanel('chat');
      await send(`Please analyse this file — \`${fileName}\`:\n\n\`\`\`\n${content}\n\`\`\``);
    } catch (err) {
      console.error('Failed to read file:', err);
    }
  }, [send]);

  return (
    <div className={styles.appRoot}>
      {/* Sidebar */}
      <nav className={styles.sidebar} aria-label="Main navigation">
        <div className={styles.sidebarLogo}>
          <span className={styles.logoText}>OpenCLI</span>
          <span className={styles.logoVersion}>Agent</span>
        </div>

        <div className={styles.navItems}>
          <button
            className={`${styles.navItem} ${activePanel === 'chat' ? styles.navItemActive : ''}`}
            onClick={() => setActivePanel('chat')}
            aria-label="Chat"
            title="Chat"
          >
            <span className={styles.navIcon}>💬</span>
            <span className={styles.navLabel}>Chat</span>
          </button>

          <button
            className={`${styles.navItem} ${activePanel === 'skills' ? styles.navItemActive : ''}`}
            onClick={() => setActivePanel('skills')}
            aria-label="Skills & Agents"
            title="Skills & Agents"
          >
            <span className={styles.navIcon}>⚡</span>
            <span className={styles.navLabel}>Skills</span>
          </button>

          <button
            className={`${styles.navItem} ${activePanel === 'model' ? styles.navItemActive : ''}`}
            onClick={() => setActivePanel('model')}
            aria-label="Model Picker"
            title="Model Picker"
          >
            <span className={styles.navIcon}>🧠</span>
            <span className={styles.navLabel}>Model</span>
          </button>

          <button
            className={`${styles.navItem} ${activePanel === 'settings' ? styles.navItemActive : ''}`}
            onClick={() => setActivePanel('settings')}
            aria-label="Settings"
            title="Settings"
          >
            <span className={styles.navIcon}>⚙️</span>
            <span className={styles.navLabel}>Settings</span>
          </button>
        </div>

        {config && (
          <div className={styles.sidebarFooter}>
            <span className={styles.providerBadge}>{config.activeProvider}</span>
            <span className={styles.modelBadge} title={config.activeModel}>
              {config.activeModel.length > 12
                ? config.activeModel.slice(0, 12) + '…'
                : config.activeModel}
            </span>
          </div>
        )}
      </nav>

      {/* Main content */}
      <main className={styles.mainContent}>
        {/* Chat panel — always mounted but hidden when not active */}
        <div className={`${styles.panel} ${activePanel === 'chat' ? styles.panelVisible : styles.panelHidden}`}>
          <ChatPanel
            messages={messages}
            isLoading={isLoading || isStreaming}
            streamingContent={streamingContent || undefined}
            onSend={handleSend}
            onReset={reset}
            workspacePath={workspacePath}
            onOpenFolder={handleOpenFolder}
            onOpenFile={handleOpenFile}
          />
        </div>

        {activePanel === 'skills' && (
          <div className={styles.panel}>
            <SkillsPanel
              onSkillActivated={(prompt) => {
                setActivePanel('chat');
                send(prompt);
              }}
              onAgentStarted={(steps) => {
                setActivePanel('chat');
                if (steps.length > 0) {
                  send(steps[0].prompt);
                }
              }}
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

        {/* Diff viewer overlay */}
        {activeDiff && (
          <div className={styles.diffOverlay}>
            <div className={styles.diffPanel}>
              <div className={styles.diffHeader}>
                <h3 className={styles.diffTitle}>File Changes</h3>
                <button
                  className={styles.diffClose}
                  onClick={() => setActiveDiff(null)}
                  aria-label="Close diff viewer"
                >
                  ✕
                </button>
              </div>
              <div className={styles.diffContent}>
                <DiffViewer diff={activeDiff} />
              </div>
            </div>
          </div>
        )}

        {/* Error bar */}
        {error && (
          <div className={styles.errorBar} role="alert">
            {error}
          </div>
        )}
      </main>

      {/* Approval dialog — rendered at root level for proper z-index */}
      <ApprovalDialog
        request={currentRequest}
        onApprove={approve}
        onReject={reject}
      />
    </div>
  );
}
