export const LOCALES = ["en-US"] as const;
export type Locale = (typeof LOCALES)[number];
export const DEFAULT_LOCALE: Locale = "en-US";
export const STORAGE_KEY = "rstory-docs-locale";

const isLocale = (v: string | null | undefined): v is Locale =>
  !!v && (LOCALES as readonly string[]).includes(v);

/** Stored choice, then the browser's languages (exact tag, then language prefix), then English. */
export function detectLocale(stored: string | null, languages: readonly string[]): Locale {
  if (isLocale(stored)) return stored;
  for (const lang of languages) {
    const exact = LOCALES.find((l) => l.toLowerCase() === lang.toLowerCase());
    if (exact) return exact;
    const prefix = LOCALES.find(
      (l) => l.split("-")[0]?.toLowerCase() === lang.split("-")[0]?.toLowerCase(),
    );
    if (prefix) return prefix;
  }
  return DEFAULT_LOCALE;
}

export function readStoredLocale(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

export function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Storage can be blocked; the choice then lasts for the visit only.
  }
}
