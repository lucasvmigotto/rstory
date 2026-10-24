import type { Maturity } from "../content/types";
import { useI18n } from "../i18n/context";

const GLYPH: Record<Maturity, string> = {
  implemented: "✓",
  partial: "◐",
  planned: "○",
  unavailable: "✕",
  upstream: "!",
  na: "·",
};
const TONE: Record<Maturity, string> = {
  implemented: "border-accent text-accent",
  partial: "border-warn text-warn",
  planned: "border-dashed border-muted text-muted",
  unavailable: "border-danger text-danger",
  upstream: "border-warn text-warn",
  na: "border-line text-muted",
};

/** Status is always a word plus a shape, never color alone. */
export function StatusBadge({ status, className = "" }: { status: Maturity; className?: string }) {
  const { ui } = useI18n();
  const m = ui.maturity[status];
  return (
    <span
      title={m.hint}
      className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-semibold whitespace-nowrap align-middle ${TONE[status]} ${className}`}
    >
      <span aria-hidden="true">{GLYPH[status]}</span>
      {m.label}
    </span>
  );
}
