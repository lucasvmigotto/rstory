import type { Maturity } from "./types";

export const SECTIONS = [
  "overview",
  "start",
  "guides",
  "concepts",
  "reference",
  "project",
] as const;
export type SectionId = (typeof SECTIONS)[number];

interface PageMeta {
  id: string;
  /** Hash route. */
  path: string;
  section: SectionId;
  maturity: Maturity;
}

// The structure is locale-independent; titles and text live in each locale's `docs` namespace.
export const PAGES = [
  { id: "overview", path: "/", section: "overview", maturity: "implemented" },
  { id: "install", path: "/install", section: "start", maturity: "planned" },
  { id: "first-run", path: "/first-run", section: "start", maturity: "implemented" },
  { id: "desktops", path: "/desktops", section: "start", maturity: "implemented" },
  { id: "guide-picker", path: "/guides/picker", section: "guides", maturity: "implemented" },
  { id: "guide-cli", path: "/guides/cli", section: "guides", maturity: "implemented" },
  { id: "guide-pause", path: "/guides/pause", section: "guides", maturity: "implemented" },
  { id: "guide-pins", path: "/guides/pins", section: "guides", maturity: "implemented" },
  { id: "guide-tray", path: "/guides/tray", section: "guides", maturity: "implemented" },
  { id: "guide-doctor", path: "/guides/doctor", section: "guides", maturity: "implemented" },
  {
    id: "concept-history",
    path: "/concepts/history",
    section: "concepts",
    maturity: "implemented",
  },
  {
    id: "concept-privacy",
    path: "/concepts/privacy",
    section: "concepts",
    maturity: "implemented",
  },
  { id: "concept-daemon", path: "/concepts/daemon", section: "concepts", maturity: "implemented" },
  {
    id: "concept-backends",
    path: "/concepts/backends",
    section: "concepts",
    maturity: "implemented",
  },
  { id: "ref-cli", path: "/reference/cli", section: "reference", maturity: "implemented" },
  { id: "ref-config", path: "/reference/config", section: "reference", maturity: "implemented" },
  { id: "ref-env", path: "/reference/env", section: "reference", maturity: "implemented" },
  {
    id: "ref-exit-codes",
    path: "/reference/exit-codes",
    section: "reference",
    maturity: "implemented",
  },
  { id: "ref-files", path: "/reference/files", section: "reference", maturity: "implemented" },
  { id: "ref-keys", path: "/reference/keys", section: "reference", maturity: "implemented" },
  { id: "ref-doctor", path: "/reference/doctor", section: "reference", maturity: "implemented" },
  {
    id: "ref-protocol",
    path: "/reference/protocol",
    section: "reference",
    maturity: "implemented",
  },
  {
    id: "project-architecture",
    path: "/project/architecture",
    section: "project",
    maturity: "implemented",
  },
  { id: "project-roadmap", path: "/project/roadmap", section: "project", maturity: "planned" },
  { id: "project-releases", path: "/project/releases", section: "project", maturity: "planned" },
  {
    id: "project-limitations",
    path: "/project/limitations",
    section: "project",
    maturity: "upstream",
  },
  {
    id: "project-accessibility",
    path: "/project/accessibility",
    section: "project",
    maturity: "implemented",
  },
  {
    id: "project-development",
    path: "/project/development",
    section: "project",
    maturity: "implemented",
  },
  { id: "glossary", path: "/glossary", section: "project", maturity: "na" },
] as const satisfies readonly PageMeta[];

export type PageId = (typeof PAGES)[number]["id"];
export type Page = PageMeta & { id: PageId };

export const pageById = (id: PageId): Page => {
  const p = PAGES.find((x) => x.id === id);
  if (!p) throw new Error(`unknown page ${id}`);
  return p;
};

/** `docs/<locale>/<this>.md` */
export const mdName = (p: { path: string }) => (p.path === "/" ? "index" : p.path.slice(1));

export const pageFor = (path: string): Page | undefined => PAGES.find((p) => p.path === path);
