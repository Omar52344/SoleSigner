"use client"

import { useState, useRef, useCallback, useEffect } from "react"
import { useQuery, useMutation } from "@tanstack/react-query"
import Webcam from "react-webcam"
import { sha256 } from "js-sha256"
import { useRouter } from "next/navigation"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { useToast } from "@/hooks/use-toast"
import { fetcher, API_URL } from "@/lib/utils"

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
}

export default function VotePage({ params }: { params: { election_id: string } }) {
    const { election_id } = params
    const [step, setStep] = useState(1)
    const { toast } = useToast()

    // State
    const [location, setLocation] = useState<{ lat: number, lng: number } | null>(null)
    const [useManualId, setUseManualId] = useState(false) // Toggle for OCR vs Manual
    const [docNumber, setDocNumber] = useState("")
    const [selfie, setSelfie] = useState<string | null>(null)
    const [docImage, setDocImage] = useState<string | null>(null)
    const [nullifier, setNullifier] = useState<string | null>(null)

    const [answers, setAnswers] = useState<Record<string, any>>({})
    const [receipt, setReceipt] = useState<any>(null)

    const webcamRef = useRef<Webcam>(null)

    // 1. Fetch Election Data
    const { data: election, isLoading, error } = useQuery<Election>({
        queryKey: ['election', election_id],
        queryFn: () => fetcher(`/elections/${election_id}`)
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
            toast({ title: "Identity Verified", description: "You may now vote." })
        },
        onError: (err) => {
            toast({ title: "Verification Failed", description: err.message, variant: "destructive" })
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

    // Handler: Capture Images
    const captureSelfie = useCallback(() => {
        const imageSrc = webcamRef.current?.getScreenshot()
        if (imageSrc) setSelfie(imageSrc)
    }, [webcamRef])

    const captureDoc = useCallback(() => {
        const imageSrc = webcamRef.current?.getScreenshot()
        if (imageSrc) setDocImage(imageSrc)
    }, [webcamRef])


    // Steps Rendering
    if (isLoading) return <div className="p-10 text-center">Loading Election...</div>
    if (error || !election) return <div className="p-10 text-center text-red-500">Error loading election.</div>

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
                    <span className={step >= 1 ? "text-primary font-bold" : ""}>1. Identity</span> &gt;
                    <span className={step >= 2 ? "text-primary font-bold" : ""}>2. Biometrics</span> &gt;
                    <span className={step >= 3 ? "text-primary font-bold" : ""}>3. Vote</span> &gt;
                    <span className={step >= 4 ? "text-primary font-bold" : ""}>4. Done</span>
                </div>
            </div>

            {/* STEP 1: IDENTITY & LOCATION */}
            {step === 1 && (
                <Card>
                    <CardHeader>
                        <CardTitle>Verification</CardTitle>
                        <CardDescription>We need your location and ID number to proceed.</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <Button onClick={handleGetLocation} variant="outline" className="w-full">
                            {location ? "Location Acquired ✅" : "📍 Enable Location"}
                        </Button>

                        <div className="space-y-2">
                            <label className="text-sm font-medium">Document Number</label>
                            <Input
                                placeholder="Enter ID Number manually for demo"
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
                            disabled={!location || !docNumber}
                            onClick={() => setStep(2)}
                        >
                            Next: Biometrics
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* STEP 2: BIOMETRICS */}
            {step === 2 && (
                <Card>
                    <CardHeader>
                        <CardTitle>Face & ID Capture</CardTitle>
                        <CardDescription>Look at the camera.</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4 flex flex-col items-center">
                        <div className="relative w-full aspect-video bg-black rounded-lg overflow-hidden">
                            <Webcam
                                audio={false}
                                ref={webcamRef}
                                screenshotFormat="image/jpeg"
                                className="w-full h-full object-cover"
                            />
                        </div>

                        <div className="grid grid-cols-2 gap-2 w-full">
                            <Button variant={selfie ? "default" : "secondary"} onClick={captureSelfie}>
                                {selfie ? "Retake Selfie" : "Capture Selfie"}
                            </Button>
                            <Button variant={docImage ? "default" : "secondary"} onClick={captureDoc}>
                                {docImage ? "Retake ID" : "Capture ID"}
                            </Button>
                        </div>

                        {selfie && docImage && (
                            <div className="text-xs text-green-600 font-bold">Both images captured ready for processing.</div>
                        )}
                    </CardContent>
                    <CardFooter>
                        <Button
                            className="w-full"
                            disabled={!selfie || !docImage}
                            onClick={() => identityMutation.mutate({
                                election_id,
                                selfie_base64: selfie,
                                document_base64: docImage,
                                latitude: location?.lat || 0,
                                longitude: location?.lng || 0
                            })}
                        >
                            {identityMutation.isPending ? "Verifying..." : "Verify Identity"}
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* STEP 3: VOTE */}
            {step === 3 && (
                <Card>
                    <CardHeader>
                        <CardTitle>Cast Your Vote</CardTitle>
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
                            {submitVoteMutation.isPending ? "Submitting..." : "Submit Vote Securely"}
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* STEP 4: CONFIRMATION */}
            {step === 4 && receipt && (
                <Card className="bg-green-50 border-green-200">
                    <CardHeader>
                        <CardTitle className="text-green-800">Vote Recorded!</CardTitle>
                        <CardDescription className="text-green-700">Your vote has been cryptographically secured.</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="bg-white p-4 rounded border font-mono text-xs break-all">
                            <p className="font-bold text-gray-500">Ballot Hash:</p>
                            {receipt.ballot_hash}
                        </div>
                        <div className="text-sm text-gray-600">
                            Save this receipt. You can use it to audit your vote later without revealing your choices.
                        </div>
                    </CardContent>
                    <CardFooter className="flex flex-col gap-2">
                        <Button className="w-full" onClick={downloadReceipt}>
                            Download Receipt (.json)
                        </Button>
                        <Button variant="outline" className="w-full" onClick={() => router.push('/')}>
                            Return Home
                        </Button>
                    </CardFooter>
                </Card>
            )}
        </div>
    )
}
