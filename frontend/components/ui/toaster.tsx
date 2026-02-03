"use client"
import { useToastState } from "@/hooks/use-toast"

export function Toaster() {
    const toasts = useToastState()

    if (toasts.length === 0) return null

    return (
        <div className="fixed top-4 right-4 z-[100] flex flex-col gap-2 w-full max-w-xs sm:right-4 sm:left-auto left-0 px-4 sm:px-0">
            {toasts.map((toast) => (
                <div
                    key={toast.id}
                    className={`
                        p-4 rounded-lg shadow-xl border transition-all animate-in slide-in-from-right-full fade-in duration-300
                        ${toast.variant === "destructive"
                            ? "bg-red-600 text-white border-red-700 shadow-red-900/20"
                            : "bg-white text-slate-900 border-slate-200 dark:bg-slate-800 dark:text-slate-50 dark:border-slate-700"}
                    `}
                >
                    {toast.title && <div className="font-semibold text-sm">{toast.title}</div>}
                    {toast.description && <div className="text-sm opacity-90 mt-1">{toast.description}</div>}
                </div>
            ))}
        </div>
    )
}
