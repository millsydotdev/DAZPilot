import { useState, useEffect, useCallback } from 'react';
import {
  Loader2,
  Cpu,
  Check,
  AlertCircle,
  RefreshCw,
  Download,
  FolderOpen,
  ArrowLeft,
  Zap,
  Monitor,
} from 'lucide-react';
import { useLocalAiStore, type LocalModelInfo } from '../store/localAiStore';
import { useOllamaStore } from '../store/ollamaStore';
import { useConnectionStore } from '../store/connectionStore';
import { usePluginStore } from '../store';
import { Button, Card, VStack, Text } from './ui';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store';
import styles from './FirstLaunchWizard.module.css';

interface FirstLaunchWizardProps {
  onComplete: () => void;
}

type WizardStep =
  | 'checking'
  | 'ai_backend_choice'
  | 'ollama_setup'
  | 'no_model'
  | 'downloading'
  | 'ready'
  | 'plugin_setup'
  | 'plugin_downloading'
  | 'bridge_test'
  | 'sdk_setup'
  | 'starting'
  | 'ready_to_launch'
  | 'error';

type AiBackend = 'local' | 'ollama';

interface ModelPreset {
  id: string;
  name: string;
  description: string;
  url: string;
  filename: string;
  sizeDesc: string;
}

const MODEL_PRESETS: ModelPreset[] = [
  {
    id: 'tinyllama',
    name: 'TinyLlama 1.1B',
    description: 'Fastest model, great for simple scripting and low-end hardware.',
    url: 'https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_S.gguf',
    filename: 'tinyllama-1.1b-chat-v1.0.Q4_K_S.gguf',
    sizeDesc: '~600MB',
  },
  {
    id: 'phi2',
    name: 'Phi-2 2.7B',
    description: 'Balanced speed and reasoning, excellent for scene logic.',
    url: 'https://huggingface.co/TheBloke/phi-2-GGUF/resolve/main/phi-2.Q4_K_M.gguf',
    filename: 'phi-2.Q4_K_M.gguf',
    sizeDesc: '~1.8GB',
  },
  {
    id: 'llama3',
    name: 'Llama-3 8B Instruct',
    description: 'High quality reasoning, slower but best at complex commands.',
    url: 'https://huggingface.co/QuantFactory/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf',
    filename: 'Meta-Llama-3-8B-Instruct.Q4_K_M.gguf',
    sizeDesc: '~4.9GB',
  },
];

const WIZARD_STEPS: { key: WizardStep; label: string }[] = [
  { key: 'ai_backend_choice', label: 'AI Backend' },
  { key: 'no_model', label: 'Model' },
  { key: 'plugin_setup', label: 'Plugin' },
  { key: 'sdk_setup', label: 'SDK' },
  { key: 'ready_to_launch', label: 'Launch' },
];

function getStepIndex(step: WizardStep): number {
  const map: Record<string, number> = {
    checking: -1,
    ai_backend_choice: 0,
    ollama_setup: 0,
    no_model: 1,
    downloading: 1,
    ready: 1,
    plugin_setup: 2,
    plugin_downloading: 2,
    bridge_test: 2,
    sdk_setup: 3,
    starting: 4,
    ready_to_launch: 4,
    error: -1,
  };
  return map[step] ?? -1;
}

function isValidGgufUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return (
      (parsed.protocol === 'https:' || parsed.protocol === 'http:') &&
      (parsed.pathname.endsWith('.gguf') || parsed.pathname.includes('.gguf'))
    );
  } catch {
    return false;
  }
}

interface DownloadProgressPayload {
  progress: number;
  total: number | null;
  downloaded: number;
}

export function FirstLaunchWizard({ onComplete }: FirstLaunchWizardProps) {
  const [step, setStep] = useState<WizardStep>('checking');
  const [stepHistory, setStepHistory] = useState<WizardStep[]>([]);
  const [aiBackend, setAiBackend] = useState<AiBackend>('local');
  const [selectedLocalModel, setSelectedLocalModel] = useState<LocalModelInfo | null>(null);
  const [selectedPreset, setSelectedPreset] = useState<ModelPreset>(MODEL_PRESETS[0]);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadedBytes, setDownloadedBytes] = useState(0);
  const [customGgufUrl, setCustomGgufUrl] = useState('');
  const [customGgufFilename, setCustomGgufFilename] = useState('');
  const [customUrlError, setCustomUrlError] = useState<string | null>(null);
  const [sdkPath, setSdkPath] = useState<string | null>(null);
  const [sdkChecking, setSdkChecking] = useState(false);
  const [bridgeTestResult, setBridgeTestResult] = useState<'success' | 'failed' | null>(null);
  const [bridgeTesting, setBridgeTesting] = useState(false);
  const [selectedOllamaModel, setSelectedOllamaModel] = useState<string | null>(null);
  const [ollamaPullName, setOllamaPullName] = useState('');

  const setAiProvider = useAppStore((s) => s.setAiProvider);

  const {
    isRunning,
    checkServerStatus,
    startServer,
    loadModels,
    models,
    getModelsDir,
    modelsDir,
    error,
    isLoading,
    downloadModel,
  } = useLocalAiStore();

  const {
    isRunning: ollamaRunning,
    models: ollamaModels,
    isLoading: ollamaLoading,
    checkStatus: checkOllamaStatus,
    loadModels: loadOllamaModels,
    pullModel: pullOllamaModel,
  } = useOllamaStore();

  const {
    status: pluginStatus,
    customPath: pluginCustomPath,
    checkPluginStatus,
    browseCustomPath,
    downloadAndInstall: downloadAndInstallPlugin,
    installLocal: installLocalPlugin,
    error: pluginError,
  } = usePluginStore();

  const { connect: connectBridge } = useConnectionStore();

  const navigateTo = useCallback(
    (newStep: WizardStep) => {
      setStepHistory((prev) => [...prev, step]);
      setStep(newStep);
    },
    [step]
  );

  const goBack = useCallback(() => {
    setStepHistory((prev) => {
      const next = [...prev];
      const last = next.pop();
      if (last) setStep(last);
      return next;
    });
  }, []);

  useEffect(() => {
    const init = async () => {
      await getModelsDir();
      await loadModels();
      await checkServerStatus();
      await checkPluginStatus();
      await checkOllamaStatus();
    };
    init();
  }, [checkServerStatus, getModelsDir, loadModels, checkPluginStatus, checkOllamaStatus]);

  useEffect(() => {
    if (step !== 'checking') return;

    if (isRunning) {
      setStep('ready_to_launch');
    } else if (models.length > 0) {
      setAiBackend('local');
      setSelectedLocalModel(models[0]);
      setStep('ready');
    } else if (!isLoading) {
      setStep('ai_backend_choice');
    }
  }, [isRunning, models, step, isLoading]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      unlisten = await listen<DownloadProgressPayload>('download-progress', (event) => {
        setDownloadProgress(event.payload.progress);
        setDownloadedBytes(event.payload.downloaded);
      });
    };
    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleSelectBackend = (backend: AiBackend) => {
    setAiBackend(backend);
    if (backend === 'ollama') {
      navigateTo('ollama_setup');
      loadOllamaModels().catch(console.error);
    } else {
      navigateTo('no_model');
    }
  };

  const handleOllamaPull = async () => {
    if (!ollamaPullName.trim()) return;
    await pullOllamaModel(ollamaPullName.trim());
    setSelectedOllamaModel(ollamaPullName.trim());
  };

  const handleOllamaContinue = async () => {
    if (!selectedOllamaModel && ollamaModels.length > 0) {
      setSelectedOllamaModel(ollamaModels[0].name);
    }
    try {
      await setAiProvider('ollama');
    } catch {
      // ignore
    }
    navigateTo('plugin_setup');
  };

  const handleDownloadModel = async () => {
    navigateTo('downloading');
    setDownloadProgress(0);
    setDownloadedBytes(0);
    try {
      await downloadModel(selectedPreset.url, selectedPreset.filename);
      setDownloadProgress(100);
      await loadModels();
      await checkServerStatus();
      if (models.length > 0) {
        setSelectedLocalModel(models[0]);
        setStep('ready');
      }
    } catch (e) {
      console.error('Download failed:', e);
      setStep('error');
    }
  };

  const handleDownloadCustomModel = async () => {
    const url = customGgufUrl.trim();
    if (!url) return;

    if (!isValidGgufUrl(url)) {
      setCustomUrlError('Please enter a valid GGUF URL (must end with .gguf)');
      return;
    }
    setCustomUrlError(null);

    let filename = customGgufFilename.trim();
    if (!filename) {
      try {
        const parts = url.split('/');
        filename = parts[parts.length - 1] || 'custom-model.gguf';
        if (!filename.endsWith('.gguf')) filename += '.gguf';
      } catch {
        filename = 'custom-model.gguf';
      }
    }

    navigateTo('downloading');
    setDownloadProgress(0);
    setDownloadedBytes(0);
    setSelectedPreset({
      id: 'custom',
      name: filename,
      description: 'Custom GGUF Model weights',
      url,
      filename,
      sizeDesc: 'Custom Size',
    });

    try {
      await downloadModel(url, filename);
      setDownloadProgress(100);
      await loadModels();
      await checkServerStatus();
      if (models.length > 0) {
        setSelectedLocalModel(models[0]);
        setStep('ready');
      }
    } catch (e) {
      console.error('Download failed:', e);
      setStep('error');
    }
  };

  const handleDownloadPlugin = async () => {
    navigateTo('plugin_downloading');
    setDownloadProgress(0);
    setDownloadedBytes(0);
    try {
      await downloadAndInstallPlugin();
      setDownloadProgress(100);
      setStep('bridge_test');
    } catch (e) {
      console.error('Plugin download failed:', e);
      setStep('error');
    }
  };

  const handleInstallLocalPlugin = async () => {
    try {
      await installLocalPlugin();
      navigateTo('bridge_test');
    } catch (e) {
      console.error('Local plugin install failed:', e);
      setStep('error');
    }
  };

  const handleTestBridge = async () => {
    setBridgeTesting(true);
    try {
      await connectBridge();
      const status = useConnectionStore.getState().status;
      setBridgeTestResult(status === 'connected' ? 'success' : 'failed');
    } catch {
      setBridgeTestResult('failed');
    } finally {
      setBridgeTesting(false);
    }
  };

  const handleRefresh = async () => {
    await loadModels();
    await checkServerStatus();
  };

  const handleProceedToSdk = async () => {
    navigateTo('sdk_setup');
    setSdkChecking(true);
    try {
      const configuredPath = await invoke<string | null>('get_app_setting', {
        key: 'daz_sdk_path',
      });
      if (configuredPath) setSdkPath(configuredPath);
    } catch (e) {
      console.error('Failed to check SDK path:', e);
    } finally {
      setSdkChecking(false);
    }
  };

  const handleBrowseSdk = async () => {
    try {
      const folder = await invoke<string | null>('select_directory', {
        title: 'Select DAZStudio 4.5+ SDK Include Folder',
      });
      if (folder) {
        setSdkPath(folder);
        await invoke('save_app_setting', { key: 'daz_sdk_path', value: folder });
      }
    } catch (e) {
      console.error('Failed to browse for SDK:', e);
    }
  };

  const handleFinishSetup = async () => {
    if (isRunning) {
      navigateTo('ready_to_launch');
      return;
    }

    if (aiBackend === 'ollama') {
      navigateTo('ready_to_launch');
      return;
    }

    const model = selectedLocalModel || (models.length > 0 ? models[0] : null);
    if (!model) {
      navigateTo('ready_to_launch');
      return;
    }

    navigateTo('starting');
    try {
      const sep = modelsDir.includes('\\') ? '\\' : '/';
      const modelPath = `${modelsDir}${sep}${model.name}`;
      await startServer(modelPath);
      navigateTo('ready_to_launch');
    } catch (e) {
      console.error('Failed to start server:', e);
      setStep('error');
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const currentStepIndex = getStepIndex(step);

  const renderStepIndicator = () => {
    if (currentStepIndex < 0) return null;
    return (
      <>
        <div className={styles.stepLabel}>
          Step {currentStepIndex + 1} of {WIZARD_STEPS.length}:{' '}
          {WIZARD_STEPS[currentStepIndex]?.label}
        </div>
        <div className={styles.stepIndicator}>
          {WIZARD_STEPS.map((s, i) => (
            <div
              key={s.key}
              className={`${styles.stepDot} ${
                i === currentStepIndex
                  ? styles.stepDotActive
                  : i < currentStepIndex
                    ? styles.stepDotCompleted
                    : ''
              }`}
            />
          ))}
        </div>
      </>
    );
  };

  const renderBackButton = () => {
    if (stepHistory.length === 0) return null;
    return (
      <div className={styles.navRow}>
        <Button onClick={goBack} variant="ghost" className={styles.backButton}>
          <ArrowLeft size={14} />
          Back
        </Button>
      </div>
    );
  };

  const renderChecking = () => (
    <div className={styles.centered}>
      <Loader2 className={styles.spinner} size={48} />
      <Text variant="body" className={styles.statusText}>
        Checking your setup...
      </Text>
    </div>
  );

  const renderAiBackendChoice = () => (
    <VStack gap="lg" className={styles.centered}>
      <Cpu size={64} className="text-cyan-400 animate-pulse mb-2" />
      <Text variant="heading1">Welcome to DazPilot</Text>
      <Text variant="body" className={styles.description}>
        Choose your AI backend. This powers the natural-language scene control.
      </Text>

      <div className={styles.backendChoiceGrid}>
        <div
          role="button"
          tabIndex={0}
          onClick={() => handleSelectBackend('local')}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') handleSelectBackend('local');
          }}
          className={`${styles.backendCard} ${styles.backendCardSelected}`}
        >
          <Cpu size={32} className={styles.backendIcon} />
          <span className={styles.backendTitle}>Local GGUF</span>
          <span className={styles.backendDesc}>Runs on your machine. No internet needed.</span>
          <span className={styles.recommendedBadge}>Recommended</span>
        </div>

        <div
          role="button"
          tabIndex={0}
          onClick={() => handleSelectBackend('ollama')}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') handleSelectBackend('ollama');
          }}
          className={`${styles.backendCard} ${!ollamaRunning ? styles.backendCardDisabled : ''}`}
        >
          <Zap size={32} className={styles.backendIcon} />
          <span className={styles.backendTitle}>Ollama</span>
          <span className={styles.backendDesc}>Use Ollama if you have it installed.</span>
          <span
            className={ollamaRunning ? styles.backendStatusOnline : styles.backendStatusOffline}
          >
            {ollamaRunning ? 'Detected' : 'Not detected'}
          </span>
        </div>
      </div>

      <Text variant="small" className="text-slate-500" style={{ marginTop: '8px' }}>
        You can change this later in Settings.
      </Text>
    </VStack>
  );

  const renderOllamaSetup = () => (
    <VStack gap="lg" className={styles.centered}>
      <Zap size={64} className="text-cyan-400 animate-pulse mb-2" />
      <Text variant="heading2">Ollama Setup</Text>
      <Text variant="body" className={styles.description}>
        {ollamaRunning
          ? 'Select a model or pull a new one.'
          : 'Ollama does not appear to be running. Start Ollama and try again.'}
      </Text>

      {ollamaRunning && (
        <div className={styles.modelCard}>
          <Text variant="heading3" className={styles.downloadTitle}>
            Available Models
          </Text>
          {ollamaModels.length > 0 ? (
            <VStack gap="sm" className={styles.readyList}>
              {ollamaModels.map((model) => (
                <div
                  key={model.name}
                  role="button"
                  tabIndex={0}
                  onClick={() => setSelectedOllamaModel(model.name)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') setSelectedOllamaModel(model.name);
                  }}
                  className={`${styles.readyItem} ${
                    selectedOllamaModel === model.name ? styles.readyItemSelected : ''
                  }`}
                >
                  <span className={styles.readyName}>{model.name}</span>
                  <span className={styles.readySize}>{Math.round(model.size / 1024 / 1024)}MB</span>
                </div>
              ))}
            </VStack>
          ) : (
            <Text variant="small" className="text-slate-500">
              No models found. Pull one below.
            </Text>
          )}

          <div className={styles.customSection}>
            <span className={styles.customLabel}>Pull a model</span>
            <div style={{ display: 'flex', gap: '8px' }}>
              <input
                type="text"
                placeholder="e.g. llama3, phi3, mistral"
                value={ollamaPullName}
                onChange={(e) => setOllamaPullName(e.target.value)}
                className={styles.input}
              />
              <Button
                onClick={handleOllamaPull}
                variant="ghost"
                disabled={!ollamaPullName.trim() || ollamaLoading}
                style={{ flexShrink: 0 }}
              >
                {ollamaLoading ? (
                  <Loader2 className={styles.spinner} size={14} />
                ) : (
                  <Download size={14} />
                )}
                Pull
              </Button>
            </div>
          </div>
        </div>
      )}

      <Button
        onClick={handleOllamaContinue}
        variant="primary"
        className={styles.actionButton}
        disabled={ollamaRunning && ollamaModels.length > 0 && !selectedOllamaModel}
      >
        {ollamaRunning ? 'Next: Daz Plugin Setup' : 'Skip & Continue'}
      </Button>
      {renderBackButton()}
    </VStack>
  );

  const renderNoModel = () => (
    <VStack gap="lg" className={styles.centered}>
      <Cpu size={64} className="text-cyan-400 animate-pulse mb-2" />
      <Text variant="heading1">Download AI Model</Text>
      <Text variant="body" className={styles.description}>
        Pick a model to run locally on your machine.
      </Text>

      <div className={styles.modelCard}>
        <Text variant="heading3" className={styles.downloadTitle}>
          <Download size={20} />
          Preset Models
        </Text>

        <VStack gap="sm" className={styles.presetsList}>
          {MODEL_PRESETS.map((preset) => (
            <div
              key={preset.id}
              role="button"
              tabIndex={0}
              onClick={() => setSelectedPreset(preset)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') setSelectedPreset(preset);
              }}
              className={`${styles.presetItem} ${
                selectedPreset.id === preset.id ? styles.presetItemSelected : ''
              }`}
            >
              <div className={styles.presetHeader}>
                <span className={styles.presetName}>{preset.name}</span>
                <span className={styles.presetSize}>{preset.sizeDesc}</span>
              </div>
              <span className={styles.presetDesc}>{preset.description}</span>
            </div>
          ))}
        </VStack>
        <Button
          onClick={handleDownloadModel}
          variant="primary"
          className={styles.actionButton}
          disabled={isLoading}
        >
          {isLoading ? <Loader2 className={styles.spinner} size={20} /> : <Download size={18} />}
          Download & Setup
        </Button>

        <div className={styles.customSection}>
          <span className={styles.customLabel}>Or Download from Direct Hugging Face GGUF URL</span>
          <input
            type="text"
            placeholder="https://huggingface.co/username/repo/resolve/main/model.gguf"
            value={customGgufUrl}
            onChange={(e) => {
              setCustomGgufUrl(e.target.value);
              setCustomUrlError(null);
            }}
            className={customUrlError ? styles.inputError : styles.input}
          />
          {customUrlError && <span className={styles.errorMessage}>{customUrlError}</span>}
          <input
            type="text"
            placeholder="Custom Filename (e.g. my-model.gguf - optional)"
            value={customGgufFilename}
            onChange={(e) => setCustomGgufFilename(e.target.value)}
            className={styles.input}
          />
          <Button
            onClick={handleDownloadCustomModel}
            variant="ghost"
            className={styles.customDownloadButton}
            disabled={!customGgufUrl.trim() || isLoading}
          >
            <Download size={14} />
            Download Custom GGUF
          </Button>
        </div>

        <div className={styles.pathSection}>
          <span className={styles.pathLabel}>Or manually place a .gguf file in:</span>
          <code className={styles.pathCode}>{modelsDir}</code>
          <Button onClick={handleRefresh} variant="ghost" className="w-full mt-3">
            <RefreshCw size={16} />I Added a Model - Refresh
          </Button>
        </div>
      </div>
      {renderBackButton()}
    </VStack>
  );

  const renderDownloading = () => (
    <VStack gap="lg" className={styles.centered}>
      <Loader2 className={styles.spinner} size={64} />
      <Text variant="heading2">Downloading AI Model</Text>
      <Text variant="body" className="text-slate-400 text-center text-sm max-w-[320px]">
        Getting {selectedPreset.name} ready for you...
      </Text>
      <div className="w-full">
        <div className={styles.progressTrack} aria-label="Download progress">
          <div
            className={styles.progressFill}
            style={{ width: `${Math.min(downloadProgress, 100)}%` }}
          />
        </div>
        <span className={styles.progressText}>
          {downloadProgress === 0 && downloadedBytes > 0
            ? `${formatBytes(downloadedBytes)} downloaded...`
            : `${Math.round(downloadProgress)}% complete`}
        </span>
      </div>
      <Text variant="small" className="text-slate-500 font-semibold mt-2">
        {formatBytes(downloadedBytes)} downloaded
      </Text>
    </VStack>
  );

  const renderReady = () => (
    <VStack gap="lg" className={styles.centered}>
      <Check
        size={48}
        className="text-emerald-400 bg-emerald-950/30 border border-emerald-500/20 rounded-full p-2.5 mb-2"
      />
      <Text variant="heading1">Ready to Start</Text>
      <Text variant="body" className={styles.description}>
        Found {models.length} model(s). Select one for the AI backend.
      </Text>

      <VStack gap="sm" className={styles.readyList}>
        {models.map((model) => (
          <div
            key={model.name}
            role="button"
            tabIndex={0}
            onClick={() => setSelectedLocalModel(model)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') setSelectedLocalModel(model);
            }}
            className={`${styles.readyItem} ${
              selectedLocalModel?.name === model.name ? styles.readyItemSelected : ''
            }`}
          >
            <span className={styles.readyName}>{model.name}</span>
            <span className={styles.readySize}>{Math.round(model.size_mb)}MB</span>
          </div>
        ))}
      </VStack>

      <Button
        onClick={() => navigateTo('plugin_setup')}
        variant="primary"
        className={styles.actionButton}
        disabled={!selectedLocalModel || isLoading}
      >
        Next: Daz Plugin Setup
      </Button>

      <Button onClick={handleRefresh} variant="ghost" className="w-full">
        <RefreshCw size={16} />
        Scan for More Models
      </Button>
      {renderBackButton()}
    </VStack>
  );

  const renderPluginSetup = () => {
    const isInstalled = pluginStatus === 'installed';
    const isChecking = pluginStatus === 'checking';
    const isDownloading = pluginStatus === 'downloading';

    return (
      <VStack gap="lg" className={styles.centered}>
        <Monitor size={64} className="text-cyan-400 animate-pulse mb-2" />
        <Text variant="heading2">Daz Studio C++ Bridge</Text>
        <Text variant="body" className={styles.description}>
          DazPilot needs a C++ Bridge Plugin in Daz Studio to synchronize the viewport and execute
          commands.
        </Text>

        <div className={styles.modelCard}>
          <div className={styles.downloadTitle}>
            <span>Daz Studio Plugins Folder</span>
          </div>

          <div className={styles.pluginFolderRow}>
            <code className={styles.pathCodeInline}>
              {pluginCustomPath || 'Default plugins directory...'}
            </code>
            <Button
              onClick={browseCustomPath}
              variant="ghost"
              size="sm"
              className={styles.browseButton}
            >
              Browse...
            </Button>
          </div>

          <div className={styles.statusRow}>
            <span className={styles.pathLabel}>Status:</span>
            {isChecking ? (
              <span className={styles.statusChecking}>
                <Loader2 className={styles.spinner} size={14} /> Checking plugins folder...
              </span>
            ) : isInstalled ? (
              <span className={styles.statusInstalled}>
                <Check size={14} /> DazPilotBridge.dll linked successfully!
              </span>
            ) : (
              <span className={styles.statusNotInstalled}>
                <AlertCircle size={14} /> DazPilotBridge.dll not found.
              </span>
            )}
          </div>

          {!isInstalled && !isChecking && (
            <VStack gap="xs" className={styles.pluginActions}>
              <Button
                onClick={handleDownloadPlugin}
                variant="primary"
                className={styles.actionButton}
                disabled={isDownloading}
              >
                <Download size={16} />
                Download & Install from Releases
              </Button>
              <Button
                onClick={handleInstallLocalPlugin}
                variant="ghost"
                className={styles.actionButton}
                style={{ border: '1px solid rgba(255,255,255,0.05)' }}
                disabled={isDownloading}
              >
                Link / Copy Local DLL
              </Button>
              <div className={styles.dllInstructions}>
                <strong>How to find the DLL:</strong>
                <br />
                The bundled DLL is at <code>resources/DazPilotBridge.dll</code> inside the app.
                <br />
                Or build it: <code>npm run plugin:rebuild</code>
                <br />
                Copy to: <code>C:\Program Files\DAZ 3D\DAZStudio4\plugins\</code>
              </div>
            </VStack>
          )}

          {isInstalled && (
            <Text
              variant="small"
              className="text-slate-400 text-center"
              style={{ padding: '6px 0' }}
            >
              Bridge plugin linked successfully!
            </Text>
          )}

          <div className={styles.pluginFooter}>
            <Button
              onClick={isInstalled ? () => navigateTo('bridge_test') : handleProceedToSdk}
              variant="primary"
              className={styles.actionButton}
              disabled={isChecking}
            >
              {isInstalled ? 'Next: Test Bridge' : 'Skip & Continue to SDK'}
            </Button>
          </div>
        </div>
        {renderBackButton()}
      </VStack>
    );
  };

  const renderPluginDownloading = () => (
    <VStack gap="lg" className={styles.centered}>
      <Loader2 className={styles.spinner} size={64} />
      <Text variant="heading2">Downloading C++ Bridge Plugin</Text>
      <Text variant="body" className="text-slate-400 text-center text-sm max-w-[320px]">
        Downloading DazPilotBridge.dll from GitHub Releases...
      </Text>
      <div className="w-full">
        <div className={styles.progressTrack} aria-label="Download progress">
          <div
            className={styles.progressFill}
            style={{ width: `${Math.min(downloadProgress, 100)}%` }}
          />
        </div>
        <span className={styles.progressText}>
          {downloadProgress === 0 && downloadedBytes > 0
            ? `${formatBytes(downloadedBytes)} downloaded...`
            : `${Math.round(downloadProgress)}% complete`}
        </span>
      </div>
      <Text variant="small" className="text-slate-500 font-semibold mt-2">
        {formatBytes(downloadedBytes)} downloaded
      </Text>
    </VStack>
  );

  const renderBridgeTest = () => (
    <VStack gap="lg" className={styles.centered}>
      <Zap size={64} className="text-cyan-400 animate-pulse mb-2" />
      <Text variant="heading2">Test Bridge Connection</Text>
      <Text variant="body" className={styles.description}>
        Verify the bridge plugin can communicate with DazPilot.
      </Text>

      {bridgeTestResult === null ? (
        <div className={styles.modelCard} style={{ textAlign: 'center' }}>
          {bridgeTesting ? (
            <>
              <Loader2 className={styles.spinner} size={32} style={{ margin: '0 auto' }} />
              <Text variant="body" className={styles.statusText}>
                Testing connection to 127.0.0.1:8765...
              </Text>
            </>
          ) : (
            <>
              <Text variant="body" className="text-slate-400" style={{ marginBottom: '12px' }}>
                Click to test if Daz Studio is running with the bridge plugin loaded.
              </Text>
              <Button onClick={handleTestBridge} variant="primary" className={styles.actionButton}>
                <Zap size={16} />
                Test Connection
              </Button>
              <Text variant="small" className="text-slate-500" style={{ marginTop: '8px' }}>
                Make sure Daz Studio is running with the DazPilotBridge plugin loaded.
              </Text>
            </>
          )}
        </div>
      ) : bridgeTestResult === 'success' ? (
        <div className={styles.bridgeTestSuccess}>
          <Check size={32} className="text-emerald-400" style={{ margin: '0 auto 8px' }} />
          <Text variant="body" className="text-emerald-400 font-semibold">
            Connected to Daz Studio!
          </Text>
          <Text variant="small" className="text-slate-400" style={{ marginTop: '4px' }}>
            Bridge plugin is working correctly.
          </Text>
        </div>
      ) : (
        <div className={styles.bridgeTestFailed}>
          <AlertCircle size={32} className="text-amber-400" style={{ margin: '0 auto 8px' }} />
          <Text variant="body" className="text-amber-400 font-semibold">
            Could not connect
          </Text>
          <Text variant="small" className="text-slate-400" style={{ marginTop: '4px' }}>
            Bridge plugin is installed. Start Daz Studio to connect later.
          </Text>
        </div>
      )}

      <div className={styles.navRow}>
        <Button onClick={goBack} variant="ghost" className={styles.backButton}>
          <ArrowLeft size={14} />
          Back
        </Button>
        <Button onClick={handleProceedToSdk} variant="primary" className={styles.actionButton}>
          {bridgeTestResult === 'success' ? 'Next: SDK Setup' : 'Continue to SDK'}
        </Button>
      </div>
    </VStack>
  );

  const renderSdkSetup = () => (
    <VStack gap="lg" className={styles.centered}>
      <FolderOpen size={64} className="text-cyan-400 animate-pulse mb-2" />
      <Text variant="heading2">DAZStudio SDK Setup (Optional)</Text>
      <Text variant="body" className={styles.description}>
        For enhanced AI scripting intelligence, install the DAZStudio 4.5+ SDK via Daz Install
        Manager. This enables the AI to understand Daz Studio internal API and generate more
        accurate scripts.
      </Text>

      <div className={styles.modelCard}>
        <div className={styles.downloadTitle}>
          <FolderOpen size={20} />
          <span>SDK Location</span>
        </div>

        {sdkChecking ? (
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', padding: '12px 0' }}>
            <Loader2 className={styles.spinner} size={16} />
            <Text variant="body">Searching for SDK...</Text>
          </div>
        ) : sdkPath ? (
          <div style={{ padding: '8px 0' }}>
            <span className={styles.statusInstalled}>
              <Check size={14} /> SDK found!
            </span>
            <code className={styles.sdkPathCode}>{sdkPath}</code>
          </div>
        ) : (
          <div style={{ padding: '8px 0' }}>
            <span className={styles.statusChecking}>
              <AlertCircle size={14} /> SDK not found
            </span>
            <Text variant="small" className="text-slate-500" style={{ marginTop: '4px' }}>
              The AI will use basic scripting knowledge without the SDK.
            </Text>
          </div>
        )}

        <Button onClick={handleBrowseSdk} variant="ghost" className={styles.sdkButton}>
          <FolderOpen size={16} />
          Browse for SDK Include Folder...
        </Button>

        <div className={styles.sdkInstructions}>
          <strong>How to get the SDK:</strong>
          <ol>
            <li>Open Daz Install Manager (DIM)</li>
            <li>Search for DAZStudio 4.5+ SDK</li>
            <li>Install to the default location</li>
            <li>Click Browse above to select the SDK folder</li>
          </ol>
        </div>
      </div>

      <div className={styles.sdkFinishRow}>
        <Button onClick={handleFinishSetup} variant="primary" className={styles.actionButton}>
          {sdkPath ? 'Finish & Start AI Server' : 'Skip SDK & Start AI Server'}
        </Button>
      </div>
      {renderBackButton()}
    </VStack>
  );

  const renderStarting = () => (
    <div className={styles.centered}>
      <Loader2 className={styles.spinner} size={48} />
      <Text variant="body" className={styles.statusText}>
        Starting AI server with {selectedLocalModel?.name}...
      </Text>
    </div>
  );

  const renderReadyToLaunch = () => (
    <VStack gap="lg" className={styles.centered}>
      <Check
        size={64}
        className="text-emerald-400 bg-emerald-950/30 border border-emerald-500/20 rounded-full p-3 mb-2"
      />
      <Text variant="heading1">{"You're all set!"}</Text>
      <Text variant="body" className={styles.description}>
        {"DazPilot is ready. Here's what to do next:"}
      </Text>

      <div className={styles.postSetupCard}>
        <ul className={styles.postSetupList}>
          <li className={styles.postSetupItem}>
            <span className={styles.postSetupNumber}>1</span>
            <span>
              Open <strong>Daz Studio</strong> and load a scene
            </span>
          </li>
          <li className={styles.postSetupItem}>
            <span className={styles.postSetupNumber}>2</span>
            <span>
              Try a chat command: <strong>&quot;list all nodes in the scene&quot;</strong>
            </span>
          </li>
          <li className={styles.postSetupItem}>
            <span className={styles.postSetupNumber}>3</span>
            <span>
              Explore the <strong>Asset Browser</strong> to find your content
            </span>
          </li>
          <li className={styles.postSetupItem}>
            <span className={styles.postSetupNumber}>4</span>
            <span>
              Check <strong>Settings &gt; Connection</strong> to verify the bridge
            </span>
          </li>
        </ul>
      </div>

      <Button onClick={onComplete} variant="primary" className={styles.actionButton}>
        Enter Workspace
      </Button>
    </VStack>
  );

  const renderError = () => (
    <VStack gap="md" className={styles.centered}>
      <AlertCircle size={48} className="text-rose-500 mb-2" />
      <Text variant="heading2">Setup Failed</Text>
      <Text variant="body" className={styles.description}>
        {error || pluginError || 'An error occurred. Please try again.'}
      </Text>
      <Button
        onClick={() => {
          const last = stepHistory[stepHistory.length - 1];
          if (last) {
            goBack();
          } else {
            setStep('ai_backend_choice');
          }
        }}
        variant="primary"
        className="w-full max-w-[200px]"
      >
        Try Again
      </Button>
    </VStack>
  );

  const renderStep = () => {
    switch (step) {
      case 'checking':
        return renderChecking();
      case 'ai_backend_choice':
        return renderAiBackendChoice();
      case 'ollama_setup':
        return renderOllamaSetup();
      case 'no_model':
        return renderNoModel();
      case 'downloading':
        return renderDownloading();
      case 'ready':
        return renderReady();
      case 'plugin_setup':
        return renderPluginSetup();
      case 'plugin_downloading':
        return renderPluginDownloading();
      case 'bridge_test':
        return renderBridgeTest();
      case 'sdk_setup':
        return renderSdkSetup();
      case 'starting':
        return renderStarting();
      case 'ready_to_launch':
        return renderReadyToLaunch();
      case 'error':
        return renderError();
      default:
        return renderChecking();
    }
  };

  return (
    <div className={styles.overlay}>
      <Card className={styles.wizard}>
        <button type="button" className={styles.skipLink} onClick={onComplete}>
          Skip Setup
        </button>
        {renderStepIndicator()}
        {renderStep()}
      </Card>
    </div>
  );
}
