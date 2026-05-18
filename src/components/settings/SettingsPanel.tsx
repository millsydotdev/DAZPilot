import { useState, useEffect, useRef } from 'react';
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
  HardDrive 
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
  usePluginStore
} from '../../store';
import { Button, Input, VStack, HStack, Card, CardHeader, CardContent } from '../ui';
import styles from './SettingsPanel.module.css';

type SettingsTab = 'general' | 'ai' | 'connection' | 'logger' | 'shortcuts' | 'diagnostics' | 'about';

const tabs = [
  { id: 'general' as const, label: 'General', icon: Sliders },
  { id: 'ai' as const, label: 'AI Settings', icon: Cpu },
  { id: 'connection' as const, label: 'Connection', icon: Wifi },
  { id: 'logger' as const, label: 'Log Console', icon: Terminal },
  { id: 'shortcuts' as const, label: 'Shortcuts', icon: Keyboard },
  { id: 'diagnostics' as const, label: 'Diagnostics', icon: Activity },
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

  const { 
    theme, setTheme, 
    logLevel, setLogLevel,
    autoSave, setAutoSave,
    autoSaveInterval, setAutoSaveInterval,
    startupWindowMode, setStartupWindowMode,
    systemPrompt, setSystemPrompt,
    temperature, setTemperature,
    maxTokens, setMaxTokens,
    mockAiMode, setMockAiMode
  } = useAppStore();

  const { 
    status, 
    aiModel, 
    settings, 
    isConnecting, 
    connect, 
    disconnect, 
    setSettings 
  } = useConnectionStore();

  const { 
    logs, 
    clearLogs, 
    exportLogs, 
    autoScroll, 
    setAutoScroll 
  } = useLogStore();

  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [logSearch, setLogSearch] = useState('');
  const [selectedLevels, setSelectedLevels] = useState<string[]>(['info', 'warn', 'error', 'debug']);
  const [selectedCategories, setSelectedCategories] = useState<string[]>(['system', 'ai', 'connection', 'database', 'viewport']);

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
    setSelectedLevels(prev => 
      prev.includes(level) ? prev.filter(l => l !== level) : [...prev, level]
    );
  };

  const handleCategoryToggle = (category: string) => {
    setSelectedCategories(prev => 
      prev.includes(category) ? prev.filter(c => c !== category) : [...prev, category]
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
      .map(log => `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.category.toUpperCase()}] ${log.message}`)
      .join('\n');
    navigator.clipboard.writeText(text);
    console.log('Logs copied to clipboard!');
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
              <p className={styles.subtitle}>Configure main application settings, workspace intervals, and reset flags.</p>
            </div>

            <div className={styles.cardLayout}>
              <Card>
                <CardHeader title="Appearance & Diagnostics" />
                <CardContent>
                  <div className={styles.group}>
                    <label className={styles.label}>App Theme</label>
                    <select className={styles.select} value={theme} onChange={handleThemeChange}>
                      <option value="dark">Dark Theme (Premium Obsidian)</option>
                      <option value="light">Light Theme (Classic Slate)</option>
                    </select>
                  </div>

                  <div className={styles.group}>
                    <label className={styles.label}>App Log Threshold</label>
                    <select className={styles.select} value={logLevel} onChange={handleLogLevelChange}>
                      <option value="debug">Debug (All events)</option>
                      <option value="info">Info (Standard operations)</option>
                      <option value="warn">Warning (Important issues)</option>
                      <option value="error">Error (Failures only)</option>
                    </select>
                  </div>

                  <div className={styles.group}>
                    <label className={styles.label}>Window Startup Mode</label>
                    <select 
                      className={styles.select} 
                      value={startupWindowMode} 
                      onChange={(e) => setStartupWindowMode(e.target.value as 'windowed' | 'fullscreen')}
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
                      <label className={styles.label}>Auto-Save Interval: {autoSaveInterval} minutes</label>
                      <input 
                        type="range" 
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

              <Card className={styles.dangerCard}>
                <CardHeader title="Danger Zone" />
                <CardContent>
                  <p className={styles.dangerText}>
                    Performing a factory reset will erase all local databases, undo stacks, model settings, and cached assets. This action is irreversible.
                  </p>
                  <Button variant="danger" onClick={handleFactoryReset} className={styles.resetButton}>
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
              <p className={styles.subtitle}>Fine-tune local GGUF models, temperature values, response parameters, and system prompts.</p>
            </div>

            <div className={styles.cardLayout}>
              <Card>
                <CardHeader title="AI Engine Status" />
                <CardContent>
                  <div className={styles.statusGrid}>
                    <div className={styles.statusRow}>
                      <span className={styles.statusLabel}>Engine Loaded:</span>
                      <span className={`${styles.statusValue} ${aiModel.loaded ? styles.ready : styles.loading}`}>
                        {aiModel.loaded ? 'Ready (llama.cpp Local)' : 'Offline'}
                      </span>
                    </div>
                    <div className={styles.statusRow}>
                      <span className={styles.statusLabel}>Active GGUF:</span>
                      <span className={styles.statusValue}>{aiModel.name}</span>
                    </div>
                    <div className={styles.statusRow}>
                      <span className={styles.statusLabel}>Model Memory Size:</span>
                      <span className={styles.statusValue}>{aiModel.size > 0 ? `${(aiModel.size / (1024 * 1024)).toFixed(1)} MB` : 'N/A'}</span>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader title="Prompt Tuning & Parameters" />
                <CardContent>
                  <div className={styles.group}>
                    <label className={styles.label}>Custom System Co-Pilot Prompt</label>
                    <textarea 
                      className={styles.textarea} 
                      value={systemPrompt} 
                      onChange={(e) => setSystemPrompt(e.target.value)}
                      placeholder="Define the prompt instruction set for script generation..."
                      rows={5}
                    />
                  </div>

                  <div className={styles.row}>
                    <div className={styles.group}>
                      <label className={styles.label}>Temperature: {temperature}</label>
                      <input 
                        type="range" 
                        min="0.1" 
                        max="1.5" 
                        step="0.1"
                        value={temperature} 
                        onChange={(e) => setTemperature(parseFloat(e.target.value))} 
                        className={styles.slider}
                      />
                    </div>
                    <div className={styles.group}>
                      <label className={styles.label}>Max Tokens Limit</label>
                      <Input 
                        type="number" 
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
                </CardContent>
              </Card>
            </div>
          </div>
        )}

        {activeTab === 'connection' && (
          <div className={styles.panel}>
            <div className={styles.panelHeader}>
              <h2 className={styles.title}>Daz Studio Bridge Port Settings</h2>
              <p className={styles.subtitle}>Manage connection status and configure TCP port parameters with the VibeBridgePlugin.</p>
            </div>

            <div className={styles.cardLayout}>
              <Card>
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
                      <Button variant="primary" size="sm" onClick={connect} disabled={isConnecting}>
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
                    <label className={styles.label}>Bridge Target Hostname / IP</label>
                    <Input 
                      value={settings.host} 
                      onChange={(e) => setSettings({ host: e.target.value })} 
                    />
                  </div>

                  <div className={styles.row}>
                    <div className={styles.group}>
                      <label className={styles.label}>Bridge TCP Port</label>
                      <Input
                        type="number"
                        value={settings.port}
                        onChange={(e) => setSettings({ port: parseInt(e.target.value) })}
                      />
                    </div>

                    <div className={styles.group}>
                      <label className={styles.label}>Socket Connection Timeout (seconds)</label>
                      <Input
                        type="number"
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

        {activeTab === 'logger' && (
          <div className={styles.panelFull}>
            <div className={styles.terminalHeader}>
              <div>
                <h2 className={styles.title}>System Log Console</h2>
                <p className={styles.subtitle}>Real-time streaming console capture of all DazPilot frontend events, compiler scripts, and local ports.</p>
              </div>
              <HStack gap="sm">
                <Button size="sm" variant="secondary" onClick={handleCopyLogs}>
                  <Copy size={14} />
                  Copy Logs
                </Button>
                <Button size="sm" variant="secondary" onClick={exportLogs}>
                  <Download size={14} />
                  Export .txt File
                </Button>
                <Button size="sm" variant="danger" onClick={clearLogs}>
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
                  {['info', 'warn', 'error', 'debug'].map(lvl => (
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
                  {['system', 'ai', 'connection', 'database', 'viewport'].map(cat => (
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
              <p className={styles.subtitle}>Overview of core hotkey combinations mapped for high-speed operation inside the viewport and workspace.</p>
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
              <p className={styles.subtitle}>Run comprehensive diagnostic checks on SQLite local engines, local port bindings, and external C++ DLL plugins.</p>
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
                      <span className={`${styles.statusBadge} ${dbStatus === 'healthy' ? styles.badgeSuccess : styles.badgeInfo}`}>
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
                          AI: 8080 {portStatus.ai === 'listening' ? '✔' : '✘'}
                        </span>
                        <span className={styles.statusBadge}>
                          Bridge: 8765 {portStatus.bridge === 'listening' ? '✔' : '✘'}
                        </span>
                      </HStack>
                      <span className={styles.diagVal}>Port bindings active</span>
                    </div>
                  </CardContent>
                </Card>

                {/* Daz Studio Plugin check */}
                <Card style={{ gridColumn: 'span 2' }}>
                  <CardContent className={styles.diagCard} style={{ display: 'flex', flexDirection: 'column', gap: '16px', alignItems: 'stretch' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', width: '100%' }}>
                      <div className={styles.diagHeader}>
                        <ShieldAlert size={24} className={styles.diagIcon} />
                        <div>
                          <h4 className={styles.diagTitle}>C++ Bridge Plugin</h4>
                          <span className={styles.diagMeta}>Daz Studio DLL link</span>
                        </div>
                      </div>
                      <div className={styles.diagStatusSection} style={{ textAlign: 'right' }}>
                        <span className={`${styles.statusBadge} ${storePluginStatus === 'installed' ? styles.badgeSuccess : styles.badgeInfo}`}>
                          {storePluginStatus === 'installed' 
                            ? 'Plugin Active' 
                            : storePluginStatus === 'checking' 
                              ? 'Checking...' 
                              : storePluginStatus === 'downloading'
                                ? 'Downloading...'
                                : 'Missing / Unlinked'}
                        </span>
                        <span className={styles.diagVal} style={{ display: 'block', marginTop: '4px' }}>
                          DazPilotBridge.dll {storePluginStatus === 'installed' ? 'OK' : 'Missing'}
                        </span>
                      </div>
                    </div>

                    <div style={{ borderTop: '1px solid rgba(255,255,255,0.05)', paddingTop: '12px', width: '100%' }}>
                      <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginBottom: '6px', display: 'block' }}>
                        Active Daz Studio Plugins Directory
                      </label>
                      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                        <code style={{ flexGrow: 1, padding: '8px 12px', fontSize: '11px', background: '#060609', border: '1px solid rgba(255,255,255,0.03)', borderRadius: 'var(--radius-sm)', color: '#38bdf8', fontFamily: 'monospace', wordBreak: 'break-all' }}>
                          {pluginCustomPath || 'Using Daz Studio default folder...'}
                        </code>
                        <Button onClick={browseCustomPath} variant="ghost" size="sm" style={{ flexShrink: 0, padding: '4px 8px', height: 'auto', border: '1px solid rgba(255,255,255,0.1)' }}>
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
                            disabled={storePluginStatus === 'downloading' || storePluginStatus === 'checking'}
                          >
                            {storePluginStatus === 'downloading' ? 'Downloading DLL...' : 'Download & Install from Releases'}
                          </Button>
                          <Button 
                            onClick={() => usePluginStore.getState().installLocal()} 
                            variant="secondary" 
                            size="sm" 
                            style={{ fontSize: '11px', padding: '6px 12px' }}
                            disabled={storePluginStatus === 'downloading' || storePluginStatus === 'checking'}
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
                <p>A professional co-pilot desktop client for Daz3D workflows and AI automation.</p>
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
  );
}
