import { Settings } from 'lucide-react';
import { Logo } from '../ui/Logo';
import { cn } from '../../utils/cn';
import type { AppTab } from '../../types/app';
import { useConnectionStore } from '../../store';

export interface SidebarTab {
  id: AppTab;
  label: string;
  icon: React.ReactNode;
}

interface AppSidebarProps {
  tabs: SidebarTab[];
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
  onOpenSettings?: () => void;
}

export function AppSidebar({ tabs, activeTab, onTabChange, onOpenSettings }: AppSidebarProps) {
  const status = useConnectionStore((s) => s.status);
  const isConnected = status === 'connected';
  const isConnecting = status === 'connecting';

  return (
    <aside
      className={cn(
        'relative flex w-[48px] shrink-0 flex-col overflow-hidden border-r border-border-subtle bg-surface'
      )}
      aria-label="Main navigation"
    >
      <div className="flex h-[48px] shrink-0 items-center justify-center">
        <Logo size={32} />
      </div>

      <nav className="flex flex-1 flex-col gap-1 p-2">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            title={tab.label}
            aria-label={tab.label}
            aria-current={activeTab === tab.id ? 'page' : undefined}
            className={cn(
              'relative flex h-10 items-center justify-center rounded-md transition-colors',
              activeTab === tab.id
                ? 'border-l-2 border-accent bg-accent/5 text-accent'
                : 'border-l-2 border-transparent text-zinc-500 hover:border-accent/20 hover:bg-accent/5 hover:text-zinc-200'
            )}
            onClick={() => onTabChange(tab.id)}
          >
            {tab.icon}
          </button>
        ))}
      </nav>

      <div className="mt-auto flex flex-col items-center gap-2 border-t border-border-subtle p-2">
        <div
          className={cn(
            'flex h-8 w-8 items-center justify-center rounded-full',
            isConnected && 'ring-2 ring-cyan/50',
            isConnecting && 'ring-2 ring-amber-400/50'
          )}
          title={isConnected ? 'Bridge connected' : isConnecting ? 'Connecting…' : 'Bridge offline'}
          aria-label={
            isConnected ? 'Bridge connected' : isConnecting ? 'Connecting' : 'Bridge offline'
          }
        >
          <span
            className={cn(
              'h-2.5 w-2.5 rounded-full',
              isConnected && 'bg-cyan shadow-[0_0_8px_rgba(34,211,238,0.6)]',
              isConnecting && 'animate-pulse bg-amber-400',
              !isConnected && !isConnecting && 'bg-zinc-600'
            )}
          />
        </div>
        <button
          type="button"
          className="flex h-8 w-8 items-center justify-center rounded-md text-zinc-500 transition-colors hover:bg-white/[0.05] hover:text-zinc-200"
          onClick={onOpenSettings}
          aria-label="Settings"
          title="Settings"
        >
          <Settings size={16} />
        </button>
      </div>
    </aside>
  );
}
