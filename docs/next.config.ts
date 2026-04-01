import type { NextConfig } from "next";
import path from "node:path";
import { fileURLToPath } from "node:url";

const docsRoot = path.dirname(fileURLToPath(import.meta.url));

const nextConfig: NextConfig = {
  turbopack: {
    root: docsRoot,
  },
};

export default nextConfig;
