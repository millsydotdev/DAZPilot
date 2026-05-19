import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { cn } from '../../utils/cn';

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
    </header>
  );
}
