import type { ReactNode } from 'react';
import { TitleBar } from './TitleBar';
import { AppSidebar, type SidebarTab } from './AppSidebar';
import { AppHeader } from './AppHeader';
import { StatusBar } from '../ui/StatusBar';
import type { AppTab } from '../../types/app';

interface AppLayoutProps {
  tabs: SidebarTab[];
  activeTab: AppTab;
  sidebarCollapsed: boolean;
  onTabChange: (tab: AppTab) => void;
  onToggleSidebar: () => void;
  onSceneRefresh?: () => void;
  children: ReactNode;
}

export function AppLayout({
  tabs,
  activeTab,
  sidebarCollapsed,
  onTabChange,
  onToggleSidebar,
  onSceneRefresh,
  children,
}: AppLayoutProps) {
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-void">
      <TitleBar />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <AppSidebar
          tabs={tabs}
          activeTab={activeTab}
          collapsed={sidebarCollapsed}
          onTabChange={onTabChange}
          onToggleCollapse={onToggleSidebar}
        />
        <main className="flex min-w-0 flex-1 flex-col overflow-hidden bg-void">
          <AppHeader
            activeTab={activeTab}
            onRefresh={activeTab === 'scene' ? onSceneRefresh : undefined}
          />
          <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
          <StatusBar />
        </main>
      </div>
    </div>
  );
}
