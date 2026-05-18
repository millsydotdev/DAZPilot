import { forwardRef, type HTMLAttributes } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Separator.module.css';

export type SeparatorSpacing = 'xs' | 'sm' | 'md' | 'lg';

export interface SeparatorProps extends HTMLAttributes<HTMLDivElement> {
  orientation?: 'horizontal' | 'vertical';
  dashed?: boolean;
  spacing?: SeparatorSpacing;
}

export const Separator = forwardRef<HTMLDivElement, SeparatorProps>(
  ({ orientation = 'horizontal', dashed = false, spacing, className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          styles.separator,
          orientation === 'vertical' && styles.vertical,
          dashed && styles.dashed,
          spacing && styles[`spacing-${spacing}`],
          className
        )}
        {...props}
      />
    );
  }
);

Separator.displayName = 'Separator';
