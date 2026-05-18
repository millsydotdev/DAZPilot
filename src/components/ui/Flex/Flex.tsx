import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Flex.module.css';

export type FlexDirection = 'row' | 'column' | 'row-reverse' | 'column-reverse';
export type FlexGap = 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl';
export type FlexAlign = 'start' | 'center' | 'end' | 'stretch' | 'baseline';
export type FlexJustify = 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';
export type FlexWrap = 'wrap' | 'nowrap' | 'wrap-reverse';

export interface FlexProps extends HTMLAttributes<HTMLDivElement> {
  inline?: boolean;
  direction?: FlexDirection;
  gap?: FlexGap;
  align?: FlexAlign;
  justify?: FlexJustify;
  wrap?: FlexWrap;
  grow?: boolean;
  shrink?: boolean | number;
  children: ReactNode;
}

export const Flex = forwardRef<HTMLDivElement, FlexProps>(
  (
    {
      inline = false,
      direction = 'row',
      gap,
      align,
      justify,
      wrap,
      grow = false,
      shrink,
      className,
      children,
      ...props
    },
    ref
  ) => {
    return (
      <div
        ref={ref}
        className={cn(
          styles.flex,
          inline && styles.inline,
          styles[direction],
          gap && styles[`gap-${gap}`],
          align && styles[`align-${align}`],
          justify && styles[`justify-${justify}`],
          wrap && styles[wrap],
          grow && styles.grow,
          shrink === 0 && styles['shrink-0'],
          shrink === false && styles['shrink-0'],
          typeof shrink === 'number' && shrink === 0 && styles['shrink-0'],
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Flex.displayName = 'Flex';
