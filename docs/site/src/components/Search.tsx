import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { parseInline } from "../content/inline";
import { PAGES } from "../content/pages";
import type { Block } from "../content/types";
import { useI18n } from "../i18n/context";

const plain = (s: string) =>
  parseInline(s)
    .map((i) => (i.k === "status" ? "" : i.k === "link" ? i.text : i.text))
    .join("");

function blockText(b: Block): string {
  switch (b.t) {
    case "p":
    case "h2":
    case "h3":
    case "callout":
      return plain(b.text);
    case "ul":
    case "ol":
      return b.items.map(plain).join(" ");
    case "table":
      return [...b.head, ...b.rows.flat().map(plain)].join(" ");
    case "code":
      return b.text;
    default:
      return "";
  }
}

export function Search() {
  const { ui, docs } = useI18n();
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const btn = useRef<HTMLButtonElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const panelId = useId();

  const index = useMemo(
    () =>
      PAGES.map((p) => {
        const d = docs[p.id];
        return {
          page: p,
          title: d.title,
          summary: d.summary,
          hay: `${d.title} ${d.summary} ${d.blocks.map(blockText).join(" ")}`.toLowerCase(),
        };
      }),
    [docs],
  );

  const results = useMemo(() => {
    const terms = q.toLowerCase().split(/\s+/).filter(Boolean);
    if (terms.length === 0) return [];
    return index
      .filter((e) => terms.every((t) => e.hay.includes(t)))
      .sort(
        (a, b) =>
          Number(terms.every((t) => b.title.toLowerCase().includes(t))) -
          Number(terms.every((t) => a.title.toLowerCase().includes(t))),
      );
  }, [index, q]);

  useEffect(() => {
    if (open) input.current?.focus();
  }, [open]);

  const close = () => {
    setOpen(false);
    setQ("");
    btn.current?.focus();
  };

  return (
    <div className="relative">
      <button
        ref={btn}
        type="button"
        aria-expanded={open}
        aria-controls={panelId}
        onClick={() => (open ? close() : setOpen(true))}
        className="rounded border border-line px-3 py-1 text-sm hover:border-accent"
      >
        {ui.search.open}
      </button>
      {open ? (
        <search
          id={panelId}
          className="fixed inset-x-4 top-16 z-40 mx-auto max-w-xl rounded-lg border border-line bg-surface p-3 shadow-lg"
          onKeyDown={(e) => {
            if (e.key === "Escape") close();
          }}
        >
          <label className="block text-sm font-semibold" htmlFor={`${panelId}-q`}>
            {ui.search.label}
          </label>
          <input
            id={`${panelId}-q`}
            ref={input}
            type="search"
            value={q}
            placeholder={ui.search.placeholder}
            onChange={(e) => setQ(e.target.value)}
            className="mt-1 w-full rounded border border-line bg-bg px-3 py-2"
          />
          <p role="status" className="mt-2 text-sm text-muted">
            {q.trim() ? (results.length ? ui.search.count(results.length) : ui.search.none) : ""}
          </p>
          <ul className="mt-1 max-h-72 overflow-y-auto">
            {results.map((r) => (
              <li key={r.page.id}>
                <Link
                  to={r.page.path}
                  onClick={close}
                  className="block rounded px-2 py-1.5 hover:bg-surface2"
                >
                  <span className="font-semibold">{r.title}</span>
                  <span className="block text-sm text-muted">{r.summary}</span>
                </Link>
              </li>
            ))}
          </ul>
          <button
            type="button"
            onClick={close}
            className="mt-2 rounded border border-line px-3 py-1 text-sm"
          >
            {ui.search.close}
          </button>
        </search>
      ) : null}
    </div>
  );
}
