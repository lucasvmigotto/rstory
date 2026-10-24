import { mdName, PAGES } from "./pages";
import type { Maturity } from "./types";

export type Inline =
  | { k: "text"; text: string }
  | { k: "code"; text: string }
  | { k: "bold"; text: string }
  | { k: "link"; text: string; href: string }
  | { k: "status"; status: Maturity };

const STATUSES: readonly string[] = [
  "implemented",
  "partial",
  "planned",
  "unavailable",
  "upstream",
  "na",
];
const TOKEN = /\[\[([a-z]+)\]\]|`([^`]+)`|\*\*([^*]+)\*\*|\[([^\]]+)\]\(([^)\s]+)\)/g;

export function parseInline(s: string): Inline[] {
  const out: Inline[] = [];
  let last = 0;
  for (const m of s.matchAll(TOKEN)) {
    if (m.index > last) out.push({ k: "text", text: s.slice(last, m.index) });
    if (m[1] !== undefined) {
      if (!STATUSES.includes(m[1])) throw new Error(`unknown status token [[${m[1]}]]`);
      out.push({ k: "status", status: m[1] as Maturity });
    } else if (m[2] !== undefined) out.push({ k: "code", text: m[2] });
    else if (m[3] !== undefined) out.push({ k: "bold", text: m[3] });
    else out.push({ k: "link", text: m[4] ?? "", href: m[5] ?? "" });
    last = m.index + m[0].length;
  }
  if (last < s.length) out.push({ k: "text", text: s.slice(last) });
  return out;
}

/** A link target in the content: `#/route` (a page), `src:path` (a repository file) or a URL. */
export type LinkEnv = { repoFile: (path: string) => string; mdUrl: (path: string) => string };

export function resolveLink(href: string, env: LinkEnv, mode: "web" | "md"): string {
  if (href.startsWith("src:")) return env.repoFile(href.slice(4));
  if (href.startsWith("#/")) {
    const [route = "/", hash] = href.slice(1).split("#");
    if (mode === "web") return href;
    const page = PAGES.find((p) => p.path === route);
    if (!page) throw new Error(`link to unknown page ${href}`);
    return env.mdUrl(`${mdName(page)}.md`) + (hash ? `#${hash}` : "");
  }
  return href;
}
