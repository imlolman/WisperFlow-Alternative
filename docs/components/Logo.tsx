import Image from "next/image";
import { site } from "@/lib/site";

export function LogoMark({ className = "" }: { className?: string }) {
  return (
    <div className={`flex items-center gap-2.5 ${className}`}>
      <Image
        src="/app-icon.png"
        alt=""
        width={32}
        height={32}
        className="h-8 w-8 shrink-0 rounded-lg object-contain shadow-[0_0_28px_rgba(56,189,248,0.32)] ring-1 ring-white/10"
        priority
      />
      <span className="text-lg font-semibold tracking-tight text-white">{site.name}</span>
    </div>
  );
}
