import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

export const API_URL = "http://localhost:8080"; // Rust backend

export async function fetcher<T>(url: string, options?: RequestInit): Promise<T> {
    const token = typeof window !== 'undefined' ? localStorage.getItem("token") : null;
    const headers = {
        "Content-Type": "application/json",
        ...options?.headers,
        ...(token ? { "Authorization": `Bearer ${token}` } : {})
    } as HeadersInit;

    const res = await fetch(`${API_URL}${url}`, { ...options, headers });
    const text = await res.text();
    if (!res.ok) {
        let message = text;
        try {
            const json = JSON.parse(text);
            if (json.error) {
                message = json.error;
            }
        } catch {}
        throw new Error(message || "API Error");
    }
    // Handle empty responses (like 200 OK with no body)
    return text ? JSON.parse(text) : {} as T;
}
