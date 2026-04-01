"use client";

import { useCallback, useState } from "react";
import { MAC_QUARANTINE_CMD } from "@/lib/quarantine";

export function MacGatekeeperTip() {
  const [copied, setCopied] = useState(false);

  const copy = useCallback(() => {
    void navigator.clipboard.writeText(MAC_QUARANTINE_CMD).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    });
  }, []);

  return (
    <section
      className="mx-auto max-w-2xl px-6 py-12 md:py-16"
      id="mac-gatekeeper"
      aria-labelledby="mac-gatekeeper-heading"
    >
      <div
        className="relative overflow-hidden rounded-2xl border border-white/[0.08] bg-[#050a10] p-6 shadow-[inset_0_1px_0_0_rgba(56,189,248,0.06)] md:p-8"
        style={{
          backgroundImage: `
            linear-gradient(to bottom, rgba(10, 20, 36, 0.92), rgba(5, 10, 16, 0.98)),
            linear-gradient(rgba(56, 189, 248, 0.04) 1px, transparent 1px),
            linear-gradient(90deg, rgba(56, 189, 248, 0.04) 1px, transparent 1px)
          `,
          backgroundSize: "100% 100%, 100% 24px, 24px 100%",
        }}
      >
        <div className="relative z-[1]">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <h2
              id="mac-gatekeeper-heading"
              className="max-w-[20rem] text-left text-sm font-medium leading-snug text-zinc-200 md:max-w-none md:text-base"
            >
              If macOS says &ldquo;OpenBolo is damaged&rdquo; or won&apos;t open it, run this in{" "}
              <span className="text-sky-300/90">Terminal</span>:
            </h2>
            <span
              className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-white/10 bg-white/[0.04] text-xs font-semibold text-zinc-500"
              title="Gatekeeper adds a quarantine flag to apps from the internet. This command removes it for the copy in Applications. You may need to adjust the path if OpenBolo.app lives elsewhere."
            >
              i
            </span>
          </div>

          <div className="relative mt-5 flex items-stretch gap-2 rounded-xl border border-white/[0.1] bg-black/50 pl-4 pr-2 py-3 font-mono text-[13px] leading-relaxed text-sky-100/90 md:text-sm">
            <code className="min-w-0 flex-1 break-all pr-10">{MAC_QUARANTINE_CMD}</code>
            <button
              type="button"
              onClick={copy}
              className="absolute right-2 top-1/2 flex h-9 w-9 -translate-y-1/2 items-center justify-center rounded-lg border border-white/10 bg-white/[0.06] text-zinc-300 transition-colors hover:border-sky-500/35 hover:bg-white/[0.1] hover:text-white"
              aria-label={copied ? "Copied" : "Copy command to clipboard"}
            >
              {copied ? (
                <span className="text-[11px] font-sans font-medium text-sky-400">OK</span>
              ) : (
                <CopyIcon className="h-4 w-4" />
              )}
            </button>
          </div>

          <p className="mt-4 text-xs leading-relaxed text-zinc-500">
            Drag <strong className="font-medium text-zinc-400">OpenBolo.app</strong> into{" "}
            <strong className="font-medium text-zinc-400">Applications</strong> first, then run the
            command. If the app is somewhere else, replace the path with the full path to your{" "}
            <code className="rounded bg-white/[0.06] px-1 py-0.5 text-[11px] text-zinc-400">
              OpenBolo.app
            </code>
            .
          </p>
        </div>
      </div>
    </section>
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
