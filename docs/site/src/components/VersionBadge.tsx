import type { VersionEntry } from "../../scripts/footer-versions";

export function VersionBadge({ entry }: { entry: VersionEntry }) {
  return entry.href ? (
    <a href={entry.href} rel="noreferrer">
      {entry.text}
    </a>
  ) : (
    <span>{entry.text}</span>
  );
}
