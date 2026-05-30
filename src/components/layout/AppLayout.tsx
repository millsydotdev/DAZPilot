import type { ReactNode } from 'react';
import { TitleBar } from './TitleBar';
import { AppSidebar, type SidebarTab } from './AppSidebar';
import { StatusBar } from '../ui/StatusBar';
import type { AppTab } from '../../types/app';

interface AppLayoutProps {
  tabs: SidebarTab[];
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
  children: ReactNode;
}

export function AppLayout({ tabs, activeTab, onTabChange, children }: AppLayoutProps) {
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-void">
      <TitleBar />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <AppSidebar
          tabs={tabs}
          activeTab={activeTab}
          onTabChange={onTabChange}
          onOpenSettings={() => onTabChange('settings')}
        />
        <main className="flex min-w-0 flex-1 flex-col overflow-hidden bg-void">
          <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
          {activeTab === 'viewport' && <StatusBar />}
        </main>
      </div>
    </div>
  );
}
