"use client"

import { useQuery } from "@tanstack/react-query"
import { useParams } from "next/navigation"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { API_URL, fetcher } from "@/lib/utils"
import { useLanguage } from "@/components/language-provider"
import Link from "next/link"
import { Button } from "@/components/ui/button"

interface Results {
    [candidate: string]: number
}

interface Election {
    title: string
    status: string
}

export default function ResultsPage() {
    const { id: electionId } = useParams()
    const { t } = useLanguage()

    const { data: election } = useQuery<Election>({
        queryKey: ['election', electionId],
        queryFn: () => fetcher(`/elections/${electionId}`)
    })

    const { data: results, isLoading } = useQuery<Results>({
        queryKey: ['election-results', electionId],
        queryFn: () => fetcher(`/elections/${electionId}/results`)
    })

    if (isLoading) return <div className="p-10 text-center">{t("common.loading")}</div>

    // Check if sealed ? Or just show whatever?
    // User requirement: "pagina de resultados para usuarios"
    // Usually results are hidden until closed, but for this task I will show them.

    const sortedResults = results
        ? Object.entries(results).sort(([, a], [, b]) => b - a)
        : []

    const totalVotes = sortedResults.reduce((acc, [, count]) => acc + count, 0)

    return (
        <div className="container mx-auto py-10 space-y-8 max-w-2xl">
            <div className="space-y-2">
                <Button variant="link" asChild className="p-0 text-muted-foreground">
                    <Link href="/">&larr; Back to Home</Link>
                </Button>
                <h1 className="text-3xl font-bold">{election?.title} - Results</h1>
                <p className="text-muted-foreground">Status: {election?.status}</p>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>Vote Tally</CardTitle>
                    <CardDescription>Total Votes: {totalVotes}</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {sortedResults.length === 0 ? (
                        <div className="text-center text-muted-foreground">No votes yet.</div>
                    ) : (
                        sortedResults.map(([candidate, count]) => (
                            <div key={candidate} className="space-y-1">
                                <div className="flex justify-between text-sm font-medium">
                                    <span>{candidate}</span>
                                    <span>{count} ({totalVotes > 0 ? Math.round(count / totalVotes * 100) : 0}%)</span>
                                </div>
                                <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                                    <div
                                        className="h-full bg-primary"
                                        style={{ width: `${totalVotes > 0 ? (count / totalVotes) * 100 : 0}%` }}
                                    ></div>
                                </div>
                            </div>
                        ))
                    )}
                </CardContent>
            </Card>

            <div className="text-center">
                <Link href={`/verify`}>
                    <Button variant="outline">Verify Audit Trail</Button>
                </Link>
            </div>
        </div>
    )
}
