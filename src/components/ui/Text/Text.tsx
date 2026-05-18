import { forwardRef, type HTMLAttributes } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Text.module.css';

export type TextVariant = 'heading1' | 'heading2' | 'heading3' | 'body' | 'small' | 'muted';

export interface TextProps extends HTMLAttributes<HTMLParagraphElement> {
  variant?: TextVariant;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  truncate?: boolean;
}

export const Text = forwardRef<HTMLParagraphElement, TextProps>(
  (
    {
      variant = 'body',
      bold = false,
      italic = false,
      underline = false,
      truncate = false,
      className,
      children,
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
          bold && styles.bold,
          italic && styles.italic,
          underline && styles.underline,
          truncate && styles.truncate,
          className
        )}
        {...props}
      >
        {children}
      </p>
    );
  }
);

Text.displayName = 'Text';
