import type { UiStrings } from "../types";

export const ui: UiStrings = {
  siteName: "rstory",
  tagline: "A small, private clipboard history for Linux",
  skipToContent: "Skip to content",
  sections: {
    overview: "Overview",
    start: "Get started",
    guides: "Guides",
    concepts: "Concepts",
    reference: "Reference",
    project: "Project",
  },
  nav: {
    label: "Documentation",
    sectionPages: "Pages in this section",
    breadcrumbs: "Breadcrumb",
    onThisPage: "On this page",
  },
  search: {
    open: "Search",
    label: "Search the documentation",
    placeholder: "Search pages",
    close: "Close search",
    none: "No pages match.",
    count: (n) => (n === 1 ? "1 page matches." : `${n} pages match.`),
  },
  language: { label: "Language" },
  theme: { legend: "Theme", light: "Light theme", dark: "Dark theme" },
  footer: {
    versions: "Versions",
    license: "GPL v3+",
    licenseTitle: "GNU General Public License, version 3 or later",
  },
  copy: { button: "Copy", done: "Copied" },
  maturity: {
    implemented: {
      label: "Implemented",
      markdown: "**Implemented.**",
      hint: "Works today, traced to code and tests.",
    },
    partial: {
      label: "Partially implemented",
      markdown: "**Partially implemented — see the notes.**",
      hint: "Some of it works today.",
    },
    planned: {
      label: "Planned",
      markdown: "**Planned — not implemented yet.**",
      hint: "Intended, not built or not released.",
    },
    unavailable: {
      label: "Unavailable",
      markdown: "**Unavailable — not supported today.**",
      hint: "Mentioned, but the code does not do it.",
    },
    upstream: {
      label: "Upstream limitation",
      markdown: "**Upstream limitation — outside rstory's control.**",
      hint: "A limit of the desktop, protocol or application.",
    },
    na: { label: "Reference", markdown: "**Reference.**", hint: "Not a feature." },
  },
  demo: {
    planned:
      "Recorded demo: not recorded yet. The terminal recording is planned; the commands below are what it will show.",
  },
  cli: {
    exit: "Exit code",
    examples: "Examples",
    regenerate: "Generated from the built binary's --help output by make docs-reference.",
  },
  doctor: { id: "Check", status: "Status", message: "Message without a session", hint: "Hint" },
  notFound: {
    title: "Page not found",
    body: "This page does not exist. Use the navigation or search to find what you need.",
    home: "Go to the overview",
  },
  llm: {
    summary:
      "rstory is a small, private clipboard history for Linux. One static Rust binary records what you copy on Wayland and X11, never uses the network, and skips copies that apps mark as sensitive.",
    optional: "Optional",
  },
};
