import { Link, useLocation } from "react-router-dom";
import { PAGES, pageFor, SECTIONS } from "../content/pages";
import { useI18n } from "../i18n/context";
import { LanguageSwitcher } from "./LanguageSwitcher";
import { Search } from "./Search";

export function Header() {
  const { ui } = useI18n();
  const { pathname } = useLocation();
  return (
    <header className="sticky top-0 z-30 border-b border-line bg-bg/95 backdrop-blur">
      <div className="mx-auto flex max-w-6xl flex-wrap items-center gap-x-6 gap-y-2 px-4 py-3">
        <Link
          to="/"
          className="flex items-center gap-2 font-mono text-lg font-bold text-fg no-underline"
        >
          <img src={`${import.meta.env.BASE_URL}favicon.svg`} alt="" width="24" height="24" />
          {ui.siteName}
        </Link>
        <nav
          aria-label={ui.nav.label}
          className="order-last w-full md:order-none md:w-auto md:flex-1"
        >
          <ul className="flex flex-wrap gap-x-4 gap-y-1">
            {SECTIONS.map((s) => {
              const first = PAGES.find((p) => p.section === s);
              const current = first && pageFor(pathname)?.section === s;
              return first ? (
                <li key={s}>
                  <Link
                    to={first.path}
                    aria-current={current ? "true" : undefined}
                    className={`text-sm ${current ? "font-semibold text-fg underline" : "text-muted no-underline hover:text-fg"}`}
                  >
                    {ui.sections[s]}
                  </Link>
                </li>
              ) : null;
            })}
          </ul>
        </nav>
        <div className="ml-auto flex items-center gap-3">
          <Search />
          <LanguageSwitcher />
        </div>
      </div>
    </header>
  );
}
