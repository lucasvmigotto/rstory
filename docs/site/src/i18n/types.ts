import type { PageId, SectionId } from "../content/pages";
import type { Maturity, PageContent } from "../content/types";

/** UI chrome: everything around the documentation. */
export interface UiStrings {
  siteName: string;
  tagline: string;
  skipToContent: string;
  sections: Record<SectionId, string>;
  nav: { label: string; sectionPages: string; breadcrumbs: string; onThisPage: string };
  search: {
    open: string;
    label: string;
    placeholder: string;
    close: string;
    none: string;
    count: (n: number) => string;
  };
  language: { label: string };
  theme: { legend: string; light: string; dark: string };
  footer: { versions: string; license: string; licenseTitle: string };
  copy: { button: string; done: string };
  maturity: Record<Maturity, { label: string; markdown: string; hint: string }>;
  demo: { planned: string };
  cli: { exit: string; examples: string; regenerate: string };
  doctor: { id: string; status: string; message: string; hint: string };
  notFound: { title: string; body: string; home: string };
  llm: { summary: string; optional: string };
}

/** Documentation content: one entry per page, so a missing translation is a type error. */
export type DocsContent = Record<PageId, PageContent>;

export interface Messages {
  ui: UiStrings;
  docs: DocsContent;
}
