"use client"

import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { API_URL } from "@/lib/utils"
import { useLanguage } from "@/components/language-provider"
import { useToast } from "@/hooks/use-toast"

export default function CreateElectionPage() {
    const router = useRouter()
    const { toast } = useToast()
    const { t } = useLanguage()

    const [title, setTitle] = useState("")
    // Simple textual questions for MVP
    const [questions, setQuestions] = useState([{ id: "q1", text: "Question 1", type: "radio", options: "Yes,No" }])

    const createMutation = useMutation({
        mutationFn: async (data: any) => {
            const res = await fetch(`${API_URL}/elections/create`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data)
            })
            if (!res.ok) throw new Error(await res.text())
            return res.json()
        },
        onSuccess: (data) => {
            toast({ title: "Election Created", description: `ID: ${data.id}` })
            router.push(`/vote/${data.id}`) // Redirect to test it
        }
    })

    const handleSubmit = () => {
        // Parse options from comma string to array
        const formattedQuestions = questions.map(q => ({
            ...q,
            options: q.options.split(",").map(s => s.trim())
        }))

        const payload = {
            title,
            form_config: { questions: formattedQuestions },
            start_date: new Date().toISOString(),
            end_date: new Date(Date.now() + 86400000).toISOString(), // +1 day
            access_type: "PUBLIC"
        }
        createMutation.mutate(payload)
    }

    return (
        <div className="container mx-auto max-w-2xl py-10 space-y-8">
            <h1 className="text-3xl font-bold">Create New Election</h1>

            <div className="space-y-4">
                <label className="block text-sm font-medium">Election Title</label>
                <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="e.g. Board of Directors 2024" />
            </div>

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
