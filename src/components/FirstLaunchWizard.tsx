import { useState, useEffect } from 'react';
import { Loader2, Cpu, Check, AlertCircle, RefreshCw, Download, FolderOpen } from 'lucide-react';
import { useLocalAiStore, type LocalModelInfo } from '../store/localAiStore';
import { usePluginStore } from '../store';
import { Button, Card, VStack, Text } from './ui';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import styles from './FirstLaunchWizard.module.css';

interface FirstLaunchWizardProps {
  onComplete: () => void;
}

type WizardStep =
  | 'checking'
  | 'no_model'
  | 'downloading'
  | 'ready'
  | 'plugin_setup'
  | 'plugin_downloading'
  | 'sdk_setup'
  | 'starting'
  | 'error';

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

interface DownloadProgressPayload {
  progress: number;
  total: number | null;
  downloaded: number;
}

export function FirstLaunchWizard({ onComplete }: FirstLaunchWizardProps) {
  const [stage, setStage] = useState<'model' | 'plugin'>('model');
  const [step, setStep] = useState<WizardStep>('checking');
  const [selectedLocalModel, setSelectedLocalModel] = useState<LocalModelInfo | null>(null);
  const [selectedPreset, setSelectedPreset] = useState<ModelPreset>(MODEL_PRESETS[0]);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadedBytes, setDownloadedBytes] = useState(0);
  const [customGgufUrl, setCustomGgufUrl] = useState('');
  const [customGgufFilename, setCustomGgufFilename] = useState('');
  const [sdkPath, setSdkPath] = useState<string | null>(null);
  const [sdkChecking, setSdkChecking] = useState(false);

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
    status: pluginStatus,
    customPath: pluginCustomPath,
    checkPluginStatus,
    browseCustomPath,
    downloadAndInstall: downloadAndInstallPlugin,
    installLocal: installLocalPlugin,
    error: pluginError,
  } = usePluginStore();

  useEffect(() => {
    const init = async () => {
      await getModelsDir();
      await loadModels();
      await checkServerStatus();
      await checkPluginStatus();
    };
    init();
  }, [checkServerStatus, getModelsDir, loadModels, checkPluginStatus]);

  useEffect(() => {
    if (stage === 'model') {
      if (isRunning) {
        setStage('plugin');
        setStep('plugin_setup');
      } else if (models.length > 0) {
        setStep('ready');
        setSelectedLocalModel(models[0]);
      } else if (!isLoading) {
        setStep('no_model');
      }
    }
  }, [isRunning, models, stage, isLoading]);

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

  const handleDownloadModel = async () => {
    setStep('downloading');
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

    let filename = customGgufFilename.trim();
    if (!filename) {
      try {
        const parts = url.split('/');
        filename = parts[parts.length - 1] || 'custom-model.gguf';
        if (!filename.endsWith('.gguf')) {
          filename += '.gguf';
        }
      } catch {
        filename = 'custom-model.gguf';
      }
    }

    setStep('downloading');
    setDownloadProgress(0);
    setDownloadedBytes(0);

    setSelectedPreset({
      id: 'custom',
      name: filename,
      description: 'Custom GGUF Model weights',
      url: url,
      filename: filename,
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
    setStep('plugin_downloading');
    setDownloadProgress(0);
    setDownloadedBytes(0);

    try {
      await downloadAndInstallPlugin();
      setDownloadProgress(100);
      setStep('plugin_setup');
    } catch (e) {
      console.error('Plugin download failed:', e);
      setStep('error');
    }
  };

  const handleInstallLocalPlugin = async () => {
    try {
      await installLocalPlugin();
    } catch (e) {
      console.error('Local plugin install failed:', e);
      setStep('error');
    }
  };

  const handleRefresh = async () => {
    await loadModels();
    await checkServerStatus();
  };

  const handleProceedToPlugin = () => {
    setStage('plugin');
    setStep('plugin_setup');
  };

  const handleProceedToSdk = async () => {
    setStep('sdk_setup');
    setSdkChecking(true);
    try {
      // Check if SDK is already configured
      const configuredPath = await invoke<string | null>('get_app_setting', {
        key: 'daz_sdk_path',
      });
      if (configuredPath) {
        setSdkPath(configuredPath);
      }
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
      onComplete();
      return;
    }

    const model = selectedLocalModel || (models.length > 0 ? models[0] : null);
    if (!model) {
      onComplete();
      return;
    }

    setStep('starting');
    try {
      const modelPath = `${modelsDir}\\${model.name}`;
      await startServer(modelPath);
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

  const renderChecking = () => (
    <div className={styles.centered}>
      <Loader2 className={styles.spinner} size={48} />
      <Text variant="body" className={styles.statusText}>
        Checking AI setup...
      </Text>
    </div>
  );

  const renderNoModel = () => (
    <VStack gap="lg" className={styles.centered}>
      <Cpu size={64} className="text-cyan-400 animate-pulse mb-2" />
      <Text variant="heading1">Welcome to DazPilot</Text>
      <Text variant="body" className={styles.description}>
        DazPilot is your AI co-pilot for Daz Studio — describe what you want and AI controls the
        scene for you.
      </Text>

      <div className={styles.modelCard}>
        <Text variant="heading3" className={styles.downloadTitle}>
          <Download size={20} />
          Download AI Model
        </Text>

        <VStack gap="sm" className={styles.presetsList}>
          {MODEL_PRESETS.map((preset) => (
            <div
              key={preset.id}
              onClick={() => setSelectedPreset(preset)}
              className={`${styles.presetItem} ${selectedPreset.id === preset.id ? styles.selected : ''}`}
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

        <div
          style={{
            borderTop: '1px solid rgba(255,255,255,0.05)',
            paddingTop: '16px',
            marginTop: '16px',
            display: 'flex',
            flexDirection: 'column',
            gap: '8px',
          }}
        >
          <span
            style={{
              fontSize: '11px',
              fontWeight: 600,
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
              color: 'var(--color-text-secondary)',
              display: 'block',
              marginBottom: '4px',
            }}
          >
            Or Download from Direct Hugging Face GGUF URL
          </span>
          <input
            type="text"
            placeholder="https://huggingface.co/username/repo/resolve/main/model.gguf"
            value={customGgufUrl}
            onChange={(e) => setCustomGgufUrl(e.target.value)}
            style={{
              width: '100%',
              padding: '8px 12px',
              fontSize: '12px',
              background: 'rgba(0,0,0,0.2)',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: 'var(--radius-sm)',
              color: '#fff',
              outline: 'none',
            }}
          />
          <input
            type="text"
            placeholder="Custom Filename (e.g. my-model.gguf - optional)"
            value={customGgufFilename}
            onChange={(e) => setCustomGgufFilename(e.target.value)}
            style={{
              width: '100%',
              padding: '8px 12px',
              fontSize: '12px',
              background: 'rgba(0,0,0,0.2)',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: 'var(--radius-sm)',
              color: '#fff',
              outline: 'none',
            }}
          />
          <Button
            onClick={handleDownloadCustomModel}
            variant="ghost"
            style={{
              border: '1px solid rgba(16,185,129,0.3)',
              color: '#10b981',
              display: 'flex',
              gap: '6px',
              justifyContent: 'center',
              alignItems: 'center',
            }}
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
            onClick={() => setSelectedLocalModel(model)}
            className={`${styles.readyItem} ${selectedLocalModel?.name === model.name ? styles.selected : ''}`}
          >
            <span className={styles.readyName}>{model.name}</span>
            <span className={styles.readySize}>{Math.round(model.size_mb)}MB</span>
          </div>
        ))}
      </VStack>

      <Button
        onClick={handleProceedToPlugin}
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
    </VStack>
  );

  const renderPluginSetup = () => {
    const isInstalled = pluginStatus === 'installed';
    const isChecking = pluginStatus === 'checking';
    const isDownloading = pluginStatus === 'downloading';

    return (
      <VStack gap="lg" className={styles.centered}>
        <Cpu size={64} className="text-cyan-400 animate-pulse mb-2" />
        <Text variant="heading2">Daz Studio C++ Bridge</Text>
        <Text variant="body" className={styles.description}>
          DazPilot needs a C++ Bridge Plugin in Daz Studio to synchronize the viewport and execute
          commands.
        </Text>

        <div className={styles.modelCard}>
          <div className={styles.downloadTitle}>
            <span>Daz Studio Plugins Folder</span>
          </div>

          <div style={{ display: 'flex', gap: '8px', width: '100%', alignItems: 'center' }}>
            <code
              className={styles.pathCode}
              style={{
                flexGrow: 1,
                textAlign: 'left',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {pluginCustomPath || 'Default plugins directory...'}
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

          <div style={{ marginTop: '8px' }}>
            <span className={styles.pathLabel}>Status:</span>
            {isChecking ? (
              <span
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  color: '#fbbf24',
                  fontSize: '13px',
                  fontWeight: 600,
                }}
              >
                <Loader2 className={styles.spinner} size={14} /> Checking plugins folder...
              </span>
            ) : isInstalled ? (
              <span
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  color: '#34d399',
                  fontSize: '13px',
                  fontWeight: 600,
                }}
              >
                <Check size={14} /> DazPilotBridge.dll linked successfully!
              </span>
            ) : (
              <span
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  color: '#f87171',
                  fontSize: '13px',
                  fontWeight: 600,
                }}
              >
                <AlertCircle size={14} /> DazPilotBridge.dll not found.
              </span>
            )}
          </div>

          {!isInstalled && !isChecking && (
            <VStack gap="xs" style={{ marginTop: '12px', width: '100%' }}>
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
            </VStack>
          )}

          {isInstalled && (
            <div
              style={{
                width: '100%',
                textAlign: 'center',
                padding: '6px 0',
                color: '#94a3b8',
                fontSize: '11px',
              }}
            >
              Bridge plugin linked successfully! Setup is fully complete.
            </div>
          )}

          <div
            style={{
              display: 'flex',
              gap: '8px',
              borderTop: '1px solid rgba(255,255,255,0.05)',
              paddingTop: '12px',
              marginTop: '12px',
              width: '100%',
            }}
          >
            <Button
              onClick={handleProceedToSdk}
              variant="primary"
              className={styles.actionButton}
              disabled={isChecking}
            >
              {isInstalled ? 'Next: SDK Setup' : 'Skip & Continue to SDK'}
            </Button>
          </div>
        </div>
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

  const renderSdkSetup = () => (
    <VStack gap="lg" className={styles.centered}>
      <Cpu size={64} className="text-cyan-400 animate-pulse mb-2" />
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
            <span
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                color: '#34d399',
                fontSize: '13px',
                fontWeight: 600,
              }}
            >
              <Check size={14} /> SDK found!
            </span>
            <code
              style={{
                display: 'block',
                marginTop: '8px',
                padding: '8px 12px',
                fontSize: '11px',
                background: 'rgba(0,0,0,0.2)',
                border: '1px solid rgba(255,255,255,0.1)',
                borderRadius: 'var(--radius-sm)',
                color: '#38bdf8',
                fontFamily: 'monospace',
                wordBreak: 'break-all',
              }}
            >
              {sdkPath}
            </code>
          </div>
        ) : (
          <div style={{ padding: '8px 0' }}>
            <span
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                color: '#fbbf24',
                fontSize: '13px',
                fontWeight: 600,
              }}
            >
              <AlertCircle size={14} /> SDK not found
            </span>
            <Text variant="small" className="text-slate-500" style={{ marginTop: '4px' }}>
              The AI will use basic scripting knowledge without the SDK.
            </Text>
          </div>
        )}

        <Button
          onClick={handleBrowseSdk}
          variant="ghost"
          style={{ width: '100%', marginTop: '8px', border: '1px solid rgba(255,255,255,0.1)' }}
        >
          <FolderOpen size={16} />
          Browse for SDK Include Folder...
        </Button>

        <div
          style={{
            borderTop: '1px solid rgba(255,255,255,0.05)',
            paddingTop: '12px',
            marginTop: '12px',
            fontSize: '11px',
            color: 'var(--color-text-muted)',
          }}
        >
          <strong>How to get the SDK:</strong>
          <ol style={{ margin: '8px 0 0 20px', lineHeight: '1.6' }}>
            <li>Open Daz Install Manager (DIM)</li>
            <li>Search for DAZStudio 4.5+ SDK</li>
            <li>Install to the default location</li>
            <li>Click Browse above to select the SDK folder</li>
          </ol>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '8px', width: '100%' }}>
        <Button onClick={handleFinishSetup} variant="primary" className={styles.actionButton}>
          {sdkPath ? 'Finish & Start AI Server' : 'Skip SDK & Start AI Server'}
        </Button>
      </div>
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

  const renderError = () => (
    <VStack gap="md" className={styles.centered}>
      <AlertCircle size={48} className="text-rose-500 mb-2" />
      <Text variant="heading2">Setup Failed</Text>
      <Text variant="body" className={styles.description}>
        {error || pluginError || 'An error occurred. Please try again.'}
      </Text>
      <Button
        onClick={() => setStep('no_model')}
        variant="primary"
        className="w-full max-w-[200px]"
      >
        Try Again
      </Button>
    </VStack>
  );

  return (
    <div className={styles.overlay}>
      <Card className={styles.wizard}>
        {step === 'checking' && renderChecking()}
        {step === 'no_model' && renderNoModel()}
        {step === 'downloading' && renderDownloading()}
        {step === 'ready' && renderReady()}
        {step === 'plugin_setup' && renderPluginSetup()}
        {step === 'plugin_downloading' && renderPluginDownloading()}
        {step === 'sdk_setup' && renderSdkSetup()}
        {step === 'starting' && renderStarting()}
        {step === 'error' && renderError()}
      </Card>
    </div>
  );
}
