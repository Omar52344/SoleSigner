"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { useMutation } from "@tanstack/react-query"
import { API_URL } from "@/lib/utils"
import Link from "next/link"
import { useAuth } from "@/components/auth-provider"

export default function LoginPage() {
    const [username, setUsername] = useState("")
    const [password, setPassword] = useState("")
    const { login } = useAuth()

    const loginMutation = useMutation({
        mutationFn: async () => {
            const res = await fetch(`${API_URL}/auth/login`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username, password })
            })
            if (!res.ok) throw new Error(await res.text())
            return res.json()
        },
        onSuccess: (data) => {
            login(data.token, data.username)
        }
    })

    return (
        <div className="flex min-h-screen items-center justify-center bg-slate-50 dark:bg-slate-900 p-4">
            <Card className="w-full max-w-sm">
                <CardHeader>
                    <CardTitle className="text-2xl">Admin Login</CardTitle>
                    <CardDescription>Enter your credentials to access the dashboard</CardDescription>
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

                    {loginMutation.error && (
                        <div className="text-sm text-red-500">{loginMutation.error.message}</div>
                    )}

                    <Button
                        className="w-full"
                        onClick={() => loginMutation.mutate()}
                        disabled={loginMutation.isPending || !username || !password}
                    >
                        {loginMutation.isPending ? "Logging in..." : "Login"}
                    </Button>

                    <div className="text-center text-sm">
                        Don't have an account? <Link href="/register" className="text-primary underline">Register</Link>
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
