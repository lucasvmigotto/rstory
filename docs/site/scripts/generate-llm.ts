// Emits llms.txt, llms-full.txt and one Markdown file per page and locale into dist/, from the same
// typed content the pages render. Run after `vite build` (see package.json "build").
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { llmsFull, llmsTxt, type MdContext, pageMarkdown } from "../src/content/markdown";
import { mdName, PAGES } from "../src/content/pages";
import { LOCALES } from "../src/i18n/locales";
import { MESSAGES } from "../src/i18n/registry";

const root = resolve(import.meta.dirname, "..");
const dist = resolve(root, "dist");
const cargo = readFileSync(resolve(root, "../../Cargo.toml"), "utf8");
const repo = (/^repository\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "").replace(/\/$/, "");
const version = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "0.0.0";
const siteUrl = "https://docs.lucasvmigotto.me/rstory/";

// Source links follow the build's ref, like the footer's: the tag, else the commit.
const ref = process.env.RSTORY_GIT_TAG || process.env.RSTORY_GIT_SHA || "main";

const write = (rel: string, text: string) => {
  const p = resolve(dist, rel);
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, text);
};

const [defaultLocale] = LOCALES;
for (const locale of LOCALES) {
  const ctx: MdContext = {
    locale,
    messages: MESSAGES[locale],
    siteUrl,
    repoFile: (path) => `${repo}/blob/${ref}/${path}`,
  };
  for (const p of PAGES) write(`docs/${locale}/${mdName(p)}.md`, pageMarkdown(p.id, ctx));
  if (locale === defaultLocale) {
    write("llms.txt", llmsTxt(ctx, version));
    write("llms-full.txt", llmsFull(ctx, version));
  }
}
console.log(`wrote llms.txt, llms-full.txt and ${PAGES.length * LOCALES.length} Markdown pages`);
