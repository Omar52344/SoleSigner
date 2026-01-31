"use client"

import { useQuery } from "@tanstack/react-query"
import { useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table"
import { API_URL, fetcher } from "@/lib/utils"
import { useLanguage } from "@/components/language-provider"

interface Election {
    id: string
    title: string
    start_date: string
    end_date: string
    status: string
}

export default function AdminElectionsPage() {
    const router = useRouter()
    const { t } = useLanguage()

    const { data: elections, isLoading } = useQuery<Election[]>({
        queryKey: ['admin-elections'],
        queryFn: () => fetcher('/elections')
    })

    if (isLoading) return <div className="p-10 text-center">{t("common.loading")}</div>

    return (
        <div className="container mx-auto py-10 space-y-8">
            <div className="flex justify-between items-center">
                <h1 className="text-3xl font-bold">{t("nav.admin_elections")}</h1>
                <Button onClick={() => router.push('/admin/create')}>{t("nav.create")}</Button>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>{t("admin.elections_list")}</CardTitle>
                </CardHeader>
                <CardContent>
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>{t("create.electionTitle")}</TableHead>
                                <TableHead>{t("common.startDate")}</TableHead>
                                <TableHead>{t("common.status")}</TableHead>
                                <TableHead>{t("common.actions")}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {elections?.map((election) => (
                                <TableRow key={election.id}>
                                    <TableCell className="font-medium">{election.title}</TableCell>
                                    <TableCell>{new Date(election.start_date).toLocaleDateString()}</TableCell>
                                    <TableCell>{election.status}</TableCell>
                                    <TableCell className="flex gap-2">
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={() => router.push(`/admin/monitor/${election.id}`)}
                                        >
                                            {t("admin.list.monitor")}
                                        </Button>
                                        <Button
                                            variant="secondary"
                                            size="sm"
                                            onClick={() => router.push(`/admin/whitelist/${election.id}`)}
                                        >
                                            {t("admin.list.whitelist")}
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            ))}
                            {(!elections || elections.length === 0) && (
                                <TableRow>
                                    <TableCell colSpan={4} className="text-center py-10 text-muted-foreground">
                                        {t("admin.list.empty")}
                                    </TableCell>
                                </TableRow>
                            )}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </div>
    )
}
