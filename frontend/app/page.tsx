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
        <div className="flex min-h-screen flex-col items-center justify-center p-4 bg-gradient-to-b from-slate-50 to-slate-200 dark:from-slate-950 dark:to-slate-900">
            <div className="w-full max-w-md space-y-8">
                <div className="text-center space-y-2">
                    <h1 className="text-4xl font-extrabold tracking-tight lg:text-5xl text-primary">SoleSigner</h1>
                    <p className="text-muted-foreground">{t("home.subtitle")}</p>
                </div>

                <Card>
                    <CardHeader>
                        <CardTitle>{t("home.enterElection")}</CardTitle>
                        <CardDescription>{t("home.enteruuid")}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <Input
                            placeholder={t("home.placeholder")}
                            value={electionId}
                            onChange={(e) => setElectionId(e.target.value)}
                        />
                        <Button
                            className="w-full"
                            onClick={() => router.push(`/vote/${electionId}`)}
                            disabled={!electionId}
                        >
                            {t("home.goVote")}
                        </Button>
                    </CardContent>
                </Card>

                <div className="grid grid-cols-2 gap-4">
                    <Link href="/verify" className="w-full">
                        <Button variant="outline" className="w-full">{t("home.audit")}</Button>
                    </Link>
                    <Link href="/admin/create" className="w-full">
                        <Button variant="secondary" className="w-full">{t("home.admin")}</Button>
                    </Link>
                </div>
            </div>
        </div>
    )
}
