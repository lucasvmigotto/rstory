import { useI18n } from "../i18n/context";
import { SITE, src } from "../site-meta";
import { ThemeSwitch } from "./ThemeSwitch";
import { VersionBadge } from "./VersionBadge";

export function Footer() {
  const { ui } = useI18n();
  const entries = SITE.versions.entries;
  return (
    <footer className="mt-16 border-t border-line">
      <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-center gap-x-6 gap-y-3 px-4 py-6 text-sm text-muted">
        <ul
          aria-label={ui.footer.versions}
          className="m-0 flex list-none flex-wrap items-center gap-x-2 p-0 font-mono"
        >
          {entries.map((e, i) => (
            <li key={e.name} className="inline-flex gap-2">
              {i > 0 ? <span aria-hidden="true">·</span> : null}
              <VersionBadge entry={e} />
            </li>
          ))}
        </ul>
        <a href={src("LICENSE")} title={ui.footer.licenseTitle} rel="noreferrer">
          <span aria-hidden="true">⚖ </span>
          {ui.footer.license}
        </a>
        <ThemeSwitch />
      </div>
    </footer>
  );
}
