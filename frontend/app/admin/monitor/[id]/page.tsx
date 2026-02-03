"use client"

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
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

    const queryClient = useQueryClient()

    const startMutation = useMutation({
        mutationFn: async () => {
            await fetcher(`/elections/${electionId}/start`, { method: "POST" })
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['election-stats', electionId] })
    })

    const closeMutation = useMutation({
        mutationFn: async () => {
            await fetcher(`/elections/${electionId}/close`, { method: "POST" })
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['election-stats', electionId] })
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

                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">{t("common.actions")}</CardTitle>
                    </CardHeader>
                    <CardContent className="flex flex-col gap-2">
                        {stats?.status === 'DRAFT' && (
                            <Button
                                className="w-full bg-green-600 hover:bg-green-700"
                                onClick={() => {
                                    if (confirm(t("monitor.confirmStart"))) {
                                        startMutation.mutate()
                                    }
                                }}
                                disabled={startMutation.isPending}
                            >
                                {t("monitor.startElection")}
                            </Button>
                        )}
                        {stats?.status === 'OPEN' && (
                            <Button
                                className="w-full bg-red-600 hover:bg-red-700"
                                onClick={() => {
                                    if (confirm(t("monitor.confirmClose"))) {
                                        closeMutation.mutate()
                                    }
                                }}
                                disabled={closeMutation.isPending}
                            >
                                {t("monitor.closeElection")}
                            </Button>
                        )}
                        {stats?.status === 'SEALED' && (
                            <span className="text-muted-foreground text-sm italic">Election Closed</span>
                        )}
                        <Button
                            variant="outline"
                            className="w-full"
                            onClick={() => {
                                const link = `${window.location.origin}/vote/${electionId}`;
                                navigator.clipboard.writeText(link);
                                alert(t("monitor.linkCopied"));
                            }}
                        >
                            {t("monitor.copyLink")}
                        </Button>
                        <Button
                            variant="secondary"
                            className="w-full"
                            asChild
                        >
                            <Link href={`/results/${electionId}`} target="_blank">
                                {t("monitor.viewResults")}
                            </Link>
                        </Button>
                    </CardContent>
                </Card>
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
