"use client"

import Link from "next/link"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { useState } from "react"
import { useRouter } from "next/navigation"
import { useLanguage } from "@/components/language-provider"

export default function Home() {
    const [electionId, setElectionId] = useState("")
    const router = useRouter()
    const { t } = useLanguage()

    return (
        <div className="flex min-h-screen flex-col bg-slate-50 dark:bg-slate-950">
            {/* Hero Section */}
            <section className="flex-1 flex flex-col items-center justify-center p-4 py-20 bg-gradient-to-b from-slate-50 to-slate-200 dark:from-slate-950 dark:to-slate-900 w-full">
                <div className="w-full max-w-4xl space-y-12 text-center">
                    <div className="space-y-4">
                        <h1 className="text-5xl font-extrabold tracking-tight lg:text-7xl text-primary drop-shadow-sm">SoleSigner</h1>
                        <p className="text-xl text-muted-foreground max-w-2xl mx-auto">{t("home.subtitle")}</p>
                    </div>

                    <div className="flex justify-center">
                        <Card className="w-full max-w-md shadow-lg border-2">
                            <CardHeader>
                                <CardTitle>{t("home.enterElection")}</CardTitle>
                                <CardDescription>{t("home.enteruuid")}</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <Input
                                    placeholder={t("home.placeholder")}
                                    value={electionId}
                                    onChange={(e) => setElectionId(e.target.value)}
                                    className="text-center text-lg h-12"
                                />
                                <Button
                                    className="w-full h-12 text-lg"
                                    onClick={() => router.push(`/vote/${electionId}`)}
                                    disabled={!electionId}
                                >
                                    {t("home.goVote")}
                                </Button>
                            </CardContent>
                        </Card>
                    </div>
                </div>
            </section>

            {/* Features / How it Works */}
            <section className="py-20 bg-white dark:bg-slate-950 w-full">
                <div className="container mx-auto px-4">
                    <h2 className="text-3xl font-bold text-center mb-12">{t("home.aboutTitle")}</h2>
                    <div className="grid md:grid-cols-3 gap-8">
                        <div className="text-center space-y-4 p-6 rounded-lg bg-slate-50 dark:bg-slate-900">
                            <div className="text-4xl">📸</div>
                            <h3 className="text-xl font-bold">{t("home.step1")}</h3>
                            <p className="text-muted-foreground">{t("home.step1Desc")}</p>
                        </div>
                        <div className="text-center space-y-4 p-6 rounded-lg bg-slate-50 dark:bg-slate-900">
                            <div className="text-4xl">🔐</div>
                            <h3 className="text-xl font-bold">{t("home.step2")}</h3>
                            <p className="text-muted-foreground">{t("home.step2Desc")}</p>
                        </div>
                        <div className="text-center space-y-4 p-6 rounded-lg bg-slate-50 dark:bg-slate-900">
                            <div className="text-4xl">🧾</div>
                            <h3 className="text-xl font-bold">{t("home.step3")}</h3>
                            <p className="text-muted-foreground">{t("home.step3Desc")}</p>
                        </div>
                    </div>
                </div>
            </section>

            {/* Footer / Admin Links */}
            <footer className="py-10 border-t w-full bg-slate-100 dark:bg-slate-900">
                <div className="container mx-auto px-4 flex flex-col md:flex-row justify-between items-center gap-4">
                    <div className="text-sm text-muted-foreground">
                        © 2024 SoleSigner. Open Source.
                    </div>
                    <div className="flex gap-4">
                        <Link href="/verify">
                            <Button variant="ghost">{t("home.audit")}</Button>
                        </Link>
                        <Link href="/login">
                            <Button variant="outline">{t("home.adminLogin")}</Button>
                        </Link>
                        <Link href="/register">
                            <Button variant="link">{t("home.register")}</Button>
                        </Link>
                    </div>
                </div>
            </footer>
        </div>
    )
}
