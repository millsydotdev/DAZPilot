import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './ScrollArea.module.css';

export interface ScrollAreaProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  horizontal?: boolean;
  vertical?: boolean;
}

export const ScrollArea = forwardRef<HTMLDivElement, ScrollAreaProps>(
  ({ horizontal = true, vertical = true, className, children, ...props }, ref) => {
    const showHorizontal = horizontal;
    const showVertical = vertical;

    return (
      <div ref={ref} className={cn(styles.root, className)} {...props}>
        <div
          className={styles.viewport}
          data-state-visible={showVertical || showHorizontal ? 'true' : undefined}
        >
          {children}
        </div>
        {showVertical && (
          <div className={cn(styles.scrollbar, styles.scrollbarVertical)}>
            <div className={cn(styles.thumb, styles.thumbVertical)} />
          </div>
        )}
        {showHorizontal && (
          <div className={cn(styles.scrollbar, styles.scrollbarHorizontal)}>
            <div className={cn(styles.thumb, styles.thumbHorizontal)} />
          </div>
        )}
      </div>
    );
  }
);

ScrollArea.displayName = 'ScrollArea';
