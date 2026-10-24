import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(import.meta.dirname, "../../src/styles.css"), "utf8");

function theme(selector: RegExp): Record<string, string> {
  const block = selector.exec(css)?.[1] ?? "";
  return Object.fromEntries(
    [...block.matchAll(/--([a-z0-9-]+):\s*(#[0-9a-f]{6})/g)].map((m) => [m[1], m[2]]),
  );
}
const lum = (hex: string) => {
  const c = [1, 3, 5].map((i) => {
    const v = Number.parseInt(hex.slice(i, i + 2), 16) / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * (c[0] ?? 0) + 0.7152 * (c[1] ?? 0) + 0.0722 * (c[2] ?? 0);
};
const ratio = (a: string, b: string) => {
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
};

// axe cannot compute colors in a DOM without layout, so the token pairs are checked here.
describe("color tokens meet WCAG AA for text", () => {
  const themes = {
    dark: theme(/:root,\s*:root\[data-theme="dark"\]\s*\{([^}]*)\}/),
    light: theme(/:root\[data-theme="light"\]\s*\{([^}]*)\}/),
  };
  for (const [name, t] of Object.entries(themes)) {
    for (const fg of ["fg", "muted", "accent", "warn", "danger"]) {
      for (const bg of ["bg", "surface", "surface-2"]) {
        it(`${name}: ${fg} on ${bg} is at least 4.5:1`, () => {
          expect(t[fg] && t[bg]).toBeTruthy();
          expect(ratio(t[fg] as string, t[bg] as string)).toBeGreaterThanOrEqual(4.5);
        });
      }
    }
    it(`${name}: button text on the accent is at least 4.5:1`, () => {
      expect(ratio(t["accent-fg"] as string, t.accent as string)).toBeGreaterThanOrEqual(4.5);
    });
  }
});
