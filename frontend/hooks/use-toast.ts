"use client"
// Simplified toast hook
import { useState } from "react"

export function useToast() {
    const toast = ({ title, description, variant }: any) => {
        console.log("Toast:", title, description);
        // In a real app, this updates a context
    }
    return { toast }
}
