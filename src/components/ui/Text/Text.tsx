import { forwardRef, type HTMLAttributes } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Text.module.css';

export type TextVariant = 'heading1' | 'heading2' | 'heading3' | 'body' | 'small' | 'muted';
export type TextSize = 'xs' | 'sm' | 'md' | 'lg';

export interface TextProps extends HTMLAttributes<HTMLParagraphElement> {
  variant?: TextVariant;
  size?: TextSize;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  truncate?: boolean;
  width?: string;
}

export const Text = forwardRef<HTMLParagraphElement, TextProps>(
  (
    {
      variant = 'body',
      size,
      bold = false,
      italic = false,
      underline = false,
      truncate = false,
      className,
      children,
      width,
      ...props
    },
    ref
  ) => {
    return (
      <p
        ref={ref}
        className={cn(
          styles.text,
          styles[variant],
          size && styles[size as keyof typeof styles],
          bold && styles.bold,
          italic && styles.italic,
          underline && styles.underline,
          truncate && styles.truncate,
          className
        )}
        style={width ? { width } : undefined}
        {...props}
      >
        {children}
      </p>
    );
  }
);

Text.displayName = 'Text';
