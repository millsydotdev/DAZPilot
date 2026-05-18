import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Grid.module.css';

export type GridCols = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;
export type GridGap = 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl';
export type GridSpan = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;
export type GridStart = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;

export interface GridProps extends HTMLAttributes<HTMLDivElement> {
  cols?: GridCols;
  gap?: GridGap;
  gapX?: GridGap;
  gapY?: GridGap;
  children: ReactNode;
}

export const Grid = forwardRef<HTMLDivElement, GridProps>(
  ({ cols = 1, gap, gapX, gapY, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          styles.grid,
          styles[`cols-${cols}`],
          gap && styles[`gap-${gap}`],
          gapX && styles[`gap-x-${gapX}`],
          gapY && styles[`gap-y-${gapY}`],
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Grid.displayName = 'Grid';

export interface GridItemProps extends HTMLAttributes<HTMLDivElement> {
  span?: GridSpan;
  start?: GridStart;
  children: ReactNode;
}

export const GridItem = forwardRef<HTMLDivElement, GridItemProps>(
  ({ span, start, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(span && styles[`span-${span}`], start && styles[`start-${start}`], className)}
        {...props}
      >
        {children}
      </div>
    );
  }
);

GridItem.displayName = 'GridItem';
