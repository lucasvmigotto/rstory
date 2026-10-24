import { Link } from "react-router-dom";
import { parseInline, resolveLink } from "../content/inline";
import { src } from "../site-meta";
import { StatusBadge } from "./Badge";

const env = { repoFile: src, mdUrl: (p: string) => p };

export function InlineText({ text }: { text: string }) {
  return (
    <>
      {parseInline(text).map((n, i) => {
        const key = `${i}-${n.k}`;
        switch (n.k) {
          case "text":
            return <span key={key}>{n.text}</span>;
          case "code":
            return <code key={key}>{n.text}</code>;
          case "bold":
            return <strong key={key}>{n.text}</strong>;
          case "status":
            return <StatusBadge key={key} status={n.status} />;
          case "link": {
            const href = resolveLink(n.href, env, "web");
            return href.startsWith("#/") ? (
              <Link key={key} to={href.slice(1)}>
                {n.text}
              </Link>
            ) : (
              <a key={key} href={href} rel="noreferrer">
                {n.text}
              </a>
            );
          }
          default:
            return null;
        }
      })}
    </>
  );
}
