import { useState, useEffect } from 'react';
import { Cpu, Wifi, Info, Save, Plug, Unplug, Loader2 } from 'lucide-react';
import { useAppStore, useConnectionStore } from '../../store';
import { Button, Input, VStack, HStack, Card, CardHeader, CardContent } from '../ui';
import styles from './SettingsPanel.module.css';

type SettingsTab = 'general' | 'ai' | 'connection' | 'about';

const tabs = [
  { id: 'general' as const, label: 'General', icon: Save },
  { id: 'ai' as const, label: 'AI Settings', icon: Cpu },
  { id: 'connection' as const, label: 'Connection', icon: Wifi },
  { id: 'about' as const, label: 'About', icon: Info },
];

export default function SettingsPanel() {
  const { theme, setTheme, logLevel, setLogLevel } = useAppStore();
  const { status, aiModel, settings, isConnecting, connect, disconnect, setSettings } =
    useConnectionStore();

  const [activeTab, setActiveTab] = useState<SettingsTab>('general');

  useEffect(() => {
    const interval = setInterval(() => {
      useConnectionStore.getState().checkStatus();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleThemeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTheme(e.target.value as 'dark' | 'light');
  };

  const handleLogLevelChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setLogLevel(e.target.value as 'debug' | 'info' | 'warn' | 'error');
  };

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
              <tab.icon size={18} />
              <span>{tab.label}</span>
            </button>
          ))}
        </VStack>
      </div>

      <div className={styles.content}>
        {activeTab === 'general' && (
          <div className={styles.panel}>
            <h2 className={styles.title}>General Settings</h2>

            <div className={styles.group}>
              <label className={styles.label}>App Theme</label>
              <select className={styles.select} value={theme} onChange={handleThemeChange}>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>

            <div className={styles.group}>
              <label className={styles.label}>Log Level</label>
              <select className={styles.select} value={logLevel} onChange={handleLogLevelChange}>
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warning</option>
                <option value="error">Error</option>
              </select>
            </div>

            <Button onClick={() => {}}>Save Settings</Button>
          </div>
        )}

        {activeTab === 'ai' && (
          <div className={styles.panel}>
            <h2 className={styles.title}>AI Settings</h2>

            <Card>
              <CardHeader title="AI Status" />
              <CardContent>
                <div className={styles.statusRow}>
                  <span className={styles.statusLabel}>Status:</span>
                  <span
                    className={`${styles.statusValue} ${aiModel.loaded ? styles.ready : styles.loading}`}
                  >
                    {aiModel.loaded ? 'Ready' : 'Initializing...'}
                  </span>
                </div>
                <div className={styles.statusRow}>
                  <span className={styles.statusLabel}>Model:</span>
                  <span className={styles.statusValue}>{aiModel.name}</span>
                </div>
              </CardContent>
            </Card>

            <div className={styles.group}>
              <label className={styles.label}>Model Directory</label>
              <Input defaultValue="./resources/models/" placeholder="Path to models" />
            </div>

            <div className={styles.row}>
              <div className={styles.group}>
                <label className={styles.label}>Max Tokens</label>
                <Input type="number" defaultValue={2048} />
              </div>
              <div className={styles.group}>
                <label className={styles.label}>Temperature</label>
                <Input type="number" defaultValue={0.7} step={0.1} />
              </div>
            </div>

            <Button>Save AI Settings</Button>
          </div>
        )}

        {activeTab === 'connection' && (
          <div className={styles.panel}>
            <h2 className={styles.title}>Connection Settings</h2>

            <Card>
              <CardHeader title="Connection Status" />
              <CardContent>
                <HStack gap="md" align="center">
                  <div className={`${styles.indicator} ${getStatusIndicator()}`} />
                  <span>
                    {status === 'connected'
                      ? 'Connected to Daz3D'
                      : status === 'connecting'
                        ? 'Connecting...'
                        : 'Not Connected'}
                  </span>
                  {status === 'connected' ? (
                    <Button variant="danger" size="sm" onClick={disconnect}>
                      <Unplug size={16} />
                      Disconnect
                    </Button>
                  ) : (
                    <Button variant="primary" size="sm" onClick={connect} disabled={isConnecting}>
                      {isConnecting ? (
                        <Loader2 className={styles.spin} size={16} />
                      ) : (
                        <Plug size={16} />
                      )}
                      {isConnecting ? 'Connecting...' : 'Connect'}
                    </Button>
                  )}
                </HStack>
              </CardContent>
            </Card>

            <div className={styles.group}>
              <label className={styles.label}>Port</label>
              <Input
                type="number"
                value={settings.port}
                onChange={(e) => setSettings({ port: parseInt(e.target.value) })}
              />
            </div>

            <div className={styles.group}>
              <label className={styles.label}>
                <input
                  type="checkbox"
                  className={styles.checkboxInput}
                  checked={settings.autoConnect}
                  onChange={(e) => setSettings({ autoConnect: e.target.checked })}
                />
                Auto-connect on startup
              </label>
            </div>

            <div className={styles.helpBox}>
              <h4>Troubleshooting:</h4>
              <ul>
                <li>Make sure Daz3D Studio is running</li>
                <li>If connection fails, restart both apps</li>
                <li>The connection uses TCP port 8765</li>
              </ul>
            </div>

            <Button>Save Connection Settings</Button>
          </div>
        )}

        {activeTab === 'about' && (
          <div className={styles.panel}>
            <h2 className={styles.title}>About</h2>
            <p>DazPilot v0.1.0</p>
            <p>A UI tool for Daz3D workflow automation</p>
          </div>
        )}
      </div>
    </div>
  );
}
