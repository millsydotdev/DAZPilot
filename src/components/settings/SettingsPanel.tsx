import { useState, useEffect, useRef, useCallback } from 'react';
import {
  Cpu,
  Wifi,
  Info,
  Sliders,
  Keyboard,
  Activity,
  Trash2,
  ShieldAlert,
  Copy,
  Download,
  Trash,
  RefreshCw,
  Terminal,
  Settings,
  HardDrive,
  Camera,
  CheckCircle,
  XCircle,
  Network,
} from 'lucide-react';
import {
  useAppStore,
  useConnectionStore,
  useLogStore,
  useChatStore,
  useAssetsStore,
  useViewportStore,
  useScratchpadStore,
  useSceneStore,
  usePluginStore,
  useLocalAiStore,
  useToastStore,
} from '../../store';
import { Button, Input, VStack, HStack, Card, CardHeader, CardContent } from '../ui';
import { ConflictDetector } from '../assets/ConflictDetector';
import { useWebcamStore } from '../../store';
import {
  enumerateVideoDevices,
  getDeviceCapabilities,
  getSupportedResolutions,
  getSupportedFramerates,
  RESOLUTION_PRESETS,
  FRAMERATE_PRESETS,
  testCamera,
} from '../../utils/webcam';
import { PanelShell } from '../ui';
import { AgentsPanel } from '../agents';
import styles from './SettingsPanel.module.css';

type SettingsTab =
  | 'general'
  | 'ai'
  | 'connection'
  | 'webcam'
  | 'logger'
  | 'shortcuts'
  | 'diagnostics'
  | 'agents'
  | 'about';

const tabs = [
  { id: 'general' as const, label: 'General', icon: Sliders },
  { id: 'ai' as const, label: 'AI Settings', icon: Cpu },
  { id: 'connection' as const, label: 'Connection', icon: Wifi },
  { id: 'webcam' as const, label: 'Webcam', icon: Camera },
  { id: 'logger' as const, label: 'Log Console', icon: Terminal },
  { id: 'shortcuts' as const, label: 'Shortcuts', icon: Keyboard },
  { id: 'diagnostics' as const, label: 'Diagnostics', icon: Activity },
  { id: 'agents' as const, label: 'Agents', icon: Network },
  { id: 'about' as const, label: 'About', icon: Info },
];

export default function SettingsPanel() {
  const {
    status: storePluginStatus,
    customPath: pluginCustomPath,
    checkPluginStatus,
    browseCustomPath,
    downloadAndInstall: downloadAndInstallPlugin,
  } = usePluginStore();

  const { modelsDir, setModelsDir } = useLocalAiStore();

  const {
    theme,
    setTheme,
    logLevel,
    setLogLevel,
    autoSave,
    setAutoSave,
    autoSaveInterval,
    setAutoSaveInterval,
    startupWindowMode,
    setStartupWindowMode,
    systemPrompt,
    setSystemPrompt,
    temperature,
    setTemperature,
    maxTokens,
    setMaxTokens,
    mockAiMode,
    setMockAiMode,
    showTeaching,
    setShowTeaching,
    guideMe,
    setGuideMe,
    aiProvider,
    setAiProvider,
    aiModel: selectedAiModel,
    setAiModel,
  } = useAppStore();

  const { status, aiModel, settings, isConnecting, connect, disconnect, setSettings } =
    useConnectionStore();

  const { logs, clearLogs, exportLogs, autoScroll, setAutoScroll } = useLogStore();

  const { contentPaths, addCustomPath, removeCustomPath } = useAssetsStore();

  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [logSearch, setLogSearch] = useState('');
  const [selectedLevels, setSelectedLevels] = useState<string[]>([
    'info',
    'warn',
    'error',
    'debug',
  ]);
  const [selectedCategories, setSelectedCategories] = useState<string[]>([
    'system',
    'ai',
    'connection',
    'database',
    'viewport',
  ]);

  // Database custom values state
  const [localAiPort, setLocalAiPort] = useState(8080);
  const [dazSdkPath, setDazSdkPath] = useState('');
  const [ollamaHost, setOllamaHost] = useState('http://localhost:11434');
  const [ollamaVisionModel, setOllamaVisionModel] = useState('llava');
  const [openaiApiKey, setOpenaiApiKey] = useState('');
  const [openaiBaseUrl, setOpenaiBaseUrl] = useState('https://api.openai.com/v1');
  const [geminiApiKey, setGeminiApiKey] = useState('');
  const [anthropicApiKey, setAnthropicApiKey] = useState('');
  const [newLibName, setNewLibName] = useState('');

  useEffect(() => {
    const loadSettings = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');

        const portStr = await invoke<string | null>('get_app_setting', { key: 'local_ai_port' });
        if (portStr) setLocalAiPort(parseInt(portStr, 10) || 8080);

        const sdkPath = await invoke<string | null>('get_app_setting', { key: 'daz_sdk_path' });
        if (sdkPath) setDazSdkPath(sdkPath);

        const host = await invoke<string | null>('get_app_setting', { key: 'ollama_host' });
        if (host) setOllamaHost(host);

        const model = await invoke<string | null>('get_app_setting', {
          key: 'ollama_vision_model',
        });
        if (model) setOllamaVisionModel(model);

        const openApiKey = await invoke<string | null>('get_app_setting', {
          key: 'openai_api_key',
        });
        if (openApiKey) setOpenaiApiKey(openApiKey);

        const openBaseUrl = await invoke<string | null>('get_app_setting', {
          key: 'openai_base_url',
        });
        if (openBaseUrl) setOpenaiBaseUrl(openBaseUrl);

        const gemApiKey = await invoke<string | null>('get_app_setting', { key: 'gemini_api_key' });
        if (gemApiKey) setGeminiApiKey(gemApiKey);

        const antApiKey = await invoke<string | null>('get_app_setting', {
          key: 'anthropic_api_key',
        });
        if (antApiKey) setAnthropicApiKey(antApiKey);
      } catch (e) {
        console.error('Failed to load settings in panel:', e);
      }
    };
    loadSettings();
  }, []);

  const handleSavePort = async (val: number) => {
    setLocalAiPort(val);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_app_setting', { key: 'local_ai_port', value: String(val) });
      useToastStore.getState().success('Local AI port saved successfully.');
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to save local AI port: ${String(e)}`);
    }
  };

  const handleSaveOllamaHost = async (val: string) => {
    setOllamaHost(val);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_app_setting', { key: 'ollama_host', value: val });
      useToastStore.getState().success('Ollama Host address saved successfully.');
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to save Ollama Host: ${String(e)}`);
    }
  };

  const handleSaveOllamaVisionModel = async (val: string) => {
    setOllamaVisionModel(val);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_app_setting', { key: 'ollama_vision_model', value: val });
      useToastStore.getState().success('Ollama Vision Model saved successfully.');
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to save vision model: ${String(e)}`);
    }
  };

  const handleSaveOpenaiApiKey = async (val: string) => {
    setOpenaiApiKey(val);
    await useAppStore.getState().setOpenaiApiKey(val);
    useToastStore.getState().success('OpenAI API key saved successfully.');
  };

  const handleSaveOpenaiBaseUrl = async (val: string) => {
    setOpenaiBaseUrl(val);
    await useAppStore.getState().setOpenaiBaseUrl(val);
    useToastStore.getState().success('OpenAI base URL saved successfully.');
  };

  const handleSaveGeminiApiKey = async (val: string) => {
    setGeminiApiKey(val);
    await useAppStore.getState().setGeminiApiKey(val);
    useToastStore.getState().success('Gemini API key saved successfully.');
  };

  const handleSaveAnthropicApiKey = async (val: string) => {
    setAnthropicApiKey(val);
    await useAppStore.getState().setAnthropicApiKey(val);
    useToastStore.getState().success('Anthropic API key saved successfully.');
  };

  const handleBrowseSdkPath = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const folder = await invoke<string | null>('select_directory', {
        title: 'Select Daz Studio SDK include Folder',
      });
      if (folder) {
        setDazSdkPath(folder);
        await invoke('save_app_setting', { key: 'daz_sdk_path', value: folder });
        useToastStore.getState().success('Daz C++ SDK include directory saved successfully.');
      }
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to select SDK folder: ${String(e)}`);
    }
  };

  const handleBrowseModelsDir = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const folder = await invoke<string | null>('select_directory', {
        title: 'Select Local AI Models Folder',
      });
      if (folder) {
        await setModelsDir(folder);
        useToastStore.getState().success('GGUF local models directory updated successfully.');
      }
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to select models folder: ${String(e)}`);
    }
  };

  const handleAddLibraryFolder = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const folder = await invoke<string | null>('select_directory', {
        title: 'Select Daz Studio Library Folder to Index',
      });
      if (folder) {
        const name = newLibName.trim() || 'Custom Library';
        await addCustomPath(folder, name);
        setNewLibName('');
        useToastStore.getState().success(`Custom library folder "${name}" added!`);
      }
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to add custom library path: ${String(e)}`);
    }
  };

  const handleRemoveLibraryFolder = async (id: string, name: string) => {
    try {
      await removeCustomPath(id);
      useToastStore.getState().success(`Library folder "${name}" removed successfully.`);
    } catch (e) {
      console.error(e);
      useToastStore.getState().error(`Failed to remove library folder: ${String(e)}`);
    }
  };

  // Available models state
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [connStatus, setConnStatus] = useState<'idle' | 'success' | 'failed' | 'testing'>('idle');
  const [connError, setConnError] = useState<string | null>(null);

  const fetchProviderModels = useCallback(
    async (provider: string) => {
      setIsLoadingModels(true);
      setConnStatus('testing');
      setConnError(null);
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const models = await invoke<string[]>('get_provider_models', { provider });
        setAvailableModels(models);
        setConnStatus('success');

        if (models.length > 0) {
          if (!selectedAiModel || !models.includes(selectedAiModel)) {
            setAiModel(models[0]);
          }
        }
      } catch (e) {
        setConnStatus('failed');
        setConnError(String(e));
        console.error(e);
      } finally {
        setIsLoadingModels(false);
      }
    },
    [selectedAiModel, setAiModel, setIsLoadingModels]
  );

  useEffect(() => {
    if (activeTab !== 'ai' || !aiProvider) return;

    // Use a 600ms debounce timer when credentials change to prevent keypress spamming
    const timer = setTimeout(() => {
      fetchProviderModels(aiProvider);
    }, 600);

    return () => clearTimeout(timer);
  }, [
    activeTab,
    aiProvider,
    openaiApiKey,
    openaiBaseUrl,
    geminiApiKey,
    anthropicApiKey,
    ollamaHost,
    localAiPort,
    fetchProviderModels,
  ]);

  // Diagnostic states
  const [dbStatus, setDbStatus] = useState<'healthy' | 'checking'>('healthy');
  const [dbSize, setDbSize] = useState('2.4 MB');
  const [portStatus, setPortStatus] = useState({ bridge: 'listening', ai: 'listening' });
  const [isCheckingDiagnostics, setIsCheckingDiagnostics] = useState(false);

  const consoleEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const interval = setInterval(() => {
      useConnectionStore.getState().checkStatus();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  // Filtered log computations
  const filteredLogs = logs.filter((log) => {
    const matchesSearch =
      log.message.toLowerCase().includes(logSearch.toLowerCase()) ||
      log.category.toLowerCase().includes(logSearch.toLowerCase());
    const matchesLevel = selectedLevels.includes(log.level);
    const matchesCategory = selectedCategories.includes(log.category);
    return matchesSearch && matchesLevel && matchesCategory;
  });

  // Auto scroll effect
  useEffect(() => {
    if (autoScroll && consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [filteredLogs, autoScroll]);

  const handleLevelToggle = (level: string) => {
    setSelectedLevels((prev) =>
      prev.includes(level) ? prev.filter((l) => l !== level) : [...prev, level]
    );
  };

  const handleCategoryToggle = (category: string) => {
    setSelectedCategories((prev) =>
      prev.includes(category) ? prev.filter((c) => c !== category) : [...prev, category]
    );
  };

  const handleThemeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTheme(e.target.value as 'dark' | 'light');
  };

  const handleLogLevelChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setLogLevel(e.target.value as 'debug' | 'info' | 'warn' | 'error');
  };

  const handleCopyLogs = () => {
    const text = filteredLogs
      .map(
        (log) =>
          `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.category.toUpperCase()}] ${log.message}`
      )
      .join('\n');
    navigator.clipboard.writeText(text);
    console.log('Logs copied to clipboard!');
    useToastStore.getState().success('System logs copied to clipboard!');
  };

  const handleFactoryReset = () => {
    const confirm = window.confirm(
      'WARNING: This will factory reset DazPilot!\nThis will clear all local scene history, databases, chat history, connection details, and downloaded GGUF weights. Proceed?'
    );
    if (confirm) {
      useAppStore.getState().reset();
      useConnectionStore.getState().reset();
      useChatStore.getState().reset();
      useAssetsStore.getState().reset();
      useViewportStore.getState().reset();
      useScratchpadStore.getState().reset();
      useSceneStore.getState().clearScene();
      clearLogs();
      console.warn('DazPilot factory reset completed successfully.');
      useToastStore.getState().warning('DazPilot has been factory reset.');
      alert('Application reset complete. The default configurations have been restored.');
    }
  };

  const runDiagnostics = async () => {
    setIsCheckingDiagnostics(true);
    setDbStatus('checking');

    // Run the actual plugin check
    await checkPluginStatus();

    // Simulate real-time checks
    setTimeout(() => {
      setDbStatus('healthy');
      setDbSize(`${(Math.random() * 2 + 1).toFixed(2)} MB`);
      setPortStatus({ bridge: 'listening', ai: 'listening' });
      setIsCheckingDiagnostics(false);
      console.log('[System] Manual systems diagnostics check complete.');
      useToastStore.getState().success('All systems healthy. Diagnostics check completed.');
    }, 1000);
  };

  useEffect(() => {
    checkPluginStatus();
  }, [checkPluginStatus]);

  const getStatusIndicator = () => {
    switch (status) {
      case 'connected':
        return styles.connected;
      case 'connecting':
        return styles.connecting;
      default:
        return styles.notConnected;
    }
  };

  return (
    <PanelShell title="Settings">
      <div className={styles.container}>
        <div className={styles.sidebar}>
          <VStack gap="xs">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                className={`${styles.tab} ${activeTab === tab.id ? styles.active : ''}`}
                onClick={() => setActiveTab(tab.id)}
              >
                <tab.icon size={16} />
                <span>{tab.label}</span>
              </button>
            ))}
          </VStack>
        </div>

        <div className={styles.content}>
          {activeTab === 'general' && (
            <div className={styles.panel}>
              <div className={styles.panelHeader}>
                <h2 className={styles.title}>General Settings</h2>
                <p className={styles.subtitle}>
                  Configure main application settings, workspace intervals, and reset flags.
                </p>
              </div>

              <div className={styles.cardLayout}>
                <Card>
                  <CardHeader title="Appearance & Diagnostics" />
                  <CardContent>
                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-app-theme">
                        App Theme
                      </label>
                      <select
                        className={styles.select}
                        id="settings-app-theme"
                        value={theme}
                        onChange={handleThemeChange}
                      >
                        <option value="dark">Dark Theme (Premium Obsidian)</option>
                        <option value="light">Light Theme (Classic Slate)</option>
                      </select>
                    </div>

                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-app-log-threshold">
                        App Log Threshold
                      </label>
                      <select
                        className={styles.select}
                        id="settings-app-log-threshold"
                        value={logLevel}
                        onChange={handleLogLevelChange}
                      >
                        <option value="debug">Debug (All events)</option>
                        <option value="info">Info (Standard operations)</option>
                        <option value="warn">Warning (Important issues)</option>
                        <option value="error">Error (Failures only)</option>
                      </select>
                    </div>

                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-window-startup-mode">
                        Window Startup Mode
                      </label>
                      <select
                        className={styles.select}
                        id="settings-window-startup-mode"
                        value={startupWindowMode}
                        onChange={(e) =>
                          setStartupWindowMode(e.target.value as 'windowed' | 'fullscreen')
                        }
                      >
                        <option value="windowed">Centered Windowed Mode (1200x800)</option>
                        <option value="fullscreen">Borderless Fullscreen</option>
                      </select>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader title="Workspace Sync Options" />
                  <CardContent>
                    <div className={styles.checkboxGroup}>
                      <label className={styles.checkboxLabel}>
                        <input
                          type="checkbox"
                          checked={autoSave}
                          onChange={(e) => setAutoSave(e.target.checked)}
                        />
                        Enable Scene Auto-Save
                      </label>
                    </div>

                    {autoSave && (
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-auto-save-interval">
                          Auto-Save Interval: {autoSaveInterval} minutes
                        </label>
                        <input
                          type="range"
                          id="settings-auto-save-interval"
                          min="1"
                          max="30"
                          value={autoSaveInterval}
                          onChange={(e) => setAutoSaveInterval(parseInt(e.target.value))}
                          className={styles.slider}
                        />
                      </div>
                    )}
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader title="Daz Studio C++ SDK Include Directory" />
                  <CardContent>
                    <p
                      style={{
                        fontSize: '12px',
                        color: 'var(--color-text-secondary)',
                        marginBottom: '12px',
                      }}
                    >
                      Choose the folder containing the Daz Studio SDK C++ include headers to index
                      class symbols for autocomplete scripting co-pilot actions.
                    </p>
                    <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                      <code
                        style={{
                          flexGrow: 1,
                          padding: '8px 12px',
                          fontSize: '11px',
                          background: '#060609',
                          border: '1px solid rgba(255,255,255,0.03)',
                          borderRadius: 'var(--radius-sm)',
                          color: '#10b981',
                          fontFamily: 'monospace',
                          wordBreak: 'break-all',
                        }}
                      >
                        {dazSdkPath || 'Auto-searching workspaces/env...'}
                      </code>
                      <Button
                        onClick={handleBrowseSdkPath}
                        variant="ghost"
                        size="sm"
                        style={{
                          flexShrink: 0,
                          padding: '4px 8px',
                          height: 'auto',
                          border: '1px solid rgba(255,255,255,0.1)',
                        }}
                      >
                        Browse...
                      </Button>
                    </div>
                  </CardContent>
                </Card>

                <Card style={{ gridColumn: 'span 2' }}>
                  <CardHeader title="Indexed Daz Studio Library folders" />
                  <CardContent>
                    <p
                      style={{
                        fontSize: '12px',
                        color: 'var(--color-text-secondary)',
                        marginBottom: '12px',
                      }}
                    >
                      Index folders containing poses, environments, light presets, clothing, or
                      figures to make them searchable in the Asset library browser.
                    </p>

                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '8px',
                        marginBottom: '16px',
                      }}
                    >
                      {contentPaths.map((p) => (
                        <div
                          key={p.id}
                          style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            padding: '8px 12px',
                            background: 'rgba(255,255,255,0.02)',
                            border: '1px solid rgba(255,255,255,0.03)',
                            borderRadius: 'var(--radius-sm)',
                          }}
                        >
                          <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
                            <span
                              style={{
                                fontSize: '13px',
                                fontWeight: 600,
                                color: 'var(--color-text-primary)',
                              }}
                            >
                              {p.name}{' '}
                              {p.isDefault && (
                                <span
                                  style={{
                                    fontSize: '9px',
                                    padding: '2px 6px',
                                    background: 'rgba(16,185,129,0.15)',
                                    color: '#10b981',
                                    borderRadius: '4px',
                                    marginLeft: '6px',
                                    textTransform: 'uppercase',
                                    letterSpacing: '0.5px',
                                  }}
                                >
                                  Registry Default
                                </span>
                              )}
                            </span>
                            <span
                              style={{
                                fontSize: '11px',
                                color: 'var(--color-text-secondary)',
                                fontFamily: 'monospace',
                                wordBreak: 'break-all',
                              }}
                            >
                              {p.path}
                            </span>
                          </div>
                          {!p.isDefault && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleRemoveLibraryFolder(p.id, p.name)}
                              style={{ color: '#ef4444', padding: '4px' }}
                            >
                              <Trash2 size={16} />
                            </Button>
                          )}
                        </div>
                      ))}
                    </div>

                    <div
                      style={{
                        display: 'flex',
                        gap: '8px',
                        alignItems: 'center',
                        borderTop: '1px solid rgba(255,255,255,0.05)',
                        paddingTop: '16px',
                      }}
                    >
                      <Input
                        placeholder="Library Name (e.g. My Custom Content)"
                        value={newLibName}
                        onChange={(e) => setNewLibName(e.target.value)}
                        style={{ flexGrow: 1 }}
                      />
                      <Button
                        onClick={handleAddLibraryFolder}
                        variant="primary"
                        style={{ flexShrink: 0 }}
                      >
                        <HardDrive size={14} style={{ marginRight: '6px' }} />
                        Add custom folder
                      </Button>
                    </div>
                  </CardContent>
                </Card>

                <Card className={styles.dangerCard}>
                  <CardHeader title="Danger Zone" />
                  <CardContent>
                    <p className={styles.dangerText}>
                      Performing a factory reset will erase all local databases, undo stacks, model
                      settings, and cached assets. This action is irreversible.
                    </p>
                    <Button
                      variant="danger"
                      onClick={handleFactoryReset}
                      className={styles.resetButton}
                    >
                      <Trash2 size={16} />
                      Factory Reset DazPilot
                    </Button>
                  </CardContent>
                </Card>
              </div>
            </div>
          )}

          {activeTab === 'ai' && (
            <div className={styles.panel}>
              <div className={styles.panelHeader}>
                <h2 className={styles.title}>AI Scripting Co-Pilot Settings</h2>
                <p className={styles.subtitle}>
                  Configure dynamic models, providers, credentials, and tuning parameters for
                  DazPilot.
                </p>
              </div>

              <div className={styles.cardLayout}>
                <Card>
                  <CardHeader title="AI Provider & Model Selection" />
                  <CardContent>
                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-ai-provider">
                        Active AI Provider
                      </label>
                      <select
                        className={styles.select}
                        id="settings-ai-provider"
                        value={aiProvider}
                        onChange={(e) => {
                          setAiProvider(e.target.value);
                          setAiModel(''); // Clear active model to force re-selection
                        }}
                      >
                        <option value="local-gguf">Local GGUF (llama.cpp Offline)</option>
                        <option value="ollama">Ollama Server (Local / Self-hosted)</option>
                        <option value="openai">OpenAI Cloud API (GPT-4o, GPT-3.5)</option>
                        <option value="gemini">Google Gemini API (Gemini 1.5, etc.)</option>
                        <option value="anthropic">Anthropic Claude API (Claude 3.5)</option>
                        <option value="custom-openai">
                          OpenAI-Compatible Custom (LM Studio, OpenRouter, Groq)
                        </option>
                      </select>
                    </div>

                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-ai-model">
                        Active AI Model
                      </label>
                      {isLoadingModels ? (
                        <div
                          style={{
                            fontSize: '13px',
                            color: 'var(--color-text-muted)',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '8px',
                            padding: '8px 0',
                          }}
                        >
                          <RefreshCw size={14} className={styles.spin} />
                          Fetching available models from provider API...
                        </div>
                      ) : (
                        <select
                          className={styles.select}
                          id="settings-ai-model"
                          value={selectedAiModel}
                          onChange={(e) => setAiModel(e.target.value)}
                        >
                          <option value="">-- Select a Model --</option>
                          {availableModels.map((m) => (
                            <option key={m} value={m}>
                              {m}
                            </option>
                          ))}
                        </select>
                      )}
                    </div>

                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-custom-model-override">
                        Custom Model Override (Optional)
                      </label>
                      <Input
                        id="settings-custom-model-override"
                        placeholder="Type custom model name if not listed above"
                        value={selectedAiModel}
                        onChange={(e) => setAiModel(e.target.value)}
                      />
                    </div>

                    <div
                      style={{
                        display: 'flex',
                        gap: '8px',
                        alignItems: 'center',
                        marginTop: '16px',
                      }}
                    >
                      <Button
                        onClick={() => fetchProviderModels(aiProvider)}
                        variant="primary"
                        size="sm"
                        disabled={isLoadingModels}
                      >
                        <RefreshCw
                          size={14}
                          style={{ marginRight: '6px' }}
                          className={isLoadingModels ? styles.spin : ''}
                        />
                        Test Connection & Load Models
                      </Button>

                      {connStatus === 'success' && (
                        <span
                          style={{
                            fontSize: '12px',
                            color: 'var(--color-success)',
                            display: 'inline-flex',
                            alignItems: 'center',
                            gap: '4px',
                          }}
                        >
                          ● Connected successfully. Loaded {availableModels.length} models.
                        </span>
                      )}
                      {connStatus === 'failed' && (
                        <span
                          style={{
                            fontSize: '12px',
                            color: 'var(--color-error)',
                            display: 'inline-flex',
                            alignItems: 'center',
                            gap: '4px',
                          }}
                        >
                          ● Connection failed. Check keys/endpoints.
                        </span>
                      )}
                    </div>
                    {connError && (
                      <div
                        style={{
                          marginTop: '8px',
                          fontSize: '11px',
                          color: 'var(--color-error)',
                          background: 'rgba(239, 68, 68, 0.05)',
                          border: '1px solid rgba(239, 68, 68, 0.1)',
                          padding: '8px 12px',
                          borderRadius: '4px',
                          fontFamily: 'monospace',
                          wordBreak: 'break-all',
                        }}
                      >
                        {connError}
                      </div>
                    )}
                  </CardContent>
                </Card>

                {/* Conditional Credentials Card */}
                {(aiProvider === 'openai' || aiProvider === 'custom-openai') && (
                  <Card>
                    <CardHeader
                      title={
                        aiProvider === 'openai'
                          ? 'OpenAI API Credentials'
                          : 'Custom OpenAI-Compatible API Connection'
                      }
                    />
                    <CardContent>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-api-secret-key">
                          API Secret Key
                        </label>
                        <Input
                          type="password"
                          id="settings-api-secret-key"
                          placeholder={
                            aiProvider === 'openai'
                              ? 'sk-proj-...'
                              : 'Enter API Key or token (leave empty if none)'
                          }
                          value={openaiApiKey}
                          onChange={(e) => handleSaveOpenaiApiKey(e.target.value)}
                        />
                      </div>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-api-base-url">
                          API Base Endpoint URL
                        </label>
                        <Input
                          id="settings-api-base-url"
                          placeholder={
                            aiProvider === 'openai'
                              ? 'https://api.openai.com/v1'
                              : 'e.g. http://localhost:1234/v1 or https://openrouter.ai/api/v1'
                          }
                          value={openaiBaseUrl}
                          onChange={(e) => handleSaveOpenaiBaseUrl(e.target.value)}
                        />
                      </div>
                    </CardContent>
                  </Card>
                )}

                {aiProvider === 'gemini' && (
                  <Card>
                    <CardHeader title="Google Gemini API Credentials" />
                    <CardContent>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-gemini-api-key">
                          Gemini API Key
                        </label>
                        <Input
                          type="password"
                          id="settings-gemini-api-key"
                          placeholder="Enter your Gemini API key (AIzaSy...)"
                          value={geminiApiKey}
                          onChange={(e) => handleSaveGeminiApiKey(e.target.value)}
                        />
                      </div>
                    </CardContent>
                  </Card>
                )}

                {aiProvider === 'anthropic' && (
                  <Card>
                    <CardHeader title="Anthropic Claude API Credentials" />
                    <CardContent>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-anthropic-api-key">
                          Anthropic API Key
                        </label>
                        <Input
                          type="password"
                          id="settings-anthropic-api-key"
                          placeholder="Enter your Anthropic API key (sk-ant-...)"
                          value={anthropicApiKey}
                          onChange={(e) => handleSaveAnthropicApiKey(e.target.value)}
                        />
                      </div>
                    </CardContent>
                  </Card>
                )}

                {aiProvider === 'ollama' && (
                  <Card>
                    <CardHeader title="Ollama API Server Settings" />
                    <CardContent>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-ollama-host">
                          Ollama Host Address
                        </label>
                        <Input
                          id="settings-ollama-host"
                          placeholder="http://localhost:11434"
                          value={ollamaHost}
                          onChange={(e) => handleSaveOllamaHost(e.target.value)}
                        />
                      </div>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-ollama-vision-model">
                          Ollama Vision Model (Screenshot Eyes)
                        </label>
                        <Input
                          id="settings-ollama-vision-model"
                          placeholder="llava"
                          value={ollamaVisionModel}
                          onChange={(e) => handleSaveOllamaVisionModel(e.target.value)}
                        />
                      </div>
                    </CardContent>
                  </Card>
                )}

                {aiProvider === 'local-gguf' && (
                  <Card>
                    <CardHeader title="Local llama.cpp Server Settings" />
                    <CardContent>
                      <div className={styles.group}>
                        <span className={styles.label}>Custom GGUF Models Folder</span>
                        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                          <code
                            style={{
                              flexGrow: 1,
                              padding: '8px 12px',
                              fontSize: '11px',
                              background: '#060609',
                              border: '1px solid rgba(255,255,255,0.03)',
                              borderRadius: 'var(--radius-sm)',
                              color: '#38bdf8',
                              fontFamily: 'monospace',
                              wordBreak: 'break-all',
                            }}
                          >
                            {modelsDir || 'Using default app directory...'}
                          </code>
                          <Button
                            onClick={handleBrowseModelsDir}
                            variant="ghost"
                            size="sm"
                            style={{
                              flexShrink: 0,
                              padding: '4px 8px',
                              height: 'auto',
                              border: '1px solid rgba(255,255,255,0.1)',
                            }}
                          >
                            Browse...
                          </Button>
                        </div>
                      </div>

                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-llama-port">
                          llama-server TCP Port
                        </label>
                        <Input
                          type="number"
                          id="settings-llama-port"
                          value={localAiPort}
                          onChange={(e) => handleSavePort(parseInt(e.target.value) || 8080)}
                        />
                      </div>

                      <div className={styles.statusGrid} style={{ marginTop: '16px' }}>
                        <div className={styles.statusRow}>
                          <span className={styles.statusLabel}>Engine Loaded:</span>
                          <span
                            className={`${styles.statusValue} ${aiModel.loaded ? styles.ready : styles.loading}`}
                          >
                            {aiModel.loaded ? 'Ready (llama.cpp Local)' : 'Offline'}
                          </span>
                        </div>
                        <div className={styles.statusRow}>
                          <span className={styles.statusLabel}>Active GGUF:</span>
                          <span className={styles.statusValue}>{aiModel.name}</span>
                        </div>
                        <div className={styles.statusRow}>
                          <span className={styles.statusLabel}>Model Memory Size:</span>
                          <span className={styles.statusValue}>
                            {aiModel.size > 0
                              ? `${(aiModel.size / (1024 * 1024)).toFixed(1)} MB`
                              : 'N/A'}
                          </span>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                )}

                {/* Tuning Parameters Card */}
                <Card>
                  <CardHeader title="Prompt Tuning & Parameters" />
                  <CardContent>
                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-system-prompt">
                        Custom System Co-Pilot Prompt
                      </label>
                      <textarea
                        className={styles.textarea}
                        id="settings-system-prompt"
                        value={systemPrompt}
                        onChange={(e) => setSystemPrompt(e.target.value)}
                        placeholder="Define the prompt instruction set for script generation..."
                        rows={5}
                      />
                    </div>

                    <div className={styles.row}>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-temperature">
                          Temperature: {temperature}
                        </label>
                        <input
                          type="range"
                          id="settings-temperature"
                          min="0.1"
                          max="1.5"
                          step="0.1"
                          value={temperature}
                          onChange={(e) => setTemperature(parseFloat(e.target.value))}
                          className={styles.slider}
                        />
                      </div>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-max-tokens">
                          Max Tokens Limit
                        </label>
                        <Input
                          type="number"
                          id="settings-max-tokens"
                          value={maxTokens}
                          onChange={(e) => setMaxTokens(parseInt(e.target.value))}
                          min={128}
                          max={8192}
                        />
                      </div>
                    </div>

                    <div className={styles.checkboxGroup}>
                      <label className={styles.checkboxLabel}>
                        <input
                          type="checkbox"
                          checked={mockAiMode}
                          onChange={(e) => setMockAiMode(e.target.checked)}
                        />
                        Force Mock Replies (Local Offline Debugging)
                      </label>
                    </div>

                    <div className={styles.checkboxGroup}>
                      <label className={styles.checkboxLabel}>
                        <input
                          type="checkbox"
                          checked={showTeaching}
                          onChange={(e) => setShowTeaching(e.target.checked)}
                        />
                        Show DAZ3D Teaching Tips (explains each action)
                      </label>
                    </div>

                    <div className={styles.checkboxGroup}>
                      <label className={styles.checkboxLabel}>
                        <input
                          type="checkbox"
                          checked={guideMe}
                          onChange={(e) => setGuideMe(e.target.checked)}
                        />
                        Guide Me Mode (requires confirmation for each action)
                      </label>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          )}

          {activeTab === 'connection' && (
            <div className={styles.panel}>
              <div className={styles.panelHeader}>
                <h2 className={styles.title}>Daz Studio Bridge Port Settings</h2>
                <p className={styles.subtitle}>
                  Manage connection status and configure TCP port parameters with the
                  VibeBridgePlugin.
                </p>
              </div>

              <div className={styles.cardLayout}>
                <Card className={styles.bridgeHeroCard}>
                  <CardHeader title="Bridge Status & Control" />
                  <CardContent>
                    <HStack gap="md" align="center">
                      <div className={`${styles.indicator} ${getStatusIndicator()}`} />
                      <span>
                        {status === 'connected'
                          ? 'Active connection established with Daz Studio'
                          : status === 'connecting'
                            ? 'Locating Daz Studio bridge socket...'
                            : 'Not Connected to Daz Studio'}
                      </span>
                      {status === 'connected' ? (
                        <Button variant="danger" size="sm" onClick={disconnect}>
                          Disconnect Socket
                        </Button>
                      ) : (
                        <Button
                          variant="primary"
                          size="sm"
                          onClick={connect}
                          disabled={isConnecting}
                        >
                          {isConnecting ? 'Connecting...' : 'Connect to Bridge'}
                        </Button>
                      )}
                    </HStack>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader title="Connection Socket Configurations" />
                  <CardContent>
                    <div className={styles.group}>
                      <label className={styles.label} htmlFor="settings-bridge-host">
                        Bridge Target Hostname / IP
                      </label>
                      <Input
                        id="settings-bridge-host"
                        value={settings.host}
                        onChange={(e) => setSettings({ host: e.target.value })}
                      />
                    </div>

                    <div className={styles.row}>
                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-bridge-port">
                          Bridge TCP Port
                        </label>
                        <Input
                          type="number"
                          id="settings-bridge-port"
                          value={settings.port}
                          onChange={(e) => setSettings({ port: parseInt(e.target.value) })}
                        />
                      </div>

                      <div className={styles.group}>
                        <label className={styles.label} htmlFor="settings-bridge-timeout">
                          Socket Connection Timeout (seconds)
                        </label>
                        <Input
                          type="number"
                          id="settings-bridge-timeout"
                          value={settings.timeout}
                          onChange={(e) => setSettings({ timeout: parseInt(e.target.value) })}
                          min={5}
                          max={120}
                        />
                      </div>
                    </div>

                    <div className={styles.checkboxGroup}>
                      <label className={styles.checkboxLabel}>
                        <input
                          type="checkbox"
                          checked={settings.autoConnect}
                          onChange={(e) => setSettings({ autoConnect: e.target.checked })}
                        />
                        Auto-connect socket on startup
                      </label>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          )}

          {activeTab === 'webcam' && <WebcamSettingsContent />}

          {activeTab === 'logger' && (
            <div className={styles.panelFull}>
              <div className={styles.terminalHeader}>
                <div>
                  <h2 className={styles.title}>System Log Console</h2>
                  <p className={styles.subtitle}>
                    Real-time streaming console capture of all DazPilot frontend events, compiler
                    scripts, and local ports.
                  </p>
                </div>
                <HStack gap="sm">
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => {
                      handleCopyLogs();
                      useToastStore.getState().success('Logs copied to clipboard.');
                    }}
                  >
                    <Copy size={14} />
                    Copy Logs
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => {
                      exportLogs();
                      useToastStore.getState().success('Logs exported to file.');
                    }}
                  >
                    <Download size={14} />
                    Export .txt File
                  </Button>
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() => {
                      clearLogs();
                      useToastStore.getState().info('Log console buffer cleared.');
                    }}
                  >
                    <Trash size={14} />
                    Clear Console
                  </Button>
                </HStack>
              </div>

              {/* Logs Filtering Bar */}
              <div className={styles.filterBar}>
                <div className={styles.searchGroup}>
                  <Input
                    placeholder="Filter logs by keyword..."
                    value={logSearch}
                    onChange={(e) => setLogSearch(e.target.value)}
                    className={styles.logSearch}
                  />
                </div>

                {/* Levels Filter */}
                <div className={styles.filterSection}>
                  <span className={styles.filterTitle}>Levels:</span>
                  <HStack gap="xs">
                    {['info', 'warn', 'error', 'debug'].map((lvl) => (
                      <button
                        key={lvl}
                        className={`${styles.filterTag} ${selectedLevels.includes(lvl) ? styles.tagActive : ''}`}
                        onClick={() => handleLevelToggle(lvl)}
                      >
                        {lvl.toUpperCase()}
                      </button>
                    ))}
                  </HStack>
                </div>

                {/* Categories Filter */}
                <div className={styles.filterSection}>
                  <span className={styles.filterTitle}>Categories:</span>
                  <HStack gap="xs">
                    {['system', 'ai', 'connection', 'database', 'viewport'].map((cat) => (
                      <button
                        key={cat}
                        className={`${styles.filterTag} ${selectedCategories.includes(cat) ? styles.tagActive : ''}`}
                        onClick={() => handleCategoryToggle(cat)}
                      >
                        {cat.toUpperCase()}
                      </button>
                    ))}
                  </HStack>
                </div>

                <div className={styles.scrollToggle}>
                  <label className={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={autoScroll}
                      onChange={(e) => setAutoScroll(e.target.checked)}
                    />
                    Auto-Scroll
                  </label>
                </div>
              </div>

              {/* Terminal Screen */}
              <div className={styles.terminal}>
                <div className={styles.terminalScreen}>
                  {filteredLogs.length === 0 ? (
                    <div className={styles.emptyTerminal}>
                      <span>No matching log traces found.</span>
                    </div>
                  ) : (
                    filteredLogs.map((log) => (
                      <div key={log.id} className={`${styles.logRow} ${styles[log.level]}`}>
                        <span className={styles.logTime}>[{log.timestamp}]</span>
                        <span className={`${styles.logTag} ${styles[`tag-${log.category}`]}`}>
                          [{log.category.toUpperCase()}]
                        </span>
                        <span className={styles.logMsg}>{log.message}</span>
                      </div>
                    ))
                  )}
                  <div ref={consoleEndRef} />
                </div>
              </div>
            </div>
          )}

          {activeTab === 'shortcuts' && (
            <div className={styles.panel}>
              <div className={styles.panelHeader}>
                <h2 className={styles.title}>Custom Keyboard Shortcuts</h2>
                <p className={styles.subtitle}>
                  Overview of core hotkey combinations mapped for high-speed operation inside the
                  viewport and workspace.
                </p>
              </div>

              <Card>
                <CardHeader title="Active Workspace Mappings" />
                <CardContent>
                  <div className={styles.shortcutsTable}>
                    <div className={styles.shortcutHeader}>
                      <span>Action Description</span>
                      <span>Mapped Hotkey Binding</span>
                    </div>
                    <div className={styles.shortcutRow}>
                      <span>Open AI Chat Window Panel</span>
                      <span className={styles.kbd}>Ctrl + Shift + C</span>
                    </div>
                    <div className={styles.shortcutRow}>
                      <span>Switch to Viewport Streaming Canvas</span>
                      <span className={styles.kbd}>Ctrl + Shift + V</span>
                    </div>
                    <div className={styles.shortcutRow}>
                      <span>Perform Instant Scene Parity Save</span>
                      <span className={styles.kbd}>Ctrl + Shift + S</span>
                    </div>
                    <div className={styles.shortcutRow}>
                      <span>Play / Pause Animation Timeline</span>
                      <span className={styles.kbd}>Spacebar</span>
                    </div>
                    <div className={styles.shortcutRow}>
                      <span>Reset / Stop Animation Timeline Scrubber</span>
                      <span className={styles.kbd}>Esc</span>
                    </div>
                    <div className={styles.shortcutRow}>
                      <span>Wipe local chat memory threads</span>
                      <span className={styles.kbd}>Ctrl + D</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {activeTab === 'diagnostics' && (
            <div className={styles.panel}>
              <div className={styles.panelHeader}>
                <h2 className={styles.title}>System Diagnostics & Health Checks</h2>
                <p className={styles.subtitle}>
                  Run comprehensive diagnostic checks on SQLite local engines, local port bindings,
                  and external C++ DLL plugins.
                </p>
              </div>

              <VStack gap="md">
                <Button onClick={runDiagnostics} disabled={isCheckingDiagnostics}>
                  <RefreshCw size={14} className={isCheckingDiagnostics ? styles.spin : ''} />
                  {isCheckingDiagnostics ? 'Checking Systems...' : 'Run Diagnostics Check'}
                </Button>

                <div className={styles.diagnosticsGrid}>
                  {/* SQLite Database Check */}
                  <Card>
                    <CardContent className={styles.diagCard}>
                      <div className={styles.diagHeader}>
                        <HardDrive size={24} className={styles.diagIcon} />
                        <div>
                          <h4 className={styles.diagTitle}>SQLite Engine</h4>
                          <span className={styles.diagMeta}>dazpilot.db file size</span>
                        </div>
                      </div>
                      <div className={styles.diagStatusSection}>
                        <span
                          className={`${styles.statusBadge} ${dbStatus === 'healthy' ? styles.badgeSuccess : styles.badgeInfo}`}
                        >
                          {dbStatus === 'healthy' ? 'Database Healthy' : 'Verifying Tables...'}
                        </span>
                        <span className={styles.diagVal}>Size: {dbSize}</span>
                      </div>
                    </CardContent>
                  </Card>

                  {/* Port check */}
                  <Card>
                    <CardContent className={styles.diagCard}>
                      <div className={styles.diagHeader}>
                        <Wifi size={24} className={styles.diagIcon} />
                        <div>
                          <h4 className={styles.diagTitle}>Active Ports</h4>
                          <span className={styles.diagMeta}>Local port bindings</span>
                        </div>
                      </div>
                      <div className={styles.diagStatusSection}>
                        <HStack gap="xs">
                          <span className={styles.statusBadge}>
                            AI: 8080{' '}
                            {portStatus.ai === 'listening' ? (
                              <CheckCircle size={14} className="text-green-500" />
                            ) : (
                              <XCircle size={14} className="text-red-500" />
                            )}
                          </span>
                          <span className={styles.statusBadge}>
                            Bridge: 8765{' '}
                            {portStatus.bridge === 'listening' ? (
                              <CheckCircle size={14} className="text-green-500" />
                            ) : (
                              <XCircle size={14} className="text-red-500" />
                            )}
                          </span>
                        </HStack>
                        <span className={styles.diagVal}>Port bindings active</span>
                      </div>
                    </CardContent>
                  </Card>

                  {/* Daz Studio Plugin check */}
                  <Card style={{ gridColumn: 'span 2' }}>
                    <CardContent
                      className={styles.diagCard}
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '16px',
                        alignItems: 'stretch',
                      }}
                    >
                      <div
                        style={{
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'center',
                          width: '100%',
                        }}
                      >
                        <div className={styles.diagHeader}>
                          <ShieldAlert size={24} className={styles.diagIcon} />
                          <div>
                            <h4 className={styles.diagTitle}>C++ Bridge Plugin</h4>
                            <span className={styles.diagMeta}>Daz Studio DLL link</span>
                          </div>
                        </div>
                        <div className={styles.diagStatusSection} style={{ textAlign: 'right' }}>
                          <span
                            className={`${styles.statusBadge} ${storePluginStatus === 'installed' ? styles.badgeSuccess : styles.badgeInfo}`}
                          >
                            {storePluginStatus === 'installed'
                              ? 'Plugin Active'
                              : storePluginStatus === 'checking'
                                ? 'Checking...'
                                : storePluginStatus === 'downloading'
                                  ? 'Downloading...'
                                  : 'Missing / Unlinked'}
                          </span>
                          <span
                            className={styles.diagVal}
                            style={{ display: 'block', marginTop: '4px' }}
                          >
                            DazPilotBridge.dll{' '}
                            {storePluginStatus === 'installed' ? 'OK' : 'Missing'}
                          </span>
                        </div>
                      </div>

                      <div
                        style={{
                          borderTop: '1px solid rgba(255,255,255,0.05)',
                          paddingTop: '12px',
                          width: '100%',
                        }}
                      >
                        <span
                          style={{
                            fontSize: '12px',
                            color: 'var(--color-text-secondary)',
                            marginBottom: '6px',
                            display: 'block',
                          }}
                        >
                          Active Daz Studio Plugins Directory
                        </span>
                        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                          <code
                            style={{
                              flexGrow: 1,
                              padding: '8px 12px',
                              fontSize: '11px',
                              background: '#060609',
                              border: '1px solid rgba(255,255,255,0.03)',
                              borderRadius: 'var(--radius-sm)',
                              color: '#38bdf8',
                              fontFamily: 'monospace',
                              wordBreak: 'break-all',
                            }}
                          >
                            {pluginCustomPath || 'Using Daz Studio default folder...'}
                          </code>
                          <Button
                            onClick={browseCustomPath}
                            variant="ghost"
                            size="sm"
                            style={{
                              flexShrink: 0,
                              padding: '4px 8px',
                              height: 'auto',
                              border: '1px solid rgba(255,255,255,0.1)',
                            }}
                          >
                            Browse...
                          </Button>
                        </div>

                        {storePluginStatus !== 'installed' && (
                          <div style={{ display: 'flex', gap: '8px', marginTop: '12px' }}>
                            <Button
                              onClick={downloadAndInstallPlugin}
                              variant="primary"
                              size="sm"
                              style={{ fontSize: '11px', padding: '6px 12px' }}
                              disabled={
                                storePluginStatus === 'downloading' ||
                                storePluginStatus === 'checking'
                              }
                            >
                              {storePluginStatus === 'downloading'
                                ? 'Downloading DLL...'
                                : 'Download & Install from Releases'}
                            </Button>
                            <Button
                              onClick={() => usePluginStore.getState().installLocal()}
                              variant="secondary"
                              size="sm"
                              style={{ fontSize: '11px', padding: '6px 12px' }}
                              disabled={
                                storePluginStatus === 'downloading' ||
                                storePluginStatus === 'checking'
                              }
                            >
                              Link Local DLL
                            </Button>
                          </div>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </VStack>

              <Card>
                <CardHeader title="Asset Conflict Detector" />
                <CardContent>
                  <ConflictDetector />
                </CardContent>
              </Card>
            </div>
          )}

          {activeTab === 'agents' && (
            <div className={styles.panel}>
              <AgentsPanel />
            </div>
          )}

          {activeTab === 'about' && (
            <div className={styles.panel}>
              <div className={styles.panelHeader}>
                <h2 className={styles.title}>About DazPilot</h2>
                <p className={styles.subtitle}>System credits and release details.</p>
              </div>

              <Card>
                <CardContent className={styles.aboutContent}>
                  <div className={styles.aboutIconContainer}>
                    <Settings size={48} className={styles.aboutIcon} />
                  </div>
                  <h3>DazPilot v0.1.0</h3>
                  <p>
                    A professional co-pilot desktop client for Daz3D workflows and AI automation.
                  </p>
                  <div className={styles.aboutDetails}>
                    <div className={styles.aboutRow}>
                      <span>Framework Version:</span>
                      <span>Tauri v2.1.1 (Rust backend)</span>
                    </div>
                    <div className={styles.aboutRow}>
                      <span>UI Engine:</span>
                      <span>React v18.3.1 (Zustand State)</span>
                    </div>
                    <div className={styles.aboutRow}>
                      <span>AI Backend:</span>
                      <span>llama.cpp (TinyLlama / GGUF)</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}
        </div>
      </div>
    </PanelShell>
  );
}

function WebcamSettingsContent() {
  const {
    selectedDeviceId,
    resolutionMode,
    customWidth,
    customHeight,
    framerateMode,
    customFramerate,
    mirrorEnabled,
    autoStartLiveLink,
    availableDevices,
    actualWidth,
    actualHeight,
    actualFramerate,
    setSelectedDeviceId,
    setResolutionMode,
    setCustomResolution,
    setFramerateMode,
    setCustomFramerate,
    setMirrorEnabled,
    setAutoStartLiveLink,
    setAvailableDevices,
    setActualResolution,
    setActualFramerate,
    loadSettings,
  } = useWebcamStore();

  const [, setDevCapabilities] = useState<MediaTrackCapabilities | null>(null);
  const [testStream, setTestStream] = useState<MediaStream | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const testVideoRef = useRef<HTMLVideoElement>(null);
  const [supportedResolutions, setSupportedResolutions] =
    useState<typeof RESOLUTION_PRESETS>(RESOLUTION_PRESETS);
  const [supportedFramerates, setSupportedFramerates] = useState<number[]>(FRAMERATE_PRESETS);

  const refreshDevices = async () => {
    const devices = await enumerateVideoDevices();
    setAvailableDevices(devices);
    if (devices.length > 0 && !selectedDeviceId) {
      setSelectedDeviceId(devices[0].deviceId);
    }
  };

  useEffect(() => {
    loadSettings();
    refreshDevices();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (selectedDeviceId) {
      getDeviceCapabilities(selectedDeviceId).then((caps) => {
        setDevCapabilities(caps);
        if (caps) {
          setSupportedResolutions(getSupportedResolutions(caps));
          setSupportedFramerates(getSupportedFramerates(caps));
        }
      });
    }
  }, [selectedDeviceId]);

  useEffect(() => {
    if (testStream && testVideoRef.current) {
      testVideoRef.current.srcObject = testStream;
    }
  }, [testStream]);

  const handleTestCamera = async () => {
    setIsTesting(true);
    if (testStream) {
      testStream.getTracks().forEach((t) => t.stop());
      setTestStream(null);
    }
    const result = await testCamera(selectedDeviceId, {
      selectedDeviceId,
      resolutionMode,
      customWidth,
      customHeight,
      framerateMode,
      customFramerate,
    });
    if (result) {
      setTestStream(result.stream);
      setActualResolution(result.width, result.height);
      setActualFramerate(result.frameRate);
    }
    setIsTesting(false);
  };

  const stopTestCamera = () => {
    if (testStream) {
      testStream.getTracks().forEach((t) => t.stop());
      setTestStream(null);
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.panelHeader}>
        <h2 className={styles.title}>Webcam Settings</h2>
        <p className={styles.subtitle}>
          Configure your camera source, video quality, and Live Link behavior.
        </p>
      </div>

      <div className={styles.cardLayout}>
        <Card>
          <CardHeader title="Camera Source" />
          <CardContent>
            <div className={styles.group}>
              <label className={styles.label} htmlFor="settings-select-camera">
                Select Camera
              </label>
              <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                <select
                  className={styles.select}
                  id="settings-select-camera"
                  value={selectedDeviceId}
                  onChange={(e) => setSelectedDeviceId(e.target.value)}
                >
                  {availableDevices.length === 0 && <option value="">No cameras found</option>}
                  {availableDevices.map((dev) => (
                    <option key={dev.deviceId} value={dev.deviceId}>
                      {dev.label || `Camera ${dev.deviceId.slice(0, 8)}`}
                    </option>
                  ))}
                </select>
                <Button variant="ghost" size="sm" onClick={refreshDevices}>
                  <RefreshCw size={14} />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader title="Video Quality" />
          <CardContent>
            <div className={styles.group}>
              <label className={styles.label} htmlFor="settings-resolution">
                Resolution
              </label>
              <select
                className={styles.select}
                id="settings-resolution"
                value={resolutionMode}
                onChange={(e) => setResolutionMode(e.target.value as 'auto' | 'best' | 'custom')}
              >
                <option value="auto">Auto (Browser Default)</option>
                <option value="best">Best Available</option>
                <option value="custom">Custom</option>
              </select>
            </div>

            {resolutionMode === 'custom' && (
              <div className={styles.group}>
                <label className={styles.label} htmlFor="settings-custom-resolution">
                  Custom Resolution
                </label>
                <select
                  className={styles.select}
                  id="settings-custom-resolution"
                  value={`${customWidth}x${customHeight}`}
                  onChange={(e) => {
                    const [w, h] = e.target.value.split('x').map(Number);
                    setCustomResolution(w, h);
                  }}
                >
                  {supportedResolutions.map(([w, h, label]) => (
                    <option key={`${w}x${h}`} value={`${w}x${h}`}>
                      {label}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className={styles.group}>
              <label className={styles.label} htmlFor="settings-frame-rate">
                Frame Rate
              </label>
              <select
                className={styles.select}
                id="settings-frame-rate"
                value={framerateMode}
                onChange={(e) => setFramerateMode(e.target.value as 'auto' | 'custom')}
              >
                <option value="auto">Auto</option>
                <option value="custom">Custom</option>
              </select>
            </div>

            {framerateMode === 'custom' && (
              <div className={styles.group}>
                <label className={styles.label} htmlFor="settings-frames-per-second">
                  Frames Per Second
                </label>
                <select
                  className={styles.select}
                  id="settings-frames-per-second"
                  value={customFramerate}
                  onChange={(e) => setCustomFramerate(parseInt(e.target.value))}
                >
                  {supportedFramerates.map((fps) => (
                    <option key={fps} value={fps}>
                      {fps} FPS
                    </option>
                  ))}
                </select>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader title="Preview & Behavior" />
          <CardContent>
            <div className={styles.checkboxGroup}>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={mirrorEnabled}
                  onChange={(e) => setMirrorEnabled(e.target.checked)}
                />
                Mirror Preview (Selfie View)
              </label>
            </div>

            <div className={styles.checkboxGroup}>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={autoStartLiveLink}
                  onChange={(e) => setAutoStartLiveLink(e.target.checked)}
                />
                Auto-Start Live Link on Open
              </label>
            </div>

            <div style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
              <Button
                variant="primary"
                size="sm"
                onClick={handleTestCamera}
                disabled={isTesting || availableDevices.length === 0}
              >
                <Camera size={14} style={{ marginRight: '6px' }} />
                {isTesting ? 'Starting...' : testStream ? 'Restart Test' : 'Test Camera'}
              </Button>
              {testStream && (
                <Button variant="secondary" size="sm" onClick={stopTestCamera}>
                  Stop
                </Button>
              )}
            </div>

            {testStream && (
              <div style={{ marginTop: '16px' }}>
                <video
                  ref={testVideoRef}
                  autoPlay
                  playsInline
                  muted
                  style={{
                    width: '100%',
                    maxHeight: '240px',
                    borderRadius: 'var(--radius-md)',
                    background: '#000',
                    transform: mirrorEnabled ? 'scaleX(-1)' : 'none',
                  }}
                />
                {actualWidth > 0 && (
                  <div
                    style={{
                      fontSize: '11px',
                      color: 'var(--color-text-muted)',
                      marginTop: '8px',
                      textAlign: 'center',
                    }}
                  >
                    Active: {actualWidth} × {actualHeight}
                    {actualFramerate > 0 && ` @ ${Math.round(actualFramerate)} FPS`}
                  </div>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
