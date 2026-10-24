import { Link } from "react-router-dom";
import { PAGES, type Page, type SectionId } from "../content/pages";
import { useI18n } from "../i18n/context";
import { StatusBadge } from "./Badge";

export function Breadcrumbs({ page }: { page: Page }) {
  const { ui, docs } = useI18n();
  const first = PAGES.find((p) => p.section === page.section);
  return (
    <nav aria-label={ui.nav.breadcrumbs} className="mb-4 text-sm text-muted">
      <ol className="flex flex-wrap gap-x-2">
        <li>
          <Link to={first?.path ?? "/"}>{ui.sections[page.section]}</Link>
          <span aria-hidden="true"> /</span>
        </li>
        <li aria-current="page">{docs[page.id].title}</li>
      </ol>
    </nav>
  );
}

/** The pages of the current section; a disclosure on narrow screens, a sidebar from md up. */
export function SectionNav({ section, current }: { section: SectionId; current: string }) {
  const { ui, docs } = useI18n();
  const pages = PAGES.filter((p) => p.section === section);
  const list = (
    <ul className="space-y-1">
      {pages.map((p) => (
        <li key={p.id}>
          <Link
            to={p.path}
            aria-current={p.id === current ? "page" : undefined}
            className={`flex items-center justify-between gap-2 rounded px-2 py-1 text-sm no-underline hover:bg-surface2 ${p.id === current ? "bg-surface2 font-semibold text-fg" : "text-muted"}`}
          >
            <span>{docs[p.id].title}</span>
            {p.maturity !== "implemented" && p.maturity !== "na" ? (
              <StatusBadge status={p.maturity} />
            ) : null}
          </Link>
        </li>
      ))}
    </ul>
  );
  return (
    <nav aria-label={ui.nav.sectionPages} className="md:w-64 md:shrink-0">
      <details className="md:hidden rounded-lg border border-line p-2">
        <summary className="cursor-pointer text-sm font-semibold">{ui.nav.sectionPages}</summary>
        <div className="mt-2">{list}</div>
      </details>
      <div className="hidden md:sticky md:top-20 md:block">{list}</div>
    </nav>
  );
}
