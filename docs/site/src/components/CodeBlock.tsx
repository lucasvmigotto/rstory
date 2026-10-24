import { useState } from "react";
import { useI18n } from "../i18n/context";

export function CodeCopyButton({ text }: { text: string }) {
  const { ui } = useI18n();
  const [done, setDone] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setDone(true);
      setTimeout(() => setDone(false), 1500);
    } catch {
      // Clipboard access can be denied; the text stays selectable.
    }
  };
  return (
    <button
      type="button"
      onClick={copy}
      className="absolute top-2 right-2 rounded border border-line bg-surface2 px-2 py-0.5 text-xs text-fg hover:border-accent"
    >
      {done ? ui.copy.done : ui.copy.button}
      <span className="sr-only"> code</span>
    </button>
  );
}

export function CodeBlock({
  lang,
  text,
  caption,
}: {
  lang: string;
  text: string;
  caption?: string;
}) {
  return (
    <figure className="m-0">
      {caption ? <figcaption className="mb-1 text-sm text-muted">{caption}</figcaption> : null}
      <div className="relative">
        <pre
          // Scrollable regions must be reachable from the keyboard.
          // biome-ignore lint/a11y/noNoninteractiveTabindex: keyboard access to a scrollable code block
          tabIndex={0}
          className="overflow-x-auto rounded-lg border border-line bg-surface p-4 pr-16 font-mono text-sm leading-relaxed"
        >
          <code data-lang={lang}>{text}</code>
        </pre>
        <CodeCopyButton text={text} />
      </div>
    </figure>
  );
}
