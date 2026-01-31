"use client"

import { useQuery } from "@tanstack/react-query"
import { useParams } from "next/navigation"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { API_URL, fetcher } from "@/lib/utils"
import { useLanguage } from "@/components/language-provider"
import { Button } from "@/components/ui/button"
import Link from "next/link"

interface Stats {
    total_votes: number
    status: string
}

interface Election {
    id: string
    title: string
}

export default function MonitorPage() {
    const { id: electionId } = useParams()
    const { t } = useLanguage()

    // Poll stats every 5 seconds
    const { data: stats, isLoading: statsLoading } = useQuery<Stats>({
        queryKey: ['election-stats', electionId],
        queryFn: () => fetcher(`/elections/${electionId}/stats`),
        refetchInterval: 5000
    })

    const { data: election } = useQuery<Election>({
        queryKey: ['election', electionId],
        queryFn: () => fetcher(`/elections/${electionId}`)
    })

    if (statsLoading) return <div className="p-10 text-center">{t("common.loading")}</div>

    return (
        <div className="container mx-auto py-10 space-y-8">
            <div className="space-y-2">
                <Button variant="link" asChild className="p-0 text-muted-foreground">
                    <Link href="/admin/elections">&larr; {t("monitor.backToElections")}</Link>
                </Button>
                <h1 className="text-3xl font-bold">{t("monitor.dashboard")}: {election?.title}</h1>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">{t("monitor.totalVotes")}</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{stats?.total_votes || 0}</div>
                        <p className="text-xs text-muted-foreground">
                            {t("monitor.lastMinute")}
                        </p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">{t("common.status")}</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{stats?.status}</div>
                    </CardContent>
                </Card>
                {/* Placeholder for more stats */}
            </div>

            <Card className="col-span-4">
                <CardHeader>
                    <CardTitle>{t("monitor.liveActivity")}</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-[200px] flex items-center justify-center text-muted-foreground border-2 border-dashed rounded-md">
                        Graph Placeholder (Votes over time)
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
