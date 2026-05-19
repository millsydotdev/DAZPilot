import { useState, useRef, useEffect, forwardRef, type SelectHTMLAttributes } from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '../../../utils/cn';

export interface SelectOption {
  label: string;
  value: string;
}

export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, 'onChange'> {
  label?: string;
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  fullWidth?: boolean;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, options, value, onChange, fullWidth = false, className }, _ref) => {
    const [isOpen, setIsOpen] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
      const handleClickOutside = (event: MouseEvent) => {
        if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
          setIsOpen(false);
        }
      };
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const selectedLabel = options.find((o) => o.value === value)?.label || 'Select an option';

    return (
      <div className={cn('relative', fullWidth && 'w-full', className)} ref={containerRef}>
        {label && <label className="mb-1.5 block text-xs font-medium text-zinc-400">{label}</label>}
        <button
          type="button"
          className="dp-input flex w-full items-center justify-between gap-2"
          onClick={() => setIsOpen(!isOpen)}
        >
          <span className="truncate">{selectedLabel}</span>
          <ChevronDown size={14} className={cn('transition-transform', isOpen && 'rotate-180')} />
        </button>
        {isOpen && (
          <div className="absolute top-full z-50 mt-1 w-full overflow-hidden rounded-md border border-border-subtle bg-surface shadow-xl animate-in fade-in zoom-in-95 duration-100">
            {options.map((option) => (
              <button
                key={option.value}
                type="button"
                className={cn(
                  'w-full px-3 py-2 text-left text-sm transition-colors hover:bg-white/5',
                  value === option.value ? 'text-accent' : 'text-zinc-300'
                )}
                onClick={() => {
                  onChange(option.value);
                  setIsOpen(false);
                }}
              >
                {option.label}
              </button>
            ))}
          </div>
        )}
      </div>
    );
  }
);

Select.displayName = 'Select';
