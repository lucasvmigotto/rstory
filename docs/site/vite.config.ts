import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";
import { resolveVersions } from "./scripts/footer-versions.ts";

const root = import.meta.dirname;
const read = (p: string) => readFileSync(resolve(root, p), "utf8");

const cargo = read("../../Cargo.toml");
const version = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "0.0.0";
const license = /^license\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "GPL-3.0-or-later";
const docsVersion = (JSON.parse(read("package.json")) as { version: string }).version;
const repo = (/^repository\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "").replace(/\/$/, "");

const versions = resolveVersions({
  repo,
  project: "rstory",
  version,
  docsVersion,
  git: {
    sha: process.env.RSTORY_GIT_SHA,
    tag: process.env.RSTORY_GIT_TAG,
    lastTag: process.env.RSTORY_GIT_LAST_TAG,
  },
});

export const SITE_URL = "https://docs.lucasvmigotto.me/rstory/";

export default defineConfig({
  base: "/rstory/",
  plugins: [react(), tailwindcss()],
  define: {
    __SITE__: JSON.stringify({ repo, version, docsVersion, license, versions, siteUrl: SITE_URL }),
  },
  build: { outDir: "dist", sourcemap: false },
  test: {
    environment: "happy-dom",
    include: ["tests/**/*.test.{ts,tsx}"],
    setupFiles: ["./vitest.setup.ts"],
    globals: false,
  },
});
