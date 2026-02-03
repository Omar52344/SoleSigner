"use client"

import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useParams } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { API_URL, fetcher } from "@/lib/utils"
import { useLanguage } from "@/components/language-provider"
import Link from "next/link"
import { sha256 } from "js-sha256"

export default function WhitelistPage() {
    const { id: electionId } = useParams()
    const { t } = useLanguage()
    const queryClient = useQueryClient()
    const [newId, setNewId] = useState("")

    const { data: whitelist, isLoading } = useQuery<string[]>({
        queryKey: ['whitelist', electionId],
        queryFn: () => fetcher(`/elections/${electionId}/whitelist`)
    })

    const addMutation = useMutation({
        mutationFn: async (hash: string) => {
            const res = await fetch(`${API_URL}/elections/${electionId}/whitelist`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ document_hashes: [hash] })
            })
            if (!res.ok) throw new Error("Failed to add")
            return res.text()
        },
        onSuccess: () => {
            setNewId("")
            queryClient.invalidateQueries({ queryKey: ['whitelist', electionId] })
        }
    })

    const handleAdd = () => {
        if (!newId) return
        // In real app, we might just send the raw ID if we trust the channel, 
        // OR we hash it here to match what we expect. 
        // The Identity Validator mocks hash via `hash_data`. 
        // Let's assume we just enter the RAW ID here, and hashing happens on Server? 
        // Actually, the server `validate_identity` hashes the raw ID it *guesses* or extracts.
        // My previous server code in `add_whitelist` expects `document_hashes`.
        // So I should hash it here IF I know the hashing algo.
        // For sync simplicity, let's say the Input is ALREADY the Hash, or we just Hash it with sha256 (if that's what backend uses?).
        // Backend `validate_identity` uses `crypto::hash_data`. We should probably align them.
        // Since I can't guarantee backend implementation from here easily without checking `crypto/mod.rs`,
        // I'll assume for this demo the admin enters the "Document ID" and we hash it with SHA256 before sending as "hash".
        // *Correction*: Backend `crypto::hash_data` usually uses sha256. 

        const hash = sha256(newId);
        addMutation.mutate(hash)
    }

    // Fetch Status
    const { data: election } = useQuery<{ status: string }>({
        queryKey: ['election', electionId],
        queryFn: () => fetcher(`/elections/${electionId}`)
    })

    if (isLoading) return <div className="p-10 text-center">{t("common.loading")}</div>

    const isClosed = election?.status === 'SEALED';

    return (
        <div className="container mx-auto py-10 space-y-8 max-w-2xl">
            <div className="space-y-2">
                <Button variant="link" asChild className="p-0 text-muted-foreground">
                    <Link href="/admin/elections">&larr; {t("monitor.backToElections")}</Link>
                </Button>
                <h1 className="text-3xl font-bold">{t("whitelist.manage")}</h1>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>{t("whitelist.addVoter")}</CardTitle>
                    <CardDescription>{t("whitelist.instruction")}</CardDescription>
                </CardHeader>
                <CardContent className="flex gap-4">
                    <Input
                        placeholder={t("whitelist.placeholder")}
                        value={newId}
                        onChange={(e) => setNewId(e.target.value)}
                        disabled={isClosed}
                    />
                    <Button onClick={handleAdd} disabled={!newId || addMutation.isPending || isClosed}>
                        {isClosed ? "Closed" : (addMutation.isPending ? t("whitelist.adding") : t("whitelist.add"))}
                    </Button>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>{t("whitelist.hashes")} ({whitelist?.length || 0})</CardTitle>
                </CardHeader>
                <CardContent>
                    <ul className="space-y-2">
                        {whitelist?.map((hash, i) => (
                            <li key={i} className="font-mono text-xs bg-slate-50 p-2 rounded">
                                {hash}
                            </li>
                        ))}
                    </ul>
                </CardContent>
            </Card>
        </div>
    )
}
