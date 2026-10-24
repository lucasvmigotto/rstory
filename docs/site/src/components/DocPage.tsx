import { useEffect, useRef } from "react";
import { mdName, type Page } from "../content/pages";
import { useI18n } from "../i18n/context";
import { SITE } from "../site-meta";
import { StatusBadge } from "./Badge";
import { BlockView } from "./Blocks";
import { Breadcrumbs, SectionNav } from "./Nav";

export function DocPage({ page }: { page: Page }) {
  const { docs, ui, locale } = useI18n();
  const content = docs[page.id];
  const h1 = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    document.title =
      page.id === "overview" ? `${ui.siteName} documentation` : `${content.title} · ${ui.siteName}`;
    // Focus the heading on navigation so screen readers announce the new page.
    h1.current?.focus({ preventScroll: true });
    window.scrollTo(0, 0);
    const link = document.createElement("link");
    link.rel = "alternate";
    link.type = "text/markdown";
    link.href = `${SITE.siteUrl}docs/${locale}/${mdName(page)}.md`;
    link.dataset.page = "md";
    document.head.append(link);
    return () => link.remove();
  }, [page, content.title, ui.siteName, locale]);

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-6 px-4 pt-6 md:flex-row md:gap-10">
      <SectionNav section={page.section} current={page.id} />
      <article className="fade-in min-w-0 max-w-3xl flex-1">
        <Breadcrumbs page={page} />
        <div className="prose">
          <h1 ref={h1} tabIndex={-1} className="outline-none">
            {content.title}
          </h1>
          <p className="flex flex-wrap items-center gap-2 text-muted">
            <StatusBadge status={page.maturity} />
            <span>{content.summary}</span>
          </p>
          {content.blocks.map((b, i) => (
            // Content is static and ordered, so the index is a stable key.
            // biome-ignore lint/suspicious/noArrayIndexKey: static content
            <BlockView key={i} block={b} index={i} />
          ))}
        </div>
      </article>
    </div>
  );
}
