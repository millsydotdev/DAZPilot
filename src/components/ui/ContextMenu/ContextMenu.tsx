import { useState, useRef, useEffect, ReactNode } from 'react';
import { cn } from '../../../utils/cn';

export interface ContextMenuProps {
  items: { label: string; onClick: () => void; icon?: ReactNode; danger?: boolean }[];
  children: ReactNode;
}

export function ContextMenu({ items, children }: ContextMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = () => setIsOpen(false);
    document.addEventListener('click', handleClick);
    return () => document.removeEventListener('click', handleClick);
  }, []);

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setPosition({ x: e.clientX, y: e.clientY });
    setIsOpen(true);
  };

  return (
    <div ref={containerRef} onContextMenu={handleContextMenu} className="inline-block w-full">
      {children}
      {isOpen && (
        <div
          className="fixed z-[9999] w-48 rounded-md border border-border-subtle bg-surface shadow-2xl animate-in fade-in zoom-in-95 duration-100"
          style={{ top: position.y, left: position.x }}
        >
          {items.map((item, i) => (
            <button
              key={i}
              className={cn(
                'flex w-full items-center gap-2 px-3 py-2 text-sm text-zinc-300 hover:bg-white/5',
                item.danger && 'text-accent'
              )}
              onClick={item.onClick}
            >
              {item.icon}
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
