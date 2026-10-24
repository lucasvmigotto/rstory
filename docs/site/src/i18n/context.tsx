import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { detectLocale, type Locale, readStoredLocale, storeLocale } from "./locales";
import { MESSAGES } from "./registry";
import type { Messages } from "./types";

interface I18n extends Messages {
  locale: Locale;
  setLocale: (l: Locale) => void;
}

const Ctx = createContext<I18n | null>(null);

export function I18nProvider({ children, initial }: { children: ReactNode; initial?: Locale }) {
  const [locale, setLocaleState] = useState<Locale>(
    () => initial ?? detectLocale(readStoredLocale(), navigator.languages ?? [navigator.language]),
  );
  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);
  const setLocale = useCallback((l: Locale) => {
    storeLocale(l);
    setLocaleState(l);
  }, []);
  const value = useMemo(() => ({ locale, setLocale, ...MESSAGES[locale] }), [locale, setLocale]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useI18n(): I18n {
  const v = useContext(Ctx);
  if (!v) throw new Error("useI18n outside I18nProvider");
  return v;
}
