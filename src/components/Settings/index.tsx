import { useState, useEffect } from 'react';
import type { AppConfig } from '../../lib/types';
import { loadConfig, saveConfig, storeApiKey } from '../../lib/tauri';
import styles from './Settings.module.css';

interface SettingsProps {
  onClose?: () => void;
}

const PROVIDERS = ['ollama', 'openrouter', 'huggingface', 'custom'];
const THEMES = ['system', 'light', 'dark'];

export default function Settings({ onClose }: SettingsProps) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [isSavingKey, setIsSavingKey] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadConfig()
      .then(setConfig)
      .catch((err) => setError(String(err)));
  }, []);

  const handleSave = async () => {
    if (!config) return;
    setIsSaving(true);
    setError(null);
    try {
      await saveConfig(config);
      setSaveMessage('Saved successfully');
      setTimeout(() => setSaveMessage(null), 2000);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveApiKey = async () => {
    if (!config || !apiKey.trim()) return;
    setIsSavingKey(true);
    setError(null);
    try {
      await storeApiKey(config.activeProvider, apiKey.trim());
      setApiKey('');
      setSaveMessage('API key saved to keychain');
      setTimeout(() => setSaveMessage(null), 2000);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSavingKey(false);
    }
  };

  const update = (partial: Partial<AppConfig>) => {
    setConfig((prev) => prev ? { ...prev, ...partial } : prev);
  };

  if (!config) {
    return (
      <div className={styles.container}>
        <div className={styles.loading}>Loading settings...</div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>Settings</h2>
        {onClose && (
          <button className={styles.closeButton} onClick={onClose} aria-label="Close settings">
            ✕
          </button>
        )}
      </div>

      <div className={styles.body}>
        {error && <div className={styles.error}>{error}</div>}
        {saveMessage && <div className={styles.success}>{saveMessage}</div>}

        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>LLM Provider</h3>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="provider-select">Provider</label>
            <select
              id="provider-select"
              className={styles.select}
              value={config.activeProvider}
              onChange={(e) => update({ activeProvider: e.target.value })}
            >
              {PROVIDERS.map((p) => (
                <option key={p} value={p}>{p}</option>
              ))}
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="model-input">Default Model</label>
            <input
              id="model-input"
              type="text"
              className={styles.input}
              value={config.activeModel}
              onChange={(e) => update({ activeModel: e.target.value })}
              placeholder="e.g. llama3.2"
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="api-key-input">
              API Key <span className={styles.fieldNote}>(write-only, stored in system keychain)</span>
            </label>
            <div className={styles.apiKeyRow}>
              <input
                id="api-key-input"
                type="password"
                className={styles.input}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Paste API key here..."
                autoComplete="off"
              />
              <button
                className={styles.saveKeyButton}
                onClick={handleSaveApiKey}
                disabled={isSavingKey || !apiKey.trim()}
              >
                {isSavingKey ? '...' : 'Save Key'}
              </button>
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>Appearance</h3>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="theme-select">Theme</label>
            <select
              id="theme-select"
              className={styles.select}
              value={config.theme}
              onChange={(e) => update({ theme: e.target.value })}
            >
              {THEMES.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="font-size-input">Font Size</label>
            <input
              id="font-size-input"
              type="number"
              className={styles.input}
              value={config.fontSize}
              min={8}
              max={72}
              onChange={(e) => update({ fontSize: parseInt(e.target.value, 10) || 14 })}
            />
          </div>
        </section>

        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>Shell & Sandbox</h3>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="timeout-input">Command Timeout (seconds)</label>
            <input
              id="timeout-input"
              type="number"
              className={styles.input}
              value={config.commandTimeoutS}
              min={1}
              max={3600}
              onChange={(e) => update({ commandTimeoutS: parseInt(e.target.value, 10) || 30 })}
            />
          </div>

          <div className={styles.checkboxField}>
            <input
              id="sandbox-toggle"
              type="checkbox"
              checked={config.sandboxEnabled}
              onChange={(e) => update({ sandboxEnabled: e.target.checked })}
            />
            <label htmlFor="sandbox-toggle">Enable Docker sandbox</label>
          </div>

          {config.sandboxEnabled && (
            <div className={styles.field}>
              <label className={styles.label} htmlFor="sandbox-image-input">Sandbox Docker Image</label>
              <input
                id="sandbox-image-input"
                type="text"
                className={styles.input}
                value={config.sandboxImage || ''}
                onChange={(e) => update({ sandboxImage: e.target.value || null })}
                placeholder="alpine:latest"
              />
            </div>
          )}

          <div className={styles.field}>
            <label className={styles.label} htmlFor="workdir-input">Working Directory</label>
            <input
              id="workdir-input"
              type="text"
              className={styles.input}
              value={config.workingDirectory || ''}
              onChange={(e) => update({ workingDirectory: e.target.value || null })}
              placeholder="/path/to/project"
            />
          </div>
        </section>
      </div>

      <div className={styles.footer}>
        <button
          className={styles.saveButton}
          onClick={handleSave}
          disabled={isSaving}
        >
          {isSaving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
}
