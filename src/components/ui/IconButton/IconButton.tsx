import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './IconButton.module.css';

export type IconButtonSize = 'sm' | 'md' | 'lg';
export type IconButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';

export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  size?: IconButtonSize;
  variant?: IconButtonVariant;
  icon: ReactNode;
  label: string;
  loading?: boolean;
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  (
    { size = 'md', variant = 'ghost', icon, label, loading = false, className, disabled, ...props },
    ref
  ) => {
    const isDisabled = disabled || loading;

    return (
      <button
        ref={ref}
        className={cn(
          styles.button,
          styles[size],
          styles[variant],
          loading && styles.loading,
          className
        )}
        disabled={isDisabled}
        aria-label={label}
        title={label}
        {...props}
      >
        {loading ? (
          <span className={styles.spinner} />
        ) : (
          <span className={styles.icon}>{icon}</span>
        )}
      </button>
    );
  }
);

IconButton.displayName = 'IconButton';
