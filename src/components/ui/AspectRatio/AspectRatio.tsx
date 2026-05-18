import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './AspectRatio.module.css';

export type AspectRatioRatio = '1:1' | '4:3' | '16:9' | '21:9' | '3:2' | '2:3' | '9:16';

export interface AspectRatioProps extends HTMLAttributes<HTMLDivElement> {
  ratio?: AspectRatioRatio;
  children: ReactNode;
}

export const AspectRatio = forwardRef<HTMLDivElement, AspectRatioProps>(
  ({ ratio = '16:9', children, className, ...props }, ref) => {
    const ratioClass = ratio.replace(':', '\\:');

    return (
      <div
        ref={ref}
        className={cn(styles.aspectRatio, styles[`ratio-${ratioClass}`], className)}
        {...props}
      >
        <div className={styles.content}>{children}</div>
      </div>
    );
  }
);

AspectRatio.displayName = 'AspectRatio';
