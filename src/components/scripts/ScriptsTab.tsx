import { useState } from 'react';
import ScratchpadPanel from '../scratchpad/ScratchpadPanel';
import VibeCodingPanel from '../compose/VibeCodingPanel';
import ScriptEditor from '../scripting/ScriptEditor';
import { Tabs, Tab } from '../ui';

type ScriptsSubTab = 'scratchpad' | 'vibe' | 'editor';

export default function ScriptsTab() {
  const [activeTab, setActiveTab] = useState<ScriptsSubTab>('scratchpad');

  const tabs: Tab[] = [
    { id: 'scratchpad', label: 'Scratchpad' },
    { id: 'vibe', label: 'Vibe Coding' },
    { id: 'editor', label: 'Script Editor' },
  ];

  const handleTabChange = (id: string) => {
    if (id === 'scratchpad' || id === 'vibe' || id === 'editor') {
      setActiveTab(id);
    }
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <Tabs
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={handleTabChange}
        className="border-b border-border-subtle"
      />
      <div className="flex-1 overflow-hidden">
        {activeTab === 'scratchpad' && <ScratchpadPanel />}
        {activeTab === 'vibe' && <VibeCodingPanel />}
        {activeTab === 'editor' && <ScriptEditor />}
      </div>
    </div>
  );
}
