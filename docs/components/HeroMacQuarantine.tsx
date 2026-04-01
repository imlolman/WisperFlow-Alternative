"use client";

import { useCallback, useState } from "react";
import { MAC_QUARANTINE_CMD } from "@/lib/quarantine";

export function HeroMacQuarantine() {
  const [copied, setCopied] = useState(false);

  const copy = useCallback(() => {
    void navigator.clipboard.writeText(MAC_QUARANTINE_CMD).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    });
  }, []);

  return (
    <div className="mx-auto mt-8 w-full max-w-2xl px-6 text-center">
      <p className="flex items-center justify-center gap-1.5 text-[11px] text-sky-400/80">
        If macOS says &quot;App is damaged&quot;, run this command
        <Tooltip text="We cannot currently afford an Apple Developer Certificate ($99/year), so the app is unsigned. macOS quarantines unsigned apps by default. This command removes the quarantine attribute.">
          <span className="inline-flex h-4 w-4 shrink-0 cursor-help items-center justify-center rounded-full border border-white/10 bg-white/[0.04] text-[9px] font-semibold text-zinc-500">
            i
          </span>
        </Tooltip>
      </p>

      <div className="mx-auto mt-2 flex max-w-xl items-center gap-2 rounded-lg border border-white/[0.06] bg-white/[0.02] py-1.5 pl-3 pr-1.5">
        <div className="min-w-0 flex-1 overflow-x-auto [scrollbar-width:thin]">
          <code className="inline-block whitespace-nowrap font-mono text-[10px] leading-relaxed text-zinc-500 sm:text-[11px]">
            {MAC_QUARANTINE_CMD}
          </code>
        </div>
        <button
          type="button"
          onClick={copy}
          className="flex h-6 w-6 shrink-0 items-center justify-center rounded text-zinc-600 transition-colors hover:text-zinc-300"
          aria-label={copied ? "Copied" : "Copy command"}
        >
          {copied ? (
            <svg className="h-3.5 w-3.5 text-sky-400" viewBox="0 0 24 24" fill="none" aria-hidden>
              <path
                d="M5 13l4 4L19 7"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          ) : (
            <CopyIcon className="h-3.5 w-3.5" />
          )}
        </button>
      </div>
    </div>
  );
}

function Tooltip({ text, children }: { text: string; children: React.ReactNode }) {
  const [show, setShow] = useState(false);

  return (
    <span
      className="relative"
      onMouseEnter={() => setShow(true)}
      onMouseLeave={() => setShow(false)}
    >
      {children}
      {show && (
        <span className="absolute bottom-full left-1/2 z-50 mb-2 w-64 -translate-x-1/2 rounded-lg bg-zinc-800 px-3 py-2 text-left text-[10px] leading-snug text-zinc-300 shadow-xl ring-1 ring-white/10">
          {text}
          <span className="absolute -bottom-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 bg-zinc-800 ring-1 ring-white/10 [clip-path:polygon(0%_100%,50%_50%,100%_100%)]" />
        </span>
      )}
    </span>
  );
}

function CopyIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        d="M8 4v12a2 2 0 002 2h8a2 2 0 002-2V7.242a2 2 0 00-.586-1.414l-1.242-1.242A2 2 0 0016.758 4H10a2 2 0 00-2 2z"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      <path
        d="M16 18v2a2 2 0 01-2 2H6a2 2 0 01-2-2V8a2 2 0 012-2h2"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}
