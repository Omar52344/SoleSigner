"use client"

import { Button } from "@/components/ui/button"
import { useLanguage } from "@/components/language-provider"

export function LanguageSelector() {
    const { language, setLanguage } = useLanguage()

    return (
        <div className="flex gap-2 absolute top-4 right-4 z-50">
            <Button
                variant={language === 'en' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setLanguage('en')}
                className="w-10 px-0"
            >
                EN
            </Button>
            <Button
                variant={language === 'es' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setLanguage('es')}
                className="w-10 px-0"
            >
                ES
            </Button>
        </div>
    )
}
