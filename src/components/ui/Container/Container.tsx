import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Container.module.css';

export type ContainerSize = 'sm' | 'md' | 'lg' | 'xl' | 'full';

export interface ContainerProps extends HTMLAttributes<HTMLDivElement> {
  size?: ContainerSize;
  centered?: boolean;
  children: ReactNode;
}

export const Container = forwardRef<HTMLDivElement, ContainerProps>(
  ({ size = 'lg', centered = false, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(styles.container, styles[size], centered && styles.centered, className)}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Container.displayName = 'Container';
