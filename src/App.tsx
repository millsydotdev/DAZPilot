import { useState, useEffect, useCallback } from 'react';
import {
  MessageSquare,
  FolderOpen,
  View,
  Layers,
  StickyNote,
  Settings,
  FileText,
} from 'lucide-react';
import { useHotkeys } from './hooks/useHotkey';
import {
  useAppStore,
  useConnectionStore,
  useLocalAiStore,
  useAssetsStore,
  usePluginStore,
} from './store';
import { ToastContainer } from './components/ui';
import ChatWindow from './components/chat/ChatWindow';
import AssetBrowser from './components/assets/AssetBrowser';
import ViewportCanvas from './components/viewport/ViewportCanvas';
import ScenePanel from './components/scene/ScenePanel';
import ScratchpadPanel from './components/scratchpad/ScratchpadPanel';
import SettingsPanel from './components/settings/SettingsPanel';
import PresetPanel from './components/preset/PresetPanel';
import { FirstLaunchWizard } from './components/FirstLaunchWizard';
import { ScriptApprovalPanel } from './components/ScriptApprovalPanel';
import { Launcher } from './components/Launcher';
import { AppLayout } from './components/layout/AppLayout';
import type { SidebarTab } from './components/layout/AppSidebar';
import type { AppTab } from './types/app';

const tabs: SidebarTab[] = [
  { id: 'chat', label: 'AI Chat', icon: <MessageSquare size={20} /> },
  { id: 'assets', label: 'Assets', icon: <FolderOpen size={20} /> },
  { id: 'viewport', label: 'Viewport', icon: <View size={20} /> },
  { id: 'scene', label: 'Scene', icon: <Layers size={20} /> },
  { id: 'scratchpad', label: 'Scratchpad', icon: <StickyNote size={20} /> },
  { id: 'presets', label: 'Presets', icon: <FileText size={20} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={20} /> },
];

function App() {
  const [appPhase, setAppPhase] = useState<'launcher' | 'main'>('launcher');
  const [activeTab, setActiveTab] = useState<AppTab>('chat');
  const {
    wizardCompleted: storeWizardCompleted,
    setWizardCompleted,
    sidebarCollapsed,
    toggleSidebar,
    loadAiSettings,
    theme,
  } = useAppStore();

  const loadSettings = useConnectionStore((state) => state.loadSettings);
  const getModelsDir = useLocalAiStore((state) => state.getModelsDir);
  const loadContentPaths = useAssetsStore((state) => state.loadContentPaths);
  const checkStatus = useConnectionStore((state) => state.checkStatus);
  const checkPluginStatus = usePluginStore((state) => state.checkPluginStatus);
  const pluginStatus = usePluginStore((state) => state.status);

  const [aiServerRunning, setAiServerRunning] = useState(false);
  const [dazStudioConnected, setDazStudioConnected] = useState(false);
  const pluginInstalled = pluginStatus === 'installed';

  useEffect(() => {
    loadSettings().catch(console.error);
    getModelsDir().catch(console.error);
    loadContentPaths().catch(console.error);
    loadAiSettings().catch(console.error);
    checkPluginStatus().catch(console.error);
  }, [loadSettings, getModelsDir, loadContentPaths, loadAiSettings, checkPluginStatus]);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  useEffect(() => {
    const checkAiStatus = async () => {
      try {
        const { useLocalAiStore: localAiStore } = await import('./store');
        const state = localAiStore.getState();
        setAiServerRunning(state.isRunning);
      } catch (e) {
        console.error('Failed to check AI status:', e);
      }
    };
    checkAiStatus();
    const interval = setInterval(checkAiStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const syncBridge = async () => {
      try {
        await checkStatus();
        const { status } = useConnectionStore.getState();
        setDazStudioConnected(status === 'connected');
      } catch {
        setDazStudioConnected(false);
      }
    };
    void syncBridge();
    const interval = setInterval(syncBridge, 5000);
    return () => clearInterval(interval);
  }, [checkStatus]);

  const handleLauncherComplete = useCallback((options?: { tab?: AppTab }) => {
    if (options?.tab) {
      setActiveTab(options.tab);
    }
    setAppPhase('main');
  }, []);

  const setTab = useCallback((tab: AppTab) => setActiveTab(tab), []);

  useHotkeys([
    { key: 'k', ctrl: true, handler: () => setTab('chat') },
    { key: 'a', ctrl: true, handler: () => setTab('assets') },
    { key: 'v', ctrl: true, handler: () => setTab('viewport') },
    { key: 's', ctrl: true, shift: true, handler: () => setTab('scene') },
    { key: 'p', ctrl: true, handler: () => setTab('scratchpad') },
    { key: ',', ctrl: true, handler: () => setTab('settings') },
    { key: 'b', ctrl: true, handler: toggleSidebar },
  ]);

  const wizardCompleted =
    storeWizardCompleted ||
    (typeof window !== 'undefined' &&
      (window.location.search.includes('skipModel') ||
        !(window as unknown as { __TAURI__?: unknown }).__TAURI__));

  const handleWizardComplete = () => {
    setWizardCompleted(true);
  };

  const renderContent = () => {
    switch (activeTab) {
      case 'chat':
        return <ChatWindow />;
      case 'assets':
        return <AssetBrowser />;
      case 'viewport':
        return <ViewportCanvas />;
      case 'scene':
        return <ScenePanel />;
      case 'scratchpad':
        return <ScratchpadPanel />;
      case 'presets':
        return <PresetPanel />;
      case 'settings':
        return <SettingsPanel />;
      default:
        return <ChatWindow />;
    }
  };

  if (appPhase === 'launcher') {
    return (
      <Launcher
        onComplete={handleLauncherComplete}
        aiServerRunning={aiServerRunning}
        pluginInstalled={pluginInstalled}
        dazStudioConnected={dazStudioConnected}
      />
    );
  }

  return (
    <>
      {!wizardCompleted && <FirstLaunchWizard onComplete={handleWizardComplete} />}
      <AppLayout
        tabs={tabs}
        activeTab={activeTab}
        sidebarCollapsed={sidebarCollapsed}
        onTabChange={setActiveTab}
        onToggleSidebar={toggleSidebar}
        onSceneRefresh={() => void checkStatus()}
      >
        {renderContent()}
      </AppLayout>
      <ToastContainer />
      <ScriptApprovalPanel />
    </>
  );
}

export default App;
