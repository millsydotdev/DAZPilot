import { ReactNode } from 'react';
import { cn } from '../../../utils/cn';

export interface SidebarItem {
  id: string;
  label: string;
  icon: ReactNode;
}

export interface SidebarProps {
  items: SidebarItem[];
  activeId: string;
  onItemClick: (id: string) => void;
  className?: string;
}

export function Sidebar({ items, activeId, onItemClick, className }: SidebarProps) {
  return (
    <nav className={cn('flex flex-col gap-1', className)}>
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          onClick={() => onItemClick(item.id)}
          className={cn(
            'flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-all duration-200',
            activeId === item.id
              ? 'bg-accent-muted text-accent border border-accent/20'
              : 'text-zinc-500 hover:bg-white/5 hover:text-zinc-300'
          )}
        >
          {item.icon}
          {item.label}
        </button>
      ))}
    </nav>
  );
}
