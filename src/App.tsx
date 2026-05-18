import { useState } from 'react';
import { MessageSquare, FolderOpen, View, Layers, StickyNote, Settings, ChevronLeft, ChevronRight } from 'lucide-react';
import { useAppStore } from './store';
import ChatWindow from './components/chat/ChatWindow';
import AssetBrowser from './components/assets/AssetBrowser';
import ViewportCanvas from './components/viewport/ViewportCanvas';
import ScenePanel from './components/scene/ScenePanel';
import ScratchpadPanel from './components/scratchpad/ScratchpadPanel';
import SettingsPanel from './components/settings/SettingsPanel';
import { FirstLaunchWizard } from './components/FirstLaunchWizard';

type Tab = 'chat' | 'assets' | 'viewport' | 'scene' | 'scratchpad' | 'settings';

interface TabInfo {
  id: Tab;
  label: string;
  icon: React.ReactNode;
}

const tabs: TabInfo[] = [
  { id: 'chat', label: 'AI Chat', icon: <MessageSquare size={20} /> },
  { id: 'assets', label: 'Assets', icon: <FolderOpen size={20} /> },
  { id: 'viewport', label: 'Viewport', icon: <View size={20} /> },
  { id: 'scene', label: 'Scene', icon: <Layers size={20} /> },
  { id: 'scratchpad', label: 'Scratchpad', icon: <StickyNote size={20} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={20} /> },
];

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('chat');
  const { 
    wizardCompleted: storeWizardCompleted, 
    setWizardCompleted,
    sidebarCollapsed,
    toggleSidebar
  } = useAppStore();

  const wizardCompleted = storeWizardCompleted || 
    (typeof window !== 'undefined' && (window.location.search.includes('skipModel') || !(window as unknown as { __TAURI__?: unknown }).__TAURI__));

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
      case 'settings':
        return <SettingsPanel />;
      default:
        return <ChatWindow />;
    }
  };

  return (
    <>
      {!wizardCompleted && <FirstLaunchWizard onComplete={handleWizardComplete} />}
      <div className="app-container">
      {/* Sidebar */}
      <aside className={`sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
        <div className="sidebar-header">
          {sidebarCollapsed ? (
            <div className="app-logo-compact">DP</div>
          ) : (
            <h1 className="app-title">DAZPilot</h1>
          )}
        </div>
        <nav className="sidebar-nav">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`nav-item ${activeTab === tab.id ? 'active' : ''}`}
              onClick={() => setActiveTab(tab.id)}
              title={sidebarCollapsed ? tab.label : undefined}
            >
              {tab.icon}
              <span className="nav-label">{tab.label}</span>
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <button 
            className="sidebar-toggle" 
            onClick={toggleSidebar}
            title={sidebarCollapsed ? "Expand Sidebar" : "Collapse Sidebar"}
          >
            {sidebarCollapsed ? <ChevronRight size={18} /> : <ChevronLeft size={18} />}
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="main-content">{renderContent()}</main>
    </div>
    </>
  );
}

export default App;
