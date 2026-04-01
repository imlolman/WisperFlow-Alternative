export function BackgroundGlow() {
  return (
    <div className="pointer-events-none fixed inset-0 -z-10 overflow-hidden bg-[#070b10]">
      <div
        className="absolute inset-0"
        style={{
          background: `
            radial-gradient(ellipse 100% 70% at 50% 45%, rgba(56, 189, 248, 0.12), transparent 52%),
            radial-gradient(ellipse 85% 55% at 50% 18%, rgba(30, 58, 95, 0.42), transparent 58%),
            radial-gradient(ellipse 60% 50% at 50% 55%, rgba(15, 40, 65, 0.22), transparent 65%)
          `,
        }}
      />
      <div className="absolute right-[-12%] top-[32%] h-[min(55vw,480px)] w-[min(55vw,480px)] rounded-full bg-sky-900/20 blur-[100px]" />
      <div className="absolute bottom-[8%] left-[-10%] h-[360px] w-[360px] rounded-full bg-blue-950/35 blur-[90px]" />
      <div
        className="absolute inset-0 opacity-[0.38]"
        style={{
          backgroundImage: `
            linear-gradient(rgba(56, 189, 248, 0.1) 1px, transparent 1px),
            linear-gradient(90deg, rgba(56, 189, 248, 0.1) 1px, transparent 1px)
          `,
          backgroundSize: "48px 48px",
          backgroundPosition: "center top",
        }}
      />
      <div
        className="absolute inset-0"
        style={{
          background:
            "radial-gradient(ellipse 85% 75% at 50% 35%, transparent 30%, rgba(7, 11, 16, 0.88) 100%)",
        }}
      />
    </div>
  );
}
