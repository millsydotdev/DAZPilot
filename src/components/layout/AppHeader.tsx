import { RefreshCw, Wifi, WifiOff } from 'lucide-react';
import { cn } from '../../utils/cn';
import type { AppTab } from '../../types/app';
import { useConnectionStore } from '../../store';

const TAB_TITLES: Record<AppTab, string> = {
  chat: 'AI Co-Pilot',
  assets: 'Asset Library',
  viewport: 'Viewport',
  scene: 'Scene Control',
  scratchpad: 'Scratchpad',
  presets: 'Scene Presets',
  compose: 'Vibe Coding',
  tutorial: 'Tutorials',
  settings: 'Settings',
};

interface AppHeaderProps {
  activeTab: AppTab;
  className?: string;
  onRefresh?: () => void;
}

export function AppHeader({ activeTab, className, onRefresh }: AppHeaderProps) {
  const status = useConnectionStore((s) => s.status);
  const isConnecting = useConnectionStore((s) => s.isConnecting);
  const isConnected = status === 'connected';

  return (
    <header
      className={cn(
        'flex h-14 shrink-0 items-center justify-between border-b border-border-subtle',
        'bg-surface/80 px-6 backdrop-blur-md',
        className
      )}
    >
      <div>
        <h1 className="font-display text-lg font-semibold tracking-tight text-zinc-100">
          {TAB_TITLES[activeTab]}
        </h1>
        {activeTab === 'chat' && (
          <p className="font-mono text-[10px] uppercase tracking-wider text-zinc-500">
            Neural scene assistant
          </p>
        )}
      </div>

      <div className="flex items-center gap-3">
        {(activeTab === 'chat' || activeTab === 'scene') && (
          <div
            className={cn(
              'dp-badge gap-2',
              isConnected && 'border-cyan/30 text-cyan',
              isConnecting && 'border-amber-500/30 text-amber-400'
            )}
          >
            {isConnected ? (
              <>
                <Wifi size={12} />
                Bridge connected
              </>
            ) : isConnecting ? (
              <>
                <RefreshCw size={12} className="animate-spin" />
                Connecting…
              </>
            ) : (
              <>
                <WifiOff size={12} className="text-accent" />
                Bridge offline
              </>
            )}
          </div>
        )}

        {activeTab === 'scene' && onRefresh && (
          <button type="button" className="dp-btn-ghost px-3 py-1.5 text-xs" onClick={onRefresh}>
            <RefreshCw size={14} />
            Sync scene
          </button>
        )}
      </div>
    </header>
  );
}
