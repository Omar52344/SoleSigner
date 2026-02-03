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
    if (!res.ok) {
        throw new Error(await res.text() || "API Error");
    }
    // Handle empty responses (like 200 OK with no body)
    const text = await res.text();
    return text ? JSON.parse(text) : {} as T;
}
