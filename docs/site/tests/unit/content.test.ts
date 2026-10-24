import { describe, expect, it } from "vitest";
import { parseInline } from "../../src/content/inline";
import { PAGES } from "../../src/content/pages";
import type { Block } from "../../src/content/types";
import { MESSAGES } from "../../src/i18n/registry";

const docs = MESSAGES["en-US"].docs;

/** Every string of a block, for scanning. */
function strings(b: Block): string[] {
  switch (b.t) {
    case "p":
    case "h2":
    case "h3":
    case "callout":
      return [b.text];
    case "ul":
    case "ol":
      return b.items;
    case "code":
      return [b.text];
    case "table":
      return [...b.head, ...b.rows.flat()];
    default:
      return [];
  }
}
const all = PAGES.flatMap((p) =>
  docs[p.id].blocks.flatMap((b) => strings(b).map((s) => ({ page: p.id, s }))),
);
const prose = PAGES.filter((p) => p.id !== "glossary").flatMap((p) =>
  docs[p.id].blocks
    .filter((b) => b.t !== "code")
    .flatMap((b) => strings(b).map((s) => ({ page: p.id, s }))),
);

// The no-invent list of docs/product/docs-site-discovery.md section 2.
const FLAGS = new Set([
  "--json",
  "--limit",
  "--all",
  "--yes",
  "--for",
  "--path",
  "--plain",
  "--backend",
  "--systemd",
  "--now",
  "--uninstall",
  "--enable",
  "--help",
  "--version",
  "--ascii",
  "--locked",
  "--offline",
  "--dry-run",
]);
const CONFIG_KEYS = new Set([
  "max_entries",
  "max_pinned",
  "max_entry_bytes",
  "ttl",
  "picker_command",
  "tray",
  "theme",
  "backend",
]);

describe("no-invent list", () => {
  it("shows only known flags", () => {
    for (const { page, s } of all) {
      for (const m of s.matchAll(/(?<![\w-])--[a-z][a-z-]*/g)) {
        expect(FLAGS.has(m[0]), `${page}: ${m[0]}`).toBe(true);
      }
    }
  });

  it("never shows invented install commands or debug flags", () => {
    const bad = [
      /curl [^\n]*\|\s*(ba)?sh/,
      /apt(-get)? install rstory/,
      /cargo install --git/,
      /--debug/,
      /--verbose/,
      /blob\/main/,
    ];
    for (const { page, s } of all)
      for (const re of bad) expect(re.test(s), `${page}: ${re}`).toBe(false);
  });

  it("keeps config keys to the real ones in toml examples", () => {
    for (const p of PAGES) {
      for (const b of docs[p.id].blocks) {
        if (b.t !== "code" || b.lang !== "toml") continue;
        for (const m of b.text.matchAll(/^([a-z_]+)\s*=/gm))
          expect(CONFIG_KEYS.has(m[1] ?? ""), `${p.id}: ${m[1]}`).toBe(true);
      }
    }
  });

  it("uses the glossary's terms, not the ones it rules out", () => {
    const banned = /\b(clips?|snippets?|buffers?|pastes?|pasting|pasteboard)\b/i;
    for (const { page, s } of prose)
      expect(banned.test(s.replace("Paste injection", "")), `${page}: ${s.slice(0, 60)}`).toBe(
        false,
      );
  });

  it("does not show the release channels as available", () => {
    const install = docs.install.blocks.flatMap(strings).join("\n");
    expect(install).toContain("[[planned]]");
    expect(
      docs.install.blocks.some((b) => b.t === "code" && b.caption?.startsWith("Planned")),
    ).toBe(true);
  });
});

describe("inline markup", () => {
  it("parses every string without an unknown status token or a broken span", () => {
    for (const { page, s } of all) expect(() => parseInline(s), page).not.toThrow();
  });
  it("leaves no stray backtick in prose", () => {
    for (const { page, s } of prose) {
      const stripped = parseInline(s)
        .filter((i) => i.k === "text")
        .map((i) => (i as { text: string }).text)
        .join("");
      expect(stripped.includes("`"), `${page}: ${s.slice(0, 80)}`).toBe(false);
    }
  });
});
