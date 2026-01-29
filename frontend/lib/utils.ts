import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

export const API_URL = "http://localhost:8080"; // Rust backend

export async function fetcher<T>(url: string, options?: RequestInit): Promise<T> {
    const res = await fetch(`${API_URL}${url}`, options);
    if (!res.ok) {
        throw new Error(await res.text() || "API Error");
    }
    return res.json();
}
