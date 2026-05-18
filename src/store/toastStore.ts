import { create } from 'zustand';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface ToastItem {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
  title?: string;
}

export interface ToastState {
  toasts: ToastItem[];
  addToast: (message: string, type?: ToastType, duration?: number, title?: string) => string;
  removeToast: (id: string) => void;
  success: (message: string, duration?: number, title?: string) => string;
  error: (message: string, duration?: number, title?: string) => string;
  info: (message: string, duration?: number, title?: string) => string;
  warning: (message: string, duration?: number, title?: string) => string;
}

export const useToastStore = create<ToastState>((set, get) => {
  const add = (message: string, type: ToastType = 'info', duration = 4000, title?: string) => {
    const id = Math.random().toString(36).substring(2, 9);
    set((state) => ({
      toasts: [...state.toasts, { id, message, type, duration, title }],
    }));
    if (duration > 0) {
      setTimeout(() => {
        get().removeToast(id);
      }, duration);
    }
    return id;
  };

  return {
    toasts: [],
    addToast: add,
    removeToast: (id) =>
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      })),
    success: (message, duration, title) => add(message, 'success', duration, title),
    error: (message, duration, title) => add(message, 'error', duration || 5000, title),
    info: (message, duration, title) => add(message, 'info', duration, title),
    warning: (message, duration, title) => add(message, 'warning', duration || 4500, title),
  };
});
