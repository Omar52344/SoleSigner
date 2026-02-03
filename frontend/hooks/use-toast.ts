"use client"
import { useEffect, useState } from "react"

const listeners: Array<(toasts: any[]) => void> = []
let memoryToasts: any[] = []

function notify() {
    listeners.forEach((listener) => listener([...memoryToasts]))
}

export function useToast() {
    const toast = ({ title, description, variant = "default" }: { title?: string, description?: string, variant?: "default" | "destructive" }) => {
        const id = Math.random().toString(36).substring(2, 9)
        const newToast = { id, title, description, variant }
        memoryToasts = [...memoryToasts, newToast]
        notify()

        // Auto dismiss after 3 seconds
        setTimeout(() => {
            memoryToasts = memoryToasts.filter((t) => t.id !== id)
            notify()
        }, 3000)
    }
    return { toast }
}

// Internal hook for the Toaster component
export function useToastState() {
    const [state, setState] = useState<any[]>(memoryToasts)

    useEffect(() => {
        listeners.push(setState)
        return () => {
            const index = listeners.indexOf(setState)
            if (index > -1) {
                listeners.splice(index, 1)
            }
        }
    }, [])

    return state
}
