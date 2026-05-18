import { useEffect, useState } from 'react';
import { X, CheckCircle2, AlertCircle, AlertTriangle, Info } from 'lucide-react';
import { ToastItem, useToastStore } from '../../../store/toastStore';
import styles from './Toast.module.css';

interface ToastProps {
  toast: ToastItem;
}

export function Toast({ toast }: ToastProps) {
  const removeToast = useToastStore((state) => state.removeToast);
  const [isExiting, setIsExiting] = useState(false);

  useEffect(() => {
    if (toast.duration && toast.duration > 0) {
      const exitTimer = setTimeout(() => {
        setIsExiting(true);
      }, toast.duration - 300); // Trigger fade-out animation 300ms before it is removed from store

      return () => clearTimeout(exitTimer);
    }
  }, [toast.duration]);

  const handleClose = () => {
    setIsExiting(true);
    setTimeout(() => {
      removeToast(toast.id);
    }, 250); // wait for fade animation
  };

  const getIcon = () => {
    switch (toast.type) {
      case 'success':
        return <CheckCircle2 className={styles.successIcon} size={18} />;
      case 'error':
        return <AlertCircle className={styles.errorIcon} size={18} />;
      case 'warning':
        return <AlertTriangle className={styles.warningIcon} size={18} />;
      default:
        return <Info className={styles.infoIcon} size={18} />;
    }
  };

  const getTitle = () => {
    if (toast.title) return toast.title;
    switch (toast.type) {
      case 'success':
        return 'Success';
      case 'error':
        return 'Error';
      case 'warning':
        return 'Warning';
      default:
        return 'Notification';
    }
  };

  return (
    <div className={`${styles.toast} ${styles[toast.type]} ${isExiting ? styles.exiting : ''}`}>
      <div className={styles.iconContainer}>{getIcon()}</div>
      <div className={styles.content}>
        <h4 className={styles.title}>{getTitle()}</h4>
        <p className={styles.message}>{toast.message}</p>
      </div>
      <button className={styles.closeButton} onClick={handleClose}>
        <X size={14} />
      </button>
      {toast.duration && toast.duration > 0 && (
        <div className={styles.progressBar} style={{ animationDuration: `${toast.duration}ms` }} />
      )}
    </div>
  );
}
