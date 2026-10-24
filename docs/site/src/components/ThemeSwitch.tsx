import { useEffect, useState } from "react";
import { useI18n } from "../i18n/context";

type Theme = "light" | "dark";
const KEY = "rstory-docs-theme";

function initial(): Theme {
  const set = document.documentElement.dataset.theme;
  return set === "light" ? "light" : "dark";
}

export function ThemeSwitch() {
  const { ui } = useI18n();
  const [theme, setTheme] = useState<Theme>(initial);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  const choose = (t: Theme) => {
    setTheme(t);
    try {
      localStorage.setItem(KEY, t);
    } catch {
      // Storage can be blocked; the choice then lasts for the visit only.
    }
  };
  const btn = (t: Theme, label: string, icon: string) => (
    <button
      type="button"
      aria-label={label}
      title={label}
      aria-pressed={theme === t}
      onClick={() => choose(t)}
      className={`h-8 w-8 rounded border text-base ${theme === t ? "border-accent text-fg" : "border-line text-muted"}`}
    >
      <span aria-hidden="true">{icon}</span>
    </button>
  );
  return (
    <fieldset className="m-0 flex gap-1 border-0 p-0">
      <legend className="sr-only">{ui.theme.legend}</legend>
      {btn("light", ui.theme.light, "☀")}
      {btn("dark", ui.theme.dark, "☾")}
    </fieldset>
  );
}
