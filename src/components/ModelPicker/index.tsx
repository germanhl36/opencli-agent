import { useState, useEffect, useCallback } from 'react';
import type { ModelInfo, AppConfig } from '../../lib/types';
import { listModels, saveConfig } from '../../lib/tauri';
import styles from './ModelPicker.module.css';

interface ModelPickerProps {
  config: AppConfig;
  onConfigChange: (config: AppConfig) => void;
}

export default function ModelPicker({ config, onConfigChange }: ModelPickerProps) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [providerFilter, setProviderFilter] = useState<string>('all');

  const loadModels = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await listModels();
      setModels(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadModels();
  }, [loadModels]);

  const providers = ['all', ...Array.from(new Set(models.map((m) => m.provider)))];

  const filtered = models.filter((m) => {
    const matchesSearch = m.name.toLowerCase().includes(search.toLowerCase()) ||
      m.id.toLowerCase().includes(search.toLowerCase());
    const matchesProvider = providerFilter === 'all' || m.provider === providerFilter;
    return matchesSearch && matchesProvider;
  });

  const handleSelect = async (model: ModelInfo) => {
    const newConfig: AppConfig = {
      ...config,
      activeProvider: model.provider,
      activeModel: model.id,
    };
    onConfigChange(newConfig);
    await saveConfig(newConfig).catch(console.error);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h3 className={styles.title}>Select Model</h3>
        <button
          className={styles.refreshButton}
          onClick={loadModels}
          disabled={isLoading}
          aria-label="Refresh model list"
        >
          {isLoading ? '...' : 'Refresh'}
        </button>
      </div>

      <div className={styles.controls}>
        <input
          type="search"
          className={styles.searchInput}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search models..."
          aria-label="Search models"
        />
        <div className={styles.providerTabs} role="tablist">
          {providers.map((p) => (
            <button
              key={p}
              role="tab"
              aria-selected={providerFilter === p}
              className={`${styles.providerTab} ${providerFilter === p ? styles.activeTab : ''}`}
              onClick={() => setProviderFilter(p)}
            >
              {p}
            </button>
          ))}
        </div>
      </div>

      {error && <div className={styles.error}>{error}</div>}

      <div className={styles.modelList} role="listbox" aria-label="Available models">
        {isLoading && <div className={styles.loading}>Loading models...</div>}
        {!isLoading && filtered.length === 0 && (
          <div className={styles.empty}>No models found</div>
        )}
        {filtered.map((model) => {
          const isSelected =
            model.id === config.activeModel && model.provider === config.activeProvider;
          const needsPull = model.name.includes('↓ pull to use');
          const displayName = model.name.replace(' ↓ pull to use', '');
          return (
            <button
              key={`${model.provider}:${model.id}`}
              role="option"
              aria-selected={isSelected}
              className={`${styles.modelItem} ${isSelected ? styles.selectedModel : ''} ${needsPull ? styles.unpulledModel : ''}`}
              onClick={() => handleSelect(model)}
              title={needsPull ? `Run: ollama pull ${model.id}` : model.id}
            >
              <div className={styles.modelInfo}>
                <span className={styles.modelName}>{displayName}</span>
                <span className={styles.modelId}>{model.id}</span>
              </div>
              <div className={styles.modelMeta}>
                <span className={styles.providerBadge}>{model.provider}</span>
                {model.contextLength && (
                  <span className={styles.contextLength}>
                    {(model.contextLength / 1000).toFixed(0)}k ctx
                  </span>
                )}
                {needsPull && <span className={styles.pullBadge}>↓ pull</span>}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
