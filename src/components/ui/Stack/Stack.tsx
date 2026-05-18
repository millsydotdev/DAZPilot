import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Stack.module.css';

export type StackGap = 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl';
export type StackAlign = 'start' | 'center' | 'end' | 'stretch';
export type StackJustify = 'start' | 'center' | 'end' | 'between' | 'around';
export type StackWrap = 'wrap' | 'nowrap';

export interface StackProps extends HTMLAttributes<HTMLDivElement> {
  gap?: StackGap;
  align?: StackAlign;
  justify?: StackJustify;
  wrap?: StackWrap;
  children: ReactNode;
}

export const Stack = forwardRef<HTMLDivElement, StackProps>(
  ({ gap = 'md', align, justify, wrap, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          styles.stack,
          styles[`gap-${gap}`],
          align && styles[`align-${align}`],
          justify && styles[`justify-${justify}`],
          wrap && styles[wrap],
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Stack.displayName = 'Stack';

export interface HStackProps extends Omit<StackProps, 'direction'> {
  children: ReactNode;
}

export const HStack = forwardRef<HTMLDivElement, HStackProps>((props, ref) => {
  return <Stack ref={ref} {...props} />;
});

HStack.displayName = 'HStack';

export interface VStackProps extends Omit<StackProps, 'direction'> {
  children: ReactNode;
}

export const VStack = forwardRef<HTMLDivElement, VStackProps>((props, ref) => {
  return <Stack ref={ref} {...props} />;
});

VStack.displayName = 'VStack';
