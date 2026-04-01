"use client";

import { useState } from "react";
import { site } from "@/lib/site";
import { HeroDownloads } from "@/components/HeroDownloads";
import { HeroMacQuarantine } from "@/components/HeroMacQuarantine";

export function Hero() {
  const [showQuarantine, setShowQuarantine] = useState(false);

  return (
    <section className="relative mx-auto max-w-4xl px-6 pb-8 pt-4 text-center">
      <p className="mb-6 inline-flex rounded-full border border-sky-500/30 bg-sky-500/10 px-4 py-1.5 text-xs font-medium uppercase tracking-wider text-sky-300/95">
        v0.1 · now on GitHub
      </p>
      <h1 className="text-balance text-4xl font-semibold leading-tight tracking-tight text-white sm:text-5xl md:text-6xl">
        Your voice, typed{" "}
        <span className="bg-gradient-to-r from-cyan-300 via-sky-400 to-blue-500 bg-clip-text text-transparent drop-shadow-[0_0_32px_rgba(56,189,248,0.4)]">
          locally
        </span>
        .
      </h1>
      <p className="mx-auto mt-6 max-w-2xl text-pretty text-lg text-zinc-400 sm:text-xl">
        {site.tagline} {site.description}
      </p>
      <HeroDownloads onMacDownload={() => setShowQuarantine(true)} />
      {showQuarantine && <HeroMacQuarantine />}
    </section>
  );
}
