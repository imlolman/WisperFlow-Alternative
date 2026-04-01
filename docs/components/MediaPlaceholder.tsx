const aspectMap = {
  video: "aspect-video",
  square: "aspect-square",
  wide: "aspect-[16/10]",
  tall: "aspect-[3/4]",
} as const;

export function MediaPlaceholder({
  label,
  aspect = "video",
}: {
  label: string;
  aspect?: keyof typeof aspectMap;
}) {
  return (
    <div
      className={`relative flex ${aspectMap[aspect]} w-full items-center justify-center overflow-hidden rounded-2xl border border-dashed border-white/12 bg-gradient-to-b from-white/[0.05] to-white/[0.02] text-center shadow-[inset_0_1px_0_0_rgba(255,255,255,0.06)]`}
    >
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-sky-500/12 via-transparent to-transparent" />
      <p className="relative z-[1] max-w-sm px-6 text-sm font-medium leading-relaxed text-zinc-500">
        {label}
      </p>
    </div>
  );
}
