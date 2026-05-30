import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { RefreshCw, Wifi, WifiOff } from 'lucide-react';
import { cn } from '../../utils/cn';
import { useConnectionStore } from '../../store';

interface TitleBarProps {
  className?: string;
}

export function TitleBar({ className }: TitleBarProps) {
  useEffect(() => {
    const applyTheme = async () => {
      try {
        const win = getCurrentWindow();
        await win.setBackgroundColor([10, 10, 10, 255]);
      } catch {
        // Browser dev — no Tauri window
      }
    };
    void applyTheme();
  }, []);

  const status = useConnectionStore((s) => s.status);
  const isConnecting = useConnectionStore((s) => s.isConnecting);
  const isConnected = status === 'connected';

  return (
    <header
      className={cn(
        'flex h-8 shrink-0 items-center border-b border-border-subtle bg-void px-3',
        'text-[11px] font-medium tracking-wide text-zinc-500',
        className
      )}
      data-tauri-drag-region
    >
      <span className="font-display text-zinc-400">
        Daz<span className="text-accent">Pilot</span>
      </span>
      <div className="ml-auto flex items-center gap-2">
        <div
          className={cn(
            'dp-badge gap-1.5',
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
      </div>
    </header>
  );
}
