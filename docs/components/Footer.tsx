import { LogoMark } from "@/components/Logo";
import { githubRepoUrl, site } from "@/lib/site";

export function Footer() {
  return (
    <footer className="border-t border-white/[0.06] bg-black/20">
      <div className="mx-auto flex max-w-6xl flex-col gap-8 px-6 py-14 md:flex-row md:items-start md:justify-between">
        <div className="max-w-md">
          <LogoMark />
          <p className="mt-4 text-sm leading-relaxed text-zinc-500">
            {site.tagline} Local Whisper inference, menubar-first UX, built by the community.
          </p>
        </div>
        <div className="flex flex-col gap-3 text-sm">
          <a href={githubRepoUrl} className="text-zinc-400 hover:text-white">
            GitHub
          </a>
          <a href={`${githubRepoUrl}/releases`} className="text-zinc-400 hover:text-white">
            Releases
          </a>
          <a href="#faq" className="text-zinc-400 hover:text-white">
            FAQ
          </a>
        </div>
      </div>
      <div className="border-t border-white/[0.04] py-6 text-center text-xs text-zinc-600">
        © {new Date().getFullYear()} OpenBolo contributors. Open source under the project license.
      </div>
    </footer>
  );
}
