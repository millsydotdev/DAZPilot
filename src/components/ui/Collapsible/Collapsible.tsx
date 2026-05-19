import { ReactNode, useState } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import styles from './Collapsible.module.css';

interface CollapsibleProps {
  title: string;
  icon?: ReactNode;
  defaultOpen?: boolean;
  children: ReactNode;
}

export function Collapsible({ title, icon, defaultOpen = true, children }: CollapsibleProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div className={styles.container}>
      <button className={styles.header} onClick={() => setIsOpen(!isOpen)} aria-expanded={isOpen}>
        {isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        {icon && <span className={styles.icon}>{icon}</span>}
        <span className={styles.title}>{title}</span>
      </button>
      {isOpen && <div className={styles.content}>{children}</div>}
    </div>
  );
}
