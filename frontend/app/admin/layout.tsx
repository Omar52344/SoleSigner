"use client"
import { useAuth } from "@/components/auth-provider"
import { useRouter } from "next/navigation"
import { useEffect } from "react"
import { Button } from "@/components/ui/button"

export default function AdminLayout({ children }: { children: React.ReactNode }) {
    const { token, isLoading, logout, user } = useAuth()
    const router = useRouter()

    useEffect(() => {
        if (!isLoading && !token) {
            router.push("/login")
        }
    }, [token, isLoading, router])

    if (isLoading) return <div>Loading...</div>

    if (!token) return null // Will redirect

    return (
        <div className="min-h-screen flex flex-col">
            <header className="border-b p-4 flex justify-between items-center bg-card">
                <div className="font-bold flex gap-4 items-center">
                    <span>SoleSigner Admin</span>
                    <span className="text-xs font-normal text-muted-foreground">Logged in as {user}</span>
                </div>
                <Button variant="outline" size="sm" onClick={logout} className="mr-28">Logout</Button>
            </header>
            <main className="flex-1">
                {children}
            </main>
        </div>
    )
}
