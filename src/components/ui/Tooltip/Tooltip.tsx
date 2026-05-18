import { useState, createContext, useContext, type ReactNode, type HTMLAttributes } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Tooltip.module.css';

type TooltipSide = 'top' | 'bottom' | 'left' | 'right';

interface TooltipContextValue {
  side: TooltipSide;
  open: boolean;
}

const TooltipContext = createContext<TooltipContextValue | null>(null);

const useTooltipContext = () => {
  const context = useContext(TooltipContext);
  if (!context) {
    throw new Error('Tooltip components must be used within a Tooltip provider');
  }
  return context;
};

export interface TooltipProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  side?: TooltipSide;
}

export function Tooltip({ children, side = 'top', className, ...props }: TooltipProps) {
  const [open, setOpen] = useState(false);

  return (
    <TooltipContext.Provider value={{ side, open }}>
      <div
        className={cn(styles.root, className)}
        onMouseEnter={() => setOpen(true)}
        onMouseLeave={() => setOpen(false)}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        {...props}
      >
        {children}
      </div>
    </TooltipContext.Provider>
  );
}

export interface TooltipTriggerProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
}

export function TooltipTrigger({ children, className, ...props }: TooltipTriggerProps) {
  return (
    <span className={cn(className)} {...props}>
      {children}
    </span>
  );
}

export interface TooltipContentProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export function TooltipContent({ children, className, ...props }: TooltipContentProps) {
  const { side, open } = useTooltipContext();

  if (!open) return null;

  return (
    <div className={cn(styles.tooltip, className)} data-side={side} role="tooltip" {...props}>
      {children}
      <span className={styles.arrow} />
    </div>
  );
}
