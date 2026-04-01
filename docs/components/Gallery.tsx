import { MediaPlaceholder } from "@/components/MediaPlaceholder";

const tiles = [
  "Hold-to-talk shortcut with live level meter",
  "Settings: mic pick, shortcuts, and model download",
  "Processing state while Whisper runs locally",
  "History or paste-last flow in your editor",
];

export function Gallery() {
  return (
    <section className="mx-auto max-w-6xl px-6 py-20 md:py-28" id="gallery">
      <div className="mx-auto max-w-2xl text-center">
        <h2 className="text-3xl font-semibold tracking-tight text-white md:text-4xl">
          See it in action
        </h2>
        <p className="mt-4 text-lg text-zinc-400">
          A grid of shots or short clips. Drop in exports once you capture them.
        </p>
      </div>
      <div className="mt-14 grid gap-4 sm:grid-cols-2">
        {tiles.map((label) => (
          <MediaPlaceholder key={label} label={label} aspect="video" />
        ))}
      </div>
    </section>
  );
}
