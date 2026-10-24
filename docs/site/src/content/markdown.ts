import type { Locale } from "../i18n/locales";
import type { Messages, UiStrings } from "../i18n/types";
import { type GeneratedHelp, generated } from "./generated";
import { type LinkEnv, parseInline, resolveLink } from "./inline";
import { mdName, PAGES, type PageId, SECTIONS, type SectionId } from "./pages";
import type { Block } from "./types";

export interface MdContext {
  locale: Locale;
  messages: Messages;
  siteUrl: string;
  repoFile: (path: string) => string;
  generated?: GeneratedHelp;
}

const env = (c: MdContext): LinkEnv => ({
  repoFile: c.repoFile,
  mdUrl: (p) => `${c.siteUrl}docs/${c.locale}/${p}`,
});

export const pageMdUrl = (c: MdContext, id: PageId) => {
  const p = PAGES.find((x) => x.id === id);
  if (!p) throw new Error(id);
  return `${c.siteUrl}docs/${c.locale}/${mdName(p)}.md`;
};

export function inlineToMd(s: string, ui: UiStrings, c: MdContext): string {
  return parseInline(s)
    .map((i) => {
      switch (i.k) {
        case "text":
          return i.text;
        case "code":
          return `\`${i.text}\``;
        case "bold":
          return `**${i.text}**`;
        case "link":
          return `[${i.text}](${resolveLink(i.href, env(c), "md")})`;
        case "status":
          return `**${ui.maturity[i.status].label}**`;
        default:
          return "";
      }
    })
    .join("");
}

const cell = (s: string) => s.replace(/\|/g, "\\|").replace(/\n/g, " ");
const fence = (lang: string, text: string) => {
  const ticks = text.includes("```") ? "````" : "```";
  return `${ticks}${lang}\n${text}\n${ticks}`;
};

export function blockToMd(b: Block, c: MdContext): string {
  const ui = c.messages.ui;
  const i = (s: string) => inlineToMd(s, ui, c);
  const gen = c.generated ?? generated;
  switch (b.t) {
    case "p":
      return i(b.text);
    case "h2":
      return `## ${b.text}`;
    case "h3":
      return `### ${b.text}`;
    case "ul":
      return b.items.map((x) => `- ${i(x)}`).join("\n");
    case "ol":
      return b.items.map((x, n) => `${n + 1}. ${i(x)}`).join("\n");
    case "code":
      return (b.caption ? `${b.caption}\n\n` : "") + fence(b.lang, b.text);
    case "table": {
      const head = `| ${b.head.map(cell).join(" | ")} |`;
      const sep = `| ${b.head.map(() => "---").join(" | ")} |`;
      const rows = b.rows.map((r) => `| ${r.map((x) => cell(i(x))).join(" | ")} |`);
      return [b.caption ? `${b.caption}\n` : "", head, sep, ...rows]
        .filter((x) => x !== "")
        .join("\n");
    }
    case "callout": {
      const label =
        b.kind === "planned"
          ? `${ui.maturity.planned.markdown} `
          : b.kind === "warning"
            ? "**Warning.** "
            : "**Note.** ";
      return `> ${label}${i(b.text)}`;
    }
    case "demo":
      return `### Demo: ${b.title}\n\n${ui.maturity.planned.markdown} ${ui.demo.planned}`;
    case "cli":
      return [
        `Generated from the built binary's \`--help\` output (rstory ${gen.rstory}).`,
        ...gen.commands.map((cmd) => {
          const title = cmd.name === "rstory" ? "rstory --help" : `rstory ${cmd.name} --help`;
          return `### ${title}\n\n${fence("text", cmd.help)}`;
        }),
      ].join("\n\n");
    case "configdefaults":
      return fence("toml", gen.config_defaults);
    case "doctor":
      return [
        "| Check | Status | Message without a session | Hint |",
        "| --- | --- | --- | --- |",
        ...gen.doctor.checks.map(
          (k) => `| \`${k.id}\` | ${k.status} | ${cell(k.message)} | ${cell(k.hint)} |`,
        ),
      ].join("\n");
  }
}

export function pageMarkdown(id: PageId, c: MdContext): string {
  const page = PAGES.find((p) => p.id === id);
  if (!page) throw new Error(id);
  const content = c.messages.docs[id];
  const ui = c.messages.ui;
  const head = [
    `# ${content.title}`,
    `> ${content.summary}`,
    `Maturity: ${ui.maturity[page.maturity].markdown} ${ui.maturity[page.maturity].hint}`,
  ];
  return `${[...head, ...content.blocks.map((b) => blockToMd(b, c))].join("\n\n")}\n`;
}

const sectionPages = (s: SectionId) => PAGES.filter((p) => p.section === s);

export function llmsTxt(c: MdContext, version: string): string {
  const ui = c.messages.ui;
  const lines = [
    `# rstory`,
    "",
    `> ${ui.llm.summary}`,
    "",
    `Version ${version}. Not released yet: install channels other than building from source are Planned.`,
    "",
  ];
  for (const s of SECTIONS) {
    lines.push(`## ${ui.sections[s]}`, "");
    for (const p of sectionPages(s)) {
      const d = c.messages.docs[p.id];
      const tag =
        p.maturity === "implemented" || p.maturity === "na"
          ? ""
          : ` (${ui.maturity[p.maturity].label})`;
      lines.push(`- [${d.title}](${pageMdUrl(c, p.id)}): ${d.summary}${tag}`);
    }
    lines.push("");
  }
  lines.push(
    `## ${ui.llm.optional}`,
    "",
    `- [Full documentation in one file](${c.siteUrl}llms-full.txt): every page, in navigation order`,
    "",
  );
  return lines.join("\n");
}

export function llmsFull(c: MdContext, version: string): string {
  const ui = c.messages.ui;
  const parts = [`# rstory documentation (version ${version})`, `> ${ui.llm.summary}`];
  for (const p of PAGES)
    parts.push(`<!-- ${pageMdUrl(c, p.id)} -->\n\n${pageMarkdown(p.id, c).trimEnd()}`);
  return `${parts.join("\n\n---\n\n")}\n`;
}
