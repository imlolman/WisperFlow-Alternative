"use client";

import { useState } from "react";
import { latestDownloadUrl, releaseAssets, releasesUrl } from "@/lib/site";
import { MAC_QUARANTINE_CMD } from "@/lib/quarantine";

const macArm64Href = latestDownloadUrl(releaseAssets.macArm64Dmg);
const macX64Href = latestDownloadUrl(releaseAssets.macX64Dmg);
const windowsHref = latestDownloadUrl(releaseAssets.windowsSetupExe);
const linuxHref = latestDownloadUrl(releaseAssets.linuxAppImage);

export function HeroDownloads() {
  const [macSelected, setMacSelected] = useState(false);

  return (
    <div className="mx-auto mt-10 flex w-full max-w-2xl flex-col items-center px-0">
      <div className="flex w-full flex-col items-center justify-center gap-3 sm:flex-row sm:flex-wrap sm:justify-center">
        <a
          href={macArm64Href}
          target="_blank"
          rel="noopener noreferrer"
          onClick={() => setMacSelected(true)}
          className="inline-flex w-full min-w-[200px] items-center justify-center gap-2 rounded-xl border border-white/10 bg-white/[0.06] px-6 py-3.5 text-sm font-semibold text-white transition-all hover:border-sky-500/45 hover:bg-white/[0.1] sm:w-auto"
        >
          <AppleIcon className="h-5 w-5" />
          Download for Mac
        </a>
        <a
          href={windowsHref}
          target="_blank"
          rel="noopener noreferrer"
          onClick={() => setMacSelected(false)}
          className="inline-flex w-full min-w-[200px] items-center justify-center gap-2 rounded-xl border border-white/10 bg-white/[0.06] px-6 py-3.5 text-sm font-semibold text-white transition-all hover:border-sky-500/45 hover:bg-white/[0.1] sm:w-auto"
        >
          <WindowsIcon className="h-5 w-5" />
          Download for Windows
        </a>
        <a
          href={linuxHref}
          target="_blank"
          rel="noopener noreferrer"
          onClick={() => setMacSelected(false)}
          className="inline-flex w-full min-w-[200px] items-center justify-center gap-2 rounded-xl border border-white/10 bg-white/[0.06] px-6 py-3.5 text-sm font-semibold text-white transition-all hover:border-sky-500/45 hover:bg-white/[0.1] sm:w-auto"
        >
          <LinuxIcon className="h-5 w-5" />
          Download for Linux
        </a>
      </div>

      <p className="mt-3 text-center text-xs text-zinc-500">
        Mac build is Apple Silicon (arm64).{" "}
        <a
          className="text-sky-400/90 underline-offset-2 hover:underline"
          href={macX64Href}
          target="_blank"
          rel="noopener noreferrer"
        >
          Intel Mac (x64) .dmg
        </a>
        {" · "}
        <a
          className="text-sky-400/90 underline-offset-2 hover:underline"
          href={releasesUrl}
          target="_blank"
          rel="noopener noreferrer"
        >
          All releases
        </a>
      </p>

      {macSelected ? (
        <p className="mt-4 max-w-xl text-pretty text-center text-xs leading-relaxed text-zinc-500">
          <span className="font-medium text-zinc-400">Mac: </span>
          If Gatekeeper says the app is damaged, run{" "}
          <code className="rounded border border-white/10 bg-black/40 px-1.5 py-0.5 font-mono text-[11px] text-sky-200/90">
            {MAC_QUARANTINE_CMD}
          </code>{" "}
          in Terminal (with OpenBolo in Applications).{" "}
          <a className="text-sky-400/90 underline-offset-2 hover:underline" href="#mac-gatekeeper">
            Full steps
          </a>
        </p>
      ) : null}

      <p className="mt-6 text-center text-sm text-zinc-500">
        Native apps for macOS, Windows, and Linux. Free and open source.
      </p>
    </div>
  );
}

function AppleIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z" />
    </svg>
  );
}

function WindowsIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M3 5.548l7.14-.984v6.876H3V5.548zm0 12.432l7.14.984v-6.876H3v5.892zm8.16.984l7.8 1.08V13.5h-7.8v5.464zm0-13.464v6.876h7.8V3.6l-7.8 1.08z" />
    </svg>
  );
}

function LinuxIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M12.5 2C11.12 2 10 3.12 10 4.5c0 .53.16 1.02.43 1.43-.58.33-1.05.82-1.35 1.41-.45-.2-.95-.31-1.48-.31C5.46 7.03 4 8.49 4 10.33c0 .74.22 1.43.6 2.01l-.9 2.7c-.12.35.01.73.32.94.15.11.33.17.52.17h15.92c.19 0 .37-.06.52-.17.31-.21.44-.59.32-.94l-.9-2.7c.38-.58.6-1.27.6-2.01 0-1.84-1.46-3.3-3.3-3.3-.53 0-1.03.11-1.48.31-.3-.59-.77-1.08-1.35-1.41.27-.41.43-.9.43-1.43C15 3.12 13.88 2 12.5 2zM8.2 14.2c.44.73 1.23 1.22 2.13 1.22h3.34c.9 0 1.69-.49 2.13-1.22.33.28.52.7.52 1.15 0 .83-.67 1.5-1.5 1.5H9.18c-.83 0-1.5-.67-1.5-1.5 0-.45.19-.87.52-1.15z" />
    </svg>
  );
}
