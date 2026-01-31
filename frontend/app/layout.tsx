import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/toaster";
import Providers from "./providers"; // Will create this for React Query

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
    title: "SoleSigner | Sovereign Voting",
    description: "Auditable, Digital, Sovereign Voting System",
};

import { LanguageSelector } from "@/components/language-selector";

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en">
            <body className={inter.className}>
                <Providers>
                    <main className="min-h-screen bg-background text-foreground relative">
                        <LanguageSelector />
                        {children}
                    </main>
                    <Toaster />
                </Providers>
            </body>
        </html>
    );
}
