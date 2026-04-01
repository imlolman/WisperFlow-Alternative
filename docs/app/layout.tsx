import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { githubRepoUrl, site } from "@/lib/site";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: "swap",
});

export const metadata: Metadata = {
  title: `${site.name} | ${site.tagline}`,
  description: site.description,
  icons: { icon: "/app-icon.png", apple: "/app-icon.png" },
  openGraph: {
    title: site.name,
    description: site.description,
    url: githubRepoUrl,
    siteName: site.name,
    type: "website",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="scroll-smooth">
      <body className={`${inter.variable} min-h-screen font-sans antialiased`}>{children}</body>
    </html>
  );
}
