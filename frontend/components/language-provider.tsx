"use client"

import React, { createContext, useContext, useState, ReactNode } from 'react'

type Language = 'en' | 'es'

interface Translations {
    [key: string]: {
        en: string
        es: string
    }
}

const translations: Translations = {
    // Nav
    "nav.home": { en: "Home", es: "Inicio" },
    "nav.create": { en: "Create Election", es: "Crear Elección" },
    "nav.verify": { en: "Verify Receipt", es: "Verificar Recibo" },

    // Create Election
    "create.title": { en: "Create New Election", es: "Crear Nueva Elección" },
    "create.electionTitle": { en: "Election Title", es: "Título de la Elección" },
    "create.questions": { en: "Questions", es: "Preguntas" },
    "create.addQuestion": { en: "+ Add Question", es: "+ Agregar Pregunta" },
    "create.launch": { en: "Launch Election", es: "Lanzar Elección" },
    "create.creating": { en: "Creating...", es: "Creando..." },
    "create.questionText": { en: "Question text", es: "Texto de la pregunta" },
    "create.options": { en: "Options (comma separated)", es: "Opciones (separadas por coma)" },
    "create.remove": { en: "Remove", es: "Eliminar" },

    // Vote
    "vote.identity": { en: "1. Identity", es: "1. Identidad" },
    "vote.biometrics": { en: "2. Biometrics", es: "2. Biometría" },
    "vote.cast": { en: "3. Vote", es: "3. Votar" },
    "vote.done": { en: "4. Done", es: "4. Listo" },

    // Home
    "home.subtitle": { en: "Sovereign, Auditable, Digital Voting.", es: "Voto Digital, Soberano y Auditable." },
    "home.enterElection": { en: "Enter Election", es: "Ingresar a Elección" },
    "home.enteruuid": { en: "Enter the UUID to participate.", es: "Ingrese el UUID para participar." },
    "home.placeholder": { en: "e.g. 550e8400...", es: "ej. 550e8400..." },
    "home.goVote": { en: "Go to Vote", es: "Ir a Votar" },
    "home.audit": { en: "Audit Receipt", es: "Auditar Recibo" },
    "home.admin": { en: "Admin Panel", es: "Panel Admin" },

    // Verify
    "verify.independent": { en: "Independent Audit", es: "Auditoría Independiente" },
    "verify.title": { en: "Verify Your Vote", es: "Verifica tu Voto" },
    "verify.desc": { en: "Upload your receipt to cryptographically verify inclusion.", es: "Sube tu recibo para verificar criptográficamente la inclusión." },
    "verify.ballotHash": { en: "Ballot Hash", es: "Hash de la Boleta" },
    "verify.pathLength": { en: "Path Length", es: "Longitud del Camino" },
    "verify.success": { en: "Computation Successful. Your vote is mathematically linked to the Merkle Root.", es: "Cálculo Exitoso. Tu voto está matemáticamente vinculado a la Raíz de Merkle." },
    "verify.run": { en: "Run Verification Algorithm", es: "Ejecutar Algoritmo de Verificación" },
    "verify.invalidFile": { en: "Invalid File", es: "Archivo Inválido" },
    "verify.notValidJson": { en: "Not a valid JSON receipt", es: "No es un recibo JSON válido" },
    "verify.rootComputed": { en: "Root Computed", es: "Raíz Calculada" },
    "verify.merkleRoot": { en: "Merkle Root", es: "Raíz de Merkle" },
    "verify.checkPublic": { en: "(Verify this against public board)", es: "(Verifique esto contra el tablero público)" },

    // Vote Steps
    // Step 1
    "vote.step1.title": { en: "Verification", es: "Verificación" },
    "vote.step1.desc": { en: "We need your location and ID number to proceed.", es: "Necesitamos tu ubicación y número de identificación para proceder." },
    "vote.step1.locationAcquired": { en: "Location Acquired ✅", es: "Ubicación Adquirida ✅" },
    "vote.step1.enableLocation": { en: "📍 Enable Location", es: "📍 Activar Ubicación" },
    "vote.step1.docLabel": { en: "Document Number", es: "Número de Documento" },
    "vote.step1.docPlaceholder": { en: "Enter ID Number manually for demo", es: "Ingrese número de ID manualmente (demo)" },
    "vote.step1.next": { en: "Next: Biometrics", es: "Siguiente: Biometría" },

    // Step 2
    "vote.step2.title": { en: "Face & ID Capture", es: "Captura de Rostro e ID" },
    "vote.step2.desc": { en: "Look at the camera.", es: "Mire a la cámara." },
    "vote.step2.captureSelfie": { en: "Capture Selfie", es: "Capturar Selfie" },
    "vote.step2.retakeSelfie": { en: "Retake Selfie", es: "Retomar Selfie" },
    "vote.step2.captureID": { en: "Capture ID", es: "Capturar ID" },
    "vote.step2.retakeID": { en: "Retake ID", es: "Retomar ID" },
    "vote.step2.bothCaptured": { en: "Both images captured ready for processing.", es: "Ambas imágenes capturadas listas para procesar." },
    "vote.step2.verifying": { en: "Verifying...", es: "Verificando..." },
    "vote.step2.verifyBtn": { en: "Verify Identity", es: "Verificar Identidad" },

    // Step 3
    "vote.step3.title": { en: "Cast Your Vote", es: "Emite tu Voto" },
    "vote.step3.submitting": { en: "Submitting...", es: "Enviando..." },
    "vote.step3.submitBtn": { en: "Submit Vote Securely", es: "Enviar Voto Seguro" },

    // Step 4
    "vote.step4.recorded": { en: "Vote Recorded!", es: "¡Voto Registrado!" },
    "vote.step4.secured": { en: "Your vote has been cryptographically secured.", es: "Tu voto ha sido asegurado criptográficamente." },
    "vote.step4.saveReceipt": { en: "Save this receipt. You can use it to audit your vote later without revealing your choices.", es: "Guarda este recibo. Puedes usarlo para auditar tu voto luego sin revelar tus elecciones." },
    "vote.step4.download": { en: "Download Receipt (.json)", es: "Descargar Recibo (.json)" },
    "vote.step4.return": { en: "Return Home", es: "Volver a Inicio" },

    // Common
    "common.error": { en: "Error", es: "Error" },
    "common.locationError": { en: "Location Error", es: "Error de Ubicación" },
    "common.geoNotSupported": { en: "Geolocation not supported", es: "Geolocalización no soportada" },
    "common.loading": { en: "Loading...", es: "Cargando..." },
}

interface LanguageContextType {
    language: Language
    setLanguage: (lang: Language) => void
    t: (key: string) => string
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined)

export function LanguageProvider({ children }: { children: ReactNode }) {
    const [language, setLanguage] = useState<Language>('es') // Default to Spanish as requested implicitly by "translate project"

    const t = (key: string) => {
        const item = translations[key]
        if (!item) return key
        return item[language]
    }

    return (
        <LanguageContext.Provider value={{ language, setLanguage, t }}>
            {children}
        </LanguageContext.Provider>
    )
}

export const useLanguage = () => {
    const context = useContext(LanguageContext)
    if (!context) throw new Error("useLanguage must be used within a LanguageProvider")
    return context
}
