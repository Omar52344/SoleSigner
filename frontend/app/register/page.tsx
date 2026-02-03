"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { useMutation } from "@tanstack/react-query"
import { API_URL } from "@/lib/utils"
import Link from "next/link"
import { useAuth } from "@/components/auth-provider"

export default function RegisterPage() {
    const [username, setUsername] = useState("")
    const [password, setPassword] = useState("")
    const router = useRouter()
    const { login } = useAuth()

    const registerMutation = useMutation({
        mutationFn: async () => {
            const res = await fetch(`${API_URL}/auth/register`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username, password })
            })
            if (!res.ok) throw new Error(await res.text())
            // return res.json()
        },
        onSuccess: () => {
            // Auto login after register? Or just redirect to login.
            // Let's implement auto-login logic if time permits, but for now redirect to login.
            router.push("/login")
        }
    })

    return (
        <div className="flex min-h-screen items-center justify-center bg-slate-50 dark:bg-slate-900 p-4">
            <Card className="w-full max-w-sm">
                <CardHeader>
                    <CardTitle className="text-2xl">Create Admin Account</CardTitle>
                    <CardDescription>Register to manage elections</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Username</label>
                        <Input value={username} onChange={(e) => setUsername(e.target.value)} />
                    </div>
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Password</label>
                        <Input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
                    </div>

                    {registerMutation.error && (
                        <div className="text-sm text-red-500">{registerMutation.error.message}</div>
                    )}

                    <Button
                        className="w-full"
                        onClick={() => registerMutation.mutate()}
                        disabled={registerMutation.isPending || !username || !password}
                    >
                        {registerMutation.isPending ? "Creating Account..." : "Register"}
                    </Button>

                    <div className="text-center text-sm">
                        Already have an account? <Link href="/login" className="text-primary underline">Login</Link>
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
