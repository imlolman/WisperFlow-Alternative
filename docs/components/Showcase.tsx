import { MediaPlaceholder } from "@/components/MediaPlaceholder";

export function Showcase() {
  return (
    <section className="mx-auto max-w-5xl px-6 py-8 md:py-14" aria-label="Product preview">
      <div className="relative mx-auto">
        <div
          className="pointer-events-none absolute -inset-8 rounded-[2rem] bg-sky-500/18 opacity-60 blur-3xl md:-inset-12"
          aria-hidden
        />
        <div className="relative rounded-2xl ring-1 ring-sky-500/25 ring-offset-4 ring-offset-[#070b10]">
          <MediaPlaceholder
            aspect="wide"
            label="Hero image or short demo loop: menu bar, waveform overlay, and text appearing in an app. Drop your asset here."
          />
        </div>
      </div>
    </section>
  );
}
