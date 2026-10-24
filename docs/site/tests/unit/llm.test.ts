import { describe, expect, it } from "vitest";
import {
  llmsFull,
  llmsTxt,
  type MdContext,
  pageMarkdown,
  pageMdUrl,
} from "../../src/content/markdown";
import { mdName, PAGES } from "../../src/content/pages";
import { LOCALES } from "../../src/i18n/locales";
import { MESSAGES } from "../../src/i18n/registry";

const ctx = (locale: (typeof LOCALES)[number]): MdContext => ({
  locale,
  messages: MESSAGES[locale],
  siteUrl: "https://docs.lucasvmigotto.me/rstory/",
  repoFile: (p) => `https://github.com/lucasvmigotto/rstory/blob/abc1234/${p}`,
});

describe("LLM output", () => {
  for (const locale of LOCALES) {
    const c = ctx(locale);
    const ui = c.messages.ui;

    it(`${locale}: every page renders to Markdown with its title and maturity`, () => {
      for (const p of PAGES) {
        const md = pageMarkdown(p.id, c);
        expect(md.startsWith(`# ${c.messages.docs[p.id].title}\n`), p.id).toBe(true);
        expect(md, p.id).toContain(`Maturity: ${ui.maturity[p.maturity].markdown}`);
        expect(md, p.id).not.toMatch(/\[\[[a-z]+\]\]/);
        expect(md, p.id).not.toMatch(/<\/?(div|span|a|details|summary|table|button|p|ul|li)\b/);
      }
    });

    it(`${locale}: llms.txt links every page once, and every link resolves to a page`, () => {
      const txt = llmsTxt(c, "0.1.0");
      expect(txt.startsWith("# rstory\n\n> ")).toBe(true);
      const links = [...txt.matchAll(/\]\((https:[^)]+)\)/g)].map((m) => m[1] as string);
      const known = new Set([...PAGES.map((p) => pageMdUrl(c, p.id)), `${c.siteUrl}llms-full.txt`]);
      for (const l of links) expect(known.has(l), l).toBe(true);
      for (const p of PAGES) expect(links, p.id).toContain(pageMdUrl(c, p.id));
      expect(pageMdUrl(c, "ref-cli")).toBe(`${c.siteUrl}docs/${locale}/reference/cli.md`);
      expect(mdName({ path: "/" })).toBe("index");
    });

    it(`${locale}: llms-full.txt holds every page`, () => {
      const full = llmsFull(c, "0.1.0");
      for (const p of PAGES) expect(full, p.id).toContain(`# ${c.messages.docs[p.id].title}\n`);
    });

    it(`${locale}: every non-Implemented item carries its label in the text`, () => {
      for (const p of PAGES) {
        const md = pageMarkdown(p.id, c);
        for (const b of c.messages.docs[p.id].blocks) {
          const strs: string[] =
            b.t === "table"
              ? b.rows.flat()
              : b.t === "ul" || b.t === "ol"
                ? b.items
                : "text" in b
                  ? [b.text]
                  : [];
          for (const s of strs) {
            for (const m of s.matchAll(/\[\[([a-z]+)\]\]/g)) {
              const label = ui.maturity[m[1] as keyof typeof ui.maturity].label;
              expect(md, `${p.id}: ${label}`).toContain(`**${label}**`);
            }
          }
          if (b.t === "demo") expect(md).toContain(ui.maturity.planned.markdown);
          if (b.t === "callout" && b.kind === "planned")
            expect(md).toContain(ui.maturity.planned.markdown);
        }
      }
    });
  }

  it("the Planned roadmap and install pages say so in the Markdown", () => {
    const c = ctx("en-US");
    for (const id of ["project-roadmap", "install", "project-releases"] as const) {
      expect(pageMarkdown(id, c)).toContain("**Planned — not implemented yet.**");
    }
    const roadmap = pageMarkdown("project-roadmap", c);
    const rows = roadmap
      .split("\n")
      .filter((l) => l.startsWith("| ") && !l.startsWith("| Item") && !l.startsWith("| ---"));
    expect(rows.length).toBeGreaterThan(5);
    for (const r of rows) expect(r).toContain("**Planned**");
  });

  it("keeps the CLI reference's code blocks and the generated help in the Markdown", () => {
    const md = pageMarkdown("ref-cli", ctx("en-US"));
    expect(md).toContain("### rstory ls --help");
    expect(md).toContain("```text");
    expect(md).toContain("Usage: rstory ls");
  });

  it("links to repository files at the build's ref", () => {
    const md = pageMarkdown("ref-protocol", ctx("en-US"));
    expect(md).toContain("blob/abc1234/contracts/protocol.md");
    expect(md).not.toContain("blob/main");
  });
});
