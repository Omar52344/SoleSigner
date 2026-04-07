"use client"

import { useState, useRef, useCallback, useEffect } from "react"
import { useQuery, useMutation } from "@tanstack/react-query"


import { useRouter } from "next/navigation"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { useToast } from "@/hooks/use-toast"
import { fetcher, API_URL } from "@/lib/utils"
import { useLanguage } from "@/components/language-provider"

// Types
interface Question {
    id: string
    text: string
    type: "radio" | "text" | "checkbox"
    options?: string[]
}

interface FormConfig {
    questions: Question[]
}

interface Election {
    id: string
    title: string
    form_config: FormConfig
    election_salt: string
    status: string
    access_type: "PUBLIC" | "PRIVATE"
}

export default function VotePage({ params }: { params: { election_id: string } }) {
    const { election_id } = params
    const router = useRouter()
    const [step, setStep] = useState(1)
    const { toast } = useToast()
    const { t } = useLanguage()

    // State
    const [location, setLocation] = useState<{ lat: number, lng: number } | null>(null)
    const [docNumber, setDocNumber] = useState("")
    const [nullifier, setNullifier] = useState<string | null>(null)

    const [answers, setAnswers] = useState<Record<string, any>>({})
    const [receipt, setReceipt] = useState<any>(null)

    // 1. Fetch Election Data
    const { data: election, isLoading, error } = useQuery<Election>({
        queryKey: ['election', election_id],
        queryFn: () => fetcher(`/elections/${election_id}`)
    })

    // 1.5 Check Eligibility Mutation
    const checkEligibilityMutation = useMutation({
        mutationFn: async () => {
            const res = await fetch(`${API_URL}/vote/check-eligibility`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ election_id, document_number: docNumber })
            })
            if (!res.ok) throw new Error(await res.text())
            // if ok, it means eligible
            return true
        },
        onSuccess: () => {
            setStep(2)
        },
        onError: (err) => {
            toast({ title: t("error.eligibilityFailed"), description: t("error.identityNotAuthorized"), variant: "destructive" })
        }
    })

    // 2. Identity Mutation
    const identityMutation = useMutation({
        mutationFn: async (data: any) => {
            const res = await fetch(`${API_URL}/vote/validate-identity`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data)
            })
            if (!res.ok) throw new Error(await res.text())
            return res.json()
        },
        onSuccess: (data) => {
            setNullifier(data.nullifier)
            setStep(3) // Move to Ballot
            toast({ title: t("msg.identityVerified"), description: t("msg.canVote") })
        },
        onError: (err) => {
            toast({ title: t("error.verificationFailed"), description: err.message, variant: "destructive" })
        }
    })

    // 3. Submit Vote Mutation
    const submitVoteMutation = useMutation({
        mutationFn: async (data: any) => {
            const res = await fetch(`${API_URL}/vote/submit`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data)
            })
            if (!res.ok) throw new Error(await res.text())
            return res.json()
        },
        onSuccess: (data) => {
            setReceipt(data)
            setStep(4)
        }
    })

    // Handler: Get Location
    const handleGetLocation = () => {
        if (!navigator.geolocation) {
            toast({ title: "Error", description: "Geolocation not supported", variant: "destructive" })
            return
        }
        navigator.geolocation.getCurrentPosition(
            (pos) => {
                setLocation({ lat: pos.coords.latitude, lng: pos.coords.longitude })
                toast({ title: "Location Acquired" })
            },
            (err) => toast({ title: "Location Error", description: err.message, variant: "destructive" })
        )
    }




    // Steps Rendering
    if (isLoading) return <div className="p-10 text-center">{t("common.loading")}</div>
    if (error || !election) return <div className="p-10 text-center text-red-500">{t("common.error")}</div>

    if (election.status !== 'OPEN') return (
        <div className="container mx-auto max-w-lg p-10 text-center">
            <Card className="border-yellow-400 bg-yellow-50">
                <CardHeader>
                    <CardTitle className="text-yellow-800">{t("vote.closedTitle")}</CardTitle>
                    <CardDescription className="text-yellow-700">{t("vote.closedDesc")}</CardDescription>
                </CardHeader>
                <CardFooter>
                    <Button variant="outline" className="w-full" onClick={() => router.push('/')}>
                        {t("vote.step4.return")}
                    </Button>
                </CardFooter>
            </Card>
        </div>
    )

    // Generate Receipt Download
    const downloadReceipt = () => {
        if (!receipt) return
        const blob = new Blob([JSON.stringify(receipt, null, 2)], { type: "application/json" })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = `receipt-${election_id}.json`
        a.click()
    }

    return (
        <div className="container mx-auto max-w-lg p-4 min-h-screen py-10">
            <div className="mb-6 space-y-2">
                <h1 className="text-3xl font-bold">{election.title}</h1>
                <div className="flex gap-2 text-sm text-muted-foreground">
                    <span className={step >= 1 ? "text-primary font-bold" : ""}>{t("vote.identity")}</span> &gt;
                    <span className={step >= 2 ? "text-primary font-bold" : ""}>{t("vote.biometrics")}</span> &gt;
                    <span className={step >= 3 ? "text-primary font-bold" : ""}>{t("vote.cast")}</span> &gt;
                    <span className={step >= 4 ? "text-primary font-bold" : ""}>{t("vote.done")}</span>
                </div>
            </div>

            {/* STEP 1: IDENTITY & LOCATION */}
            {step === 1 && (
                <Card>
                    <CardHeader>
                        <CardTitle>{t("vote.step1.title")}</CardTitle>
                        <CardDescription>{t("vote.step1.desc")}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <Button onClick={handleGetLocation} variant="outline" className="w-full">
                            {location ? t("vote.step1.locationAcquired") : t("vote.step1.enableLocation")}
                        </Button>

                        <div className="space-y-2">
                            <label className="text-sm font-medium">{t("vote.step1.docLabel")}</label>
                            <Input
                                placeholder={t("vote.step1.docPlaceholder")}
                                value={docNumber}
                                onChange={(e) => setDocNumber(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                In a real flow, this would be extracted via OCR in the next step, but entered here for backup.
                            </p>
                        </div>
                    </CardContent>
                    <CardFooter>
                        <Button
                            className="w-full"
                            disabled={!location || !docNumber || checkEligibilityMutation.isPending}
                            onClick={() => {
                                if (election.access_type === "PRIVATE") {
                                    checkEligibilityMutation.mutate();
                                } else {
                                    setStep(2);
                                }
                            }}
                        >
                            {checkEligibilityMutation.isPending ? "Checking..." : t("vote.step1.next")}
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* STEP 2: VERIFICATION */}
            {step === 2 && (
                <Card>
                    <CardHeader>
                        <CardTitle>{t("vote.step2.title")}</CardTitle>
                        <CardDescription>{t("vote.step2.desc")}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="space-y-2">
                            <label className="text-sm font-medium">{t("vote.step1.docLabel")}</label>
                            <div className="p-3 border rounded-md bg-gray-50">
                                {docNumber}
                            </div>
                            <p className="text-xs text-muted-foreground">
                                {t("vote.step2.confirmDoc")}
                            </p>
                        </div>
                    </CardContent>
                    <CardFooter>
                        <Button
                            className="w-full"
                            disabled={identityMutation.isPending}
                            onClick={() => identityMutation.mutate({
                                election_id,
                                document_number: docNumber
                            })}
                        >
                            {identityMutation.isPending ? t("vote.step2.verifying") : t("vote.step2.verifyBtn")}
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* STEP 3: VOTE */}
            {step === 3 && (
                <Card>
                    <CardHeader>
                        <CardTitle>{t("vote.step3.title")}</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        {election.form_config.questions?.map((q) => (
                            <div key={q.id} className="space-y-2">
                                <h3 className="font-semibold text-lg">{q.text}</h3>
                                {q.type === 'radio' && (
                                    <div className="flex flex-col gap-2">
                                        {q.options?.map((opt) => (
                                            <label key={opt} className="flex items-center space-x-2 border p-3 rounded-md cursor-pointer hover:bg-slate-50">
                                                <input
                                                    type="radio"
                                                    name={q.id}
                                                    value={opt}
                                                    onChange={(e) => setAnswers(prev => ({ ...prev, [q.id]: e.target.value }))}
                                                    className="h-4 w-4"
                                                />
                                                <span>{opt}</span>
                                            </label>
                                        ))}
                                    </div>
                                )}
                                {/* Add other types if needed */}
                            </div>
                        ))}
                    </CardContent>
                    <CardFooter>
                        <Button
                            className="w-full text-lg py-6"
                            onClick={() => {
                                const requestId = crypto.randomUUID();
                                submitVoteMutation.mutate({
                                    election_id,
                                    choices: answers,
                                    nullifier: nullifier!,
                                    request_id: requestId
                                })
                            }}
                        >
                            {submitVoteMutation.isPending ? t("vote.step3.submitting") : t("vote.step3.submitBtn")}
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* STEP 4: CONFIRMATION */}
            {step === 4 && receipt && (
                <Card className="bg-green-50 border-green-200">
                    <CardHeader>
                        <CardTitle className="text-green-800">{t("vote.step4.recorded")}</CardTitle>
                        <CardDescription className="text-green-700">{t("vote.step4.secured")}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="bg-white p-4 rounded border font-mono text-xs break-all">
                            <p className="font-bold text-gray-500">{t("verify.ballotHash")}:</p>
                            {receipt.ballot_hash}
                        </div>
                        <div className="text-sm text-gray-600">
                            {t("vote.step4.saveReceipt")}
                        </div>
                    </CardContent>
                    <CardFooter className="flex flex-col gap-2">
                        <Button className="w-full" onClick={downloadReceipt}>
                            {t("vote.step4.download")}
                        </Button>
                        <Button variant="outline" className="w-full" onClick={() => router.push('/')}>
                            {t("vote.step4.return")}
                        </Button>
                    </CardFooter>
                </Card>
            )}
        </div>
    )
}
