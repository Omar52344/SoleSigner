"use client"

import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { API_URL } from "@/lib/utils"
import { useToast } from "@/hooks/use-toast"

export default function CreateElectionPage() {
    const router = useRouter()
    const { toast } = useToast()

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
                <h3 className="text-xl font-semibold">Questions</h3>
                {questions.map((q, idx) => (
                    <Card key={idx}>
                        <CardHeader>
                            <CardTitle className="text-base">Question {idx + 1}</CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-3">
                            <Input
                                value={q.text}
                                onChange={(e) => {
                                    const newQ = [...questions];
                                    newQ[idx].text = e.target.value;
                                    setQuestions(newQ);
                                }}
                                placeholder="Question text"
                            />
                            <Input
                                value={q.options}
                                onChange={(e) => {
                                    const newQ = [...questions];
                                    newQ[idx].options = e.target.value;
                                    setQuestions(newQ);
                                }}
                                placeholder="Options (comma separated)"
                            />
                        </CardContent>
                    </Card>
                ))}
                <Button variant="outline" onClick={() => setQuestions([...questions, { id: `q${questions.length + 1}`, text: "", type: "radio", options: "" }])}>
                    + Add Question
                </Button>
            </div>

            <Button size="lg" className="w-full" onClick={handleSubmit} disabled={createMutation.isPending}>
                {createMutation.isPending ? "Creating..." : "Launch Election"}
            </Button>
        </div>
    )
}
