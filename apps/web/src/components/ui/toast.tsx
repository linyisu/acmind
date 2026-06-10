import { useToast } from "@/lib/stores/toast";
import type { ToastVariant } from "@/lib/stores/toast";
import { cn } from "@/lib/utils";
import { motion, AnimatePresence } from "motion/react";
import { X, CheckCircle2, AlertCircle, AlertTriangle, Info } from "lucide-react";

const variantStyles: Record<ToastVariant, string> = {
  default: "bg-card border-border text-foreground",
  success: "bg-emerald-950 border-emerald-700 text-emerald-100",
  error: "bg-red-950 border-red-700 text-red-100",
  warning: "bg-amber-950 border-amber-700 text-amber-100",
};

const variantIcons: Record<ToastVariant, typeof CheckCircle2> = {
  default: Info,
  success: CheckCircle2,
  error: AlertCircle,
  warning: AlertTriangle,
};

export function ToastContainer() {
  const toasts = useToast((s) => s.toasts);
  const removeToast = useToast((s) => s.removeToast);

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      <AnimatePresence>
        {toasts.map((t) => {
          const Icon = variantIcons[t.variant];
          return (
            <motion.div
              key={t.id}
              initial={{ opacity: 0, y: 12, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -8, scale: 0.95 }}
              transition={{ duration: 0.2 }}
              className={cn(
                "flex items-start gap-2 rounded-md border px-4 py-3 shadow-lg text-sm",
                variantStyles[t.variant],
              )}
            >
              <Icon className="h-4 w-4 mt-0.5 shrink-0" />
              <span className="flex-1">{t.message}</span>
              <button
                onClick={() => removeToast(t.id)}
                className="shrink-0 opacity-70 hover:opacity-100 transition-opacity"
              >
                <X className="h-4 w-4" />
              </button>
            </motion.div>
          );
        })}
      </AnimatePresence>
    </div>
  );
}
