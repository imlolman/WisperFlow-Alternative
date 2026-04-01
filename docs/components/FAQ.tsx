import type { ReactNode } from "react";
import { MAC_QUARANTINE_CMD } from "@/lib/quarantine";

const faqs: { q: string; a: ReactNode }[] = [
  {
    q: "Is OpenBolo really free?",
    a: "Yes. No subscription wall and no API keys. You run Whisper on your own hardware; we don’t bill you for transcription.",
  },
  {
    q: "How is this different from Wispr Flow, Superwhisper, or Monologue?",
    a: "OpenBolo is fully open source and runs locally by default. You can audit the code, fork it, and avoid vendor lock-in while keeping audio off third-party servers.",
  },
  {
    q: "Is my voice data sent anywhere?",
    a: "Transcription happens on your machine. Your audio and text aren’t uploaded to our servers, because we don’t operate transcription servers for the app.",
  },
  {
    q: "Which platforms are supported?",
    a: "Prebuilt downloads target macOS (Apple Silicon and Intel), Windows (x64), and Linux. Grab the right asset from the releases page for your system.",
  },
  {
    q: "Do I need an OpenAI API key?",
    a: "No. The app bundles local inference; you’re not routing speech through OpenAI’s cloud API to dictate.",
  },
  {
    q: "Why does macOS say the app is damaged?",
    a: (
      <>
        Unsigned or Gatekeeper-quarantined downloads often get that message. Put OpenBolo in
        Applications, then run this in Terminal:{" "}
        <code className="whitespace-pre-wrap break-all rounded border border-white/10 bg-black/35 px-1.5 py-0.5 font-mono text-[12px] text-sky-200/90">
          {MAC_QUARANTINE_CMD}
        </code>{" "}
        You can copy it from the section above as well. Or right-click the app, choose Open the first
        time. Prefer signed or notarized builds when available.
      </>
    ),
  },
  {
    q: "What permissions does OpenBolo need?",
    a: "Microphone access for capture, Accessibility on macOS for global shortcuts and typing into other apps, and optionally login items if you enable launch at startup.",
  },
  {
    q: "Will there be more updates?",
    a: "The project is open source: contributions and releases depend on the community. Watch the GitHub repo for tags and changelog activity.",
  },
];

export function FAQ() {
  return (
    <section className="mx-auto max-w-3xl px-6 py-20 md:py-28" id="faq">
      <h2 className="text-center text-3xl font-semibold tracking-tight text-white md:text-4xl">
        Frequently asked questions
      </h2>
      <div className="mt-12 space-y-3">
        {faqs.map((item) => (
          <details
            key={item.q}
            className="group rounded-2xl border border-white/[0.08] bg-white/[0.02] transition-colors open:border-sky-500/30 open:bg-white/[0.04]"
          >
            <summary className="flex cursor-pointer list-none items-center justify-between gap-4 px-5 py-4 text-left font-medium text-zinc-100 marker:content-none [&::-webkit-details-marker]:hidden">
              {item.q}
              <span className="shrink-0 text-sky-400/90 transition-transform group-open:rotate-45">
                +
              </span>
            </summary>
            <div className="border-t border-white/[0.06] px-5 py-4 text-sm leading-relaxed text-zinc-400">
              {item.a}
            </div>
          </details>
        ))}
      </div>
    </section>
  );
}
