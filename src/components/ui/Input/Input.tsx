import { forwardRef, type InputHTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './Input.module.css';

export type InputVariantSize = 'sm' | 'md' | 'lg';

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  variantSize?: InputVariantSize;
  error?: boolean;
  errorMessage?: string;
  label?: string;
  hint?: string;
  icon?: ReactNode;
  iconPosition?: 'left' | 'right';
  fullWidth?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      variantSize = 'md',
      error = false,
      errorMessage,
      label,
      hint,
      icon,
      iconPosition = 'left',
      fullWidth = false,
      className,
      id,
      ...props
    },
    ref
  ) => {
    const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

    return (
      <div className={cn(fullWidth && styles.fullWidth)}>
        {label && (
          <label htmlFor={inputId} className={styles.label}>
            {label}
          </label>
        )}
        <div
          className={cn(
            styles.inputWrapper,
            styles[variantSize],
            error && styles.error,
            icon && styles.hasIcon,
            icon && iconPosition === 'left' && styles.iconLeft,
            icon && iconPosition === 'right' && styles.iconRight
          )}
        >
          {icon && iconPosition === 'left' && <span className={styles.icon}>{icon}</span>}
          <input ref={ref} id={inputId} className={cn(styles.input, className)} {...props} />
          {icon && iconPosition === 'right' && <span className={styles.icon}>{icon}</span>}
        </div>
        {errorMessage && <span className={styles.errorMessage}>{errorMessage}</span>}
        {hint && !errorMessage && <span className={styles.hint}>{hint}</span>}
      </div>
    );
  }
);

Input.displayName = 'Input';
