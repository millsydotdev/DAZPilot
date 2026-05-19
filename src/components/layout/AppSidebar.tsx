import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Logo, LogoCompact } from '../ui/Logo';
import { cn } from '../../utils/cn';
import type { AppTab } from '../../types/app';

export interface SidebarTab {
  id: AppTab;
  label: string;
  icon: React.ReactNode;
}

interface AppSidebarProps {
  tabs: SidebarTab[];
  activeTab: AppTab;
  collapsed: boolean;
  onTabChange: (tab: AppTab) => void;
  onToggleCollapse: () => void;
}

export function AppSidebar({
  tabs,
  activeTab,
  collapsed,
  onTabChange,
  onToggleCollapse,
}: AppSidebarProps) {
  return (
    <aside
      className={cn(
        'relative flex flex-col overflow-hidden border-r border-border-subtle bg-surface transition-[width] duration-200',
        collapsed ? 'w-[68px]' : 'w-60'
      )}
    >
      <div
        className="pointer-events-none absolute -left-36 -top-36 h-72 w-72 rounded-full"
        style={{
          background: 'radial-gradient(circle, rgba(239, 68, 68, 0.05) 0%, transparent 70%)',
        }}
      />

      <div
        className={cn(
          'flex h-[72px] shrink-0 items-center border-b border-border-subtle px-5',
          collapsed && 'justify-center px-0'
        )}
      >
        {collapsed ? <LogoCompact size={32} /> : <Logo size={40} />}
      </div>

      <nav className="flex flex-1 flex-col gap-1.5 p-3" aria-label="Main navigation">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            aria-current={activeTab === tab.id ? 'page' : undefined}
            title={collapsed ? tab.label : undefined}
            className={cn(
              activeTab === tab.id ? 'dp-nav-item-active' : 'dp-nav-item',
              collapsed && 'justify-center px-0'
            )}
            onClick={() => onTabChange(tab.id)}
          >
            {tab.icon}
            <span className={cn('transition-opacity duration-200', collapsed && 'sr-only')}>
              {tab.label}
            </span>
          </button>
        ))}
      </nav>

      <div className={cn('border-t border-border-subtle p-3', collapsed && 'flex justify-center')}>
        <button
          type="button"
          className="flex h-9 w-9 items-center justify-center rounded-md border border-border-subtle bg-white/[0.02] text-zinc-500 transition-colors hover:border-accent/30 hover:text-zinc-200 hover:shadow-glow-red-sm"
          onClick={onToggleCollapse}
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed ? <ChevronRight size={18} /> : <ChevronLeft size={18} />}
        </button>
      </div>
    </aside>
  );
}
