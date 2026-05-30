import type { ReactNode } from 'react';
import { cn } from '../../../utils/cn';
import styles from './PanelShell.module.css';

export interface PanelShellProps {
  className?: string;
  title?: string;
  header?: ReactNode;
  actions?: ReactNode;
  /** When false, panel sizes to content instead of filling parent */
  fill?: boolean;
  children: ReactNode;
}

export function PanelShell({
  className,
  title,
  header,
  actions,
  fill = true,
  children,
}: PanelShellProps) {
  const headerContent =
    header ??
    (title ? (
      <>
        <h2 className={styles.panelHeaderTitle}>{title}</h2>
        {actions ? <div className={styles.panelHeaderActions}>{actions}</div> : null}
      </>
    ) : null);

  return (
    <div className={cn(styles.panel, fill && styles.fill, className)}>
      {headerContent ? <div className={styles.panelHeader}>{headerContent}</div> : null}
      <div className={cn(styles.panelContent, 'scrollbar-thin')}>{children}</div>
    </div>
  );
}
