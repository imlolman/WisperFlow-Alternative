export const site = {
  name: "OpenBolo",
  tagline:
    "Free and open-source alternative to Wispr Flow, Superwhisper, and Monologue.",
  description:
    "OpenBolo runs Whisper on your device (Mac, Windows, or Linux). Hold a shortcut, speak, and text appears where you type. No cloud, no API keys, no subscription.",
  github: {
    owner: "imlolman",
    repo: "OpenBolo",
  },
} as const;

/** Must match stable names produced by `.github/workflows/build.yml` release job. */
export const releaseAssets = {
  macArm64Dmg: "OpenBolo-mac-arm64.dmg",
  macX64Dmg: "OpenBolo-mac-x64.dmg",
  windowsSetupExe: "OpenBolo-win-x64-setup.exe",
  linuxAppImage: "OpenBolo-linux-x64.AppImage",
} as const;

export const githubRepoUrl = `https://github.com/${site.github.owner}/${site.github.repo}`;

export function latestDownloadUrl(filename: string): string {
  return `${githubRepoUrl}/releases/latest/download/${encodeURIComponent(filename)}`;
}

export const releasesUrl = `${githubRepoUrl}/releases/latest`;
export const starsBadgeUrl = `https://img.shields.io/github/stars/${site.github.owner}/${site.github.repo}?style=social&logo=github&label=Star`;
