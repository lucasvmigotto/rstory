import { useI18n } from "../i18n/context";
import { LOCALES, type Locale } from "../i18n/locales";

const NAMES: Record<Locale, string> = { "en-US": "English (US)" };

export function LanguageSwitcher() {
  const { ui, locale, setLocale } = useI18n();
  return (
    <label className="flex items-center gap-2 text-sm">
      <span className="sr-only">{ui.language.label}</span>
      <select
        value={locale}
        onChange={(e) => setLocale(e.target.value as Locale)}
        className="rounded border border-line bg-bg px-2 py-1"
      >
        {LOCALES.map((l) => (
          <option key={l} value={l} lang={l}>
            {NAMES[l]}
          </option>
        ))}
      </select>
    </label>
  );
}
