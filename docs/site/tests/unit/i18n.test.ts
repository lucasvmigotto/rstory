import { describe, expect, it } from "vitest";
import { PAGES } from "../../src/content/pages";
import { DEFAULT_LOCALE, detectLocale, LOCALES } from "../../src/i18n/locales";
import { MESSAGES } from "../../src/i18n/registry";

describe("locale detection", () => {
  it("prefers a stored locale", () => expect(detectLocale("en-US", ["fr"])).toBe("en-US"));
  it("ignores an unknown stored locale", () => expect(detectLocale("xx", ["en-GB"])).toBe("en-US"));
  it("matches the language prefix of the browser's languages", () =>
    expect(detectLocale(null, ["en-GB"])).toBe("en-US"));
  it("falls back to English", () => expect(detectLocale(null, ["de", "ja"])).toBe(DEFAULT_LOCALE));
  it("has en-US as the primary locale", () => expect(LOCALES[0]).toBe("en-US"));
});

describe("locale content", () => {
  for (const locale of LOCALES) {
    it(`${locale} has a titled page for every route`, () => {
      for (const p of PAGES) {
        const d = MESSAGES[locale].docs[p.id];
        expect(d.title.length, p.id).toBeGreaterThan(0);
        expect(d.summary.length, p.id).toBeGreaterThan(0);
        expect(d.blocks.length, p.id).toBeGreaterThan(0);
      }
    });
    it(`${locale} has a label for every maturity`, () => {
      for (const m of [
        "implemented",
        "partial",
        "planned",
        "unavailable",
        "upstream",
        "na",
      ] as const) {
        expect(MESSAGES[locale].ui.maturity[m].label).toBeTruthy();
      }
    });
  }
});
