"use client"

import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { API_URL } from "@/lib/utils"
import { useAuth } from "@/components/auth-provider"
import { useLanguage } from "@/components/language-provider"
import { useToast } from "@/hooks/use-toast"

export default function CreateElectionPage() {
    const router = useRouter()
    const { toast } = useToast()
    const { t } = useLanguage()
    const { token } = useAuth()

    const [title, setTitle] = useState("")
    const [startDate, setStartDate] = useState("")
    const [endDate, setEndDate] = useState("")
    const [closingType, setClosingType] = useState<"MANUAL" | "AUTO">("MANUAL")
    const [accessType, setAccessType] = useState<"PUBLIC" | "PRIVATE">("PUBLIC")

    // Simple textual questions for MVP
    const [questions, setQuestions] = useState([{ id: "q1", text: "Question 1", type: "radio", options: "Yes,No" }])

    const createMutation = useMutation({
        mutationFn: async (data: any) => {
            const res = await fetch(`${API_URL}/elections/create`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "Authorization": `Bearer ${token}`
                },
                body: JSON.stringify(data)
            })
            if (!res.ok) throw new Error(await res.text())
            return res.json()
        },
        onSuccess: (data) => {
            toast({ title: "Election Created", description: `ID: ${data.id}` })
            router.push(`/admin/elections`) // Go to admin list instead of vote page for now
        }
    })

    const handleSubmit = () => {
        // Parse options from comma string to array
        const formattedQuestions = questions.map(q => ({
            ...q,
            options: q.options.split(",").map(s => s.trim())
        }))

        // Verify dates
        const start = startDate ? new Date(startDate) : new Date()
        const end = endDate ? new Date(endDate) : new Date(Date.now() + 86400000)

        const payload = {
            title,
            form_config: {
                questions: formattedQuestions,
                closing_type: closingType,
                // We could store timezone here if needed
            },
            start_date: start.toISOString(),
            end_date: end.toISOString(),
            access_type: accessType
        }
        createMutation.mutate(payload)
    }

    return (
        <div className="container mx-auto max-w-2xl py-10 space-y-8">
            <h1 className="text-3xl font-bold">{t("create.title")}</h1>

            <div className="space-y-4">
                <label className="block text-sm font-medium">{t("create.electionTitle")}</label>
                <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="e.g. Board of Directors 2024" />
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>{t("create.dates")}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                            <label className="text-sm font-medium">{t("create.startDate")}</label>
                            <Input
                                type="datetime-local"
                                value={startDate}
                                onChange={(e) => setStartDate(e.target.value)}
                            />
                        </div>
                        <div className="space-y-2">
                            <label className="text-sm font-medium">{t("create.endDate")}</label>
                            <Input
                                type="datetime-local"
                                value={endDate}
                                onChange={(e) => setEndDate(e.target.value)}
                            />
                        </div>
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium">{t("create.closingType")}</label>
                        <div className="flex gap-4">
                            <div className="flex items-center space-x-2">
                                <input
                                    type="radio"
                                    id="manual"
                                    name="closing"
                                    checked={closingType === "MANUAL"}
                                    onChange={() => setClosingType("MANUAL")}
                                />
                                <label htmlFor="manual">{t("create.manual")}</label>
                            </div>
                            <div className="flex items-center space-x-2">
                                <input
                                    type="radio"
                                    id="auto"
                                    name="closing"
                                    checked={closingType === "AUTO"}
                                    onChange={() => setClosingType("AUTO")}
                                />
                                <label htmlFor="auto">{t("create.automatic")}</label>
                            </div>
                        </div>
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium">Access Type</label>
                        <div className="flex gap-4">
                            <div className="flex items-center space-x-2">
                                <input
                                    type="radio"
                                    id="public"
                                    name="access"
                                    checked={accessType === "PUBLIC"}
                                    onChange={() => setAccessType("PUBLIC")}
                                />
                                <label htmlFor="public">Public</label>
                            </div>
                            <div className="flex items-center space-x-2">
                                <input
                                    type="radio"
                                    id="private"
                                    name="access"
                                    checked={accessType === "PRIVATE"}
                                    onChange={() => setAccessType("PRIVATE")}
                                />
                                <label htmlFor="private">Private (Whitelist)</label>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>


            <div className="space-y-4">
                <h3 className="text-xl font-semibold">{t("create.questions")}</h3>
                {questions.map((q, idx) => (
                    <Card key={idx}>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-base">Question {idx + 1}</CardTitle>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => {
                                    const newQ = questions.filter((_, i) => i !== idx);
                                    setQuestions(newQ);
                                }}
                            >
                                {t("create.remove")}
                            </Button>
                        </CardHeader>
                        <CardContent className="space-y-3">
                            <Input
                                value={q.text}
                                onChange={(e) => {
                                    const newQ = [...questions];
                                    newQ[idx].text = e.target.value;
                                    setQuestions(newQ);
                                }}
                                placeholder={t("create.questionText")}
                            />
                            <Input
                                value={q.options}
                                onChange={(e) => {
                                    const newQ = [...questions];
                                    newQ[idx].options = e.target.value;
                                    setQuestions(newQ);
                                }}
                                placeholder={t("create.options")}
                            />
                        </CardContent>
                    </Card>
                ))}
                <Button variant="outline" onClick={() => setQuestions([...questions, { id: `q${questions.length + 1}`, text: "", type: "radio", options: "" }])}>
                    {t("create.addQuestion")}
                </Button>
            </div>

            <Button size="lg" className="w-full" onClick={handleSubmit} disabled={createMutation.isPending}>
                {createMutation.isPending ? t("create.creating") : t("create.launch")}
            </Button>
        </div>
    )
}
