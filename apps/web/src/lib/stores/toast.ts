import { create } from "zustand";

export type ToastVariant = "default" | "success" | "error" | "warning";

export interface Toast {
  id: string;
  message: string;
  variant: ToastVariant;
}

interface ToastState {
  toasts: Toast[];
  addToast: (message: string, variant?: ToastVariant) => void;
  removeToast: (id: string) => void;
}

let counter = 0;

export const useToast = create<ToastState>((set) => ({
  toasts: [],
  addToast: (message, variant = "default") => {
    const id = `toast-${++counter}`;
    set((s) => ({ toasts: [...s.toasts, { id, message, variant }] }));
    setTimeout(() => {
      set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
    }, 4000);
  },
  removeToast: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

/** Shorthand helpers for use outside React components */
export const toast = (message: string) => useToast.getState().addToast(message, "default");
toast.success = (message: string) => useToast.getState().addToast(message, "success");
toast.error = (message: string) => useToast.getState().addToast(message, "error");
toast.warning = (message: string) => useToast.getState().addToast(message, "warning");
