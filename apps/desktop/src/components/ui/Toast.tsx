import * as React from "react";
import { cn } from "./utils";
import { X } from "lucide-react";

export type ToastType = "default" | "success" | "warning" | "danger" | "info";

export interface ToastMessage {
  id: string;
  title?: string;
  description?: string;
  type?: ToastType;
}

interface ToastContextValue {
  addToast: (toast: Omit<ToastMessage, "id">) => void;
  removeToast: (id: string) => void;
}

const ToastContext = React.createContext<ToastContextValue | undefined>(undefined);

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = React.useState<ToastMessage[]>([]);

  const addToast = React.useCallback((toast: Omit<ToastMessage, "id">) => {
    const id = Math.random().toString(36).slice(2, 9);
    setToasts((prev) => [...prev, { ...toast, id }]);
    setTimeout(() => removeToast(id), 5000);
  }, []);

  const removeToast = React.useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ addToast, removeToast }}>
      {children}
      <div className="fixed bottom-4 right-4 z-50 flex max-h-screen flex-col-reverse gap-2">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={cn(
              "pointer-events-auto relative flex w-full max-w-sm flex-col gap-1 overflow-hidden rounded-[var(--radius-lg)] border p-4 pr-8 shadow-[var(--shadow-popover)] transition-all",
              {
                "bg-[var(--color-bg-elevated)] border-[var(--color-border)] text-[var(--color-fg)]": !t.type || t.type === "default",
                "bg-[var(--color-success)] border-transparent text-white": t.type === "success",
                "bg-[var(--color-danger)] border-transparent text-white": t.type === "danger",
                "bg-[var(--color-warning)] border-transparent text-white": t.type === "warning",
                "bg-[var(--color-info)] border-transparent text-white": t.type === "info",
              }
            )}
          >
            {t.title && <div className="text-[13px] font-semibold">{t.title}</div>}
            {t.description && <div className="text-[12px] opacity-90">{t.description}</div>}
            <button
              onClick={() => removeToast(t.id)}
              className="absolute right-2 top-2 rounded-md p-1 opacity-70 transition-opacity hover:opacity-100"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = React.useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return ctx;
}
