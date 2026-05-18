import { useToastStore } from '../../../store/toastStore';
import { Toast } from './Toast';
import styles from './Toast.module.css';

export function ToastContainer() {
  const toasts = useToastStore((state) => state.toasts);

  return (
    <div className={styles.container}>
      {toasts.map((toast) => (
        <Toast key={toast.id} toast={toast} />
      ))}
    </div>
  );
}
export default ToastContainer;
