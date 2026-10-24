import { examplesOf, generated } from "../content/generated";
import type { Block } from "../content/types";
import { useI18n } from "../i18n/context";
import { StatusBadge } from "./Badge";
import { CodeBlock } from "./CodeBlock";
import { InlineText } from "./InlineText";

export const slug = (s: string) =>
  s
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

function Callout({ kind, text }: { kind: "note" | "warning" | "planned"; text: string }) {
  const tone =
    kind === "warning"
      ? "border-warn"
      : kind === "planned"
        ? "border-dashed border-muted"
        : "border-accent";
  return (
    <aside className={`rounded-lg border-l-4 bg-surface px-4 py-3 ${tone}`}>
      {kind === "planned" ? (
        <p className="mb-1">
          <StatusBadge status="planned" />
        </p>
      ) : (
        <p className="mb-1 text-sm font-semibold">{kind === "warning" ? "Warning" : "Note"}</p>
      )}
      <p>
        <InlineText text={text} />
      </p>
    </aside>
  );
}

function CommandHelp() {
  const { ui } = useI18n();
  return (
    <div className="space-y-6">
      <p className="text-sm text-muted">
        {ui.cli.regenerate} (rstory {generated.rstory})
      </p>
      {generated.commands.map((c) => {
        const title = c.name === "rstory" ? "rstory --help" : `rstory ${c.name} --help`;
        const id = `cmd-${slug(c.name)}`;
        return (
          <section key={c.name} aria-labelledby={id} className="space-y-2">
            <h2 id={id} className="mt-0!">
              <code>{title}</code>
            </h2>
            <CodeBlock lang="text" text={c.help} />
            {examplesOf(c.help).length > 0 ? (
              <CodeBlock lang="sh" text={examplesOf(c.help).join("\n")} caption={ui.cli.examples} />
            ) : null}
          </section>
        );
      })}
    </div>
  );
}

function DoctorChecks() {
  const { ui } = useI18n();
  return (
    <div className="table-wrap">
      <table>
        <caption>{ui.cli.regenerate}</caption>
        <thead>
          <tr>
            <th scope="col">{ui.doctor.id}</th>
            <th scope="col">{ui.doctor.status}</th>
            <th scope="col">{ui.doctor.message}</th>
            <th scope="col">{ui.doctor.hint}</th>
          </tr>
        </thead>
        <tbody>
          {generated.doctor.checks.map((k) => (
            <tr key={k.id}>
              <th scope="row">
                <code>{k.id}</code>
              </th>
              <td>{k.status}</td>
              <td>{k.message}</td>
              <td>{k.hint}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function BlockView({ block, index = 0 }: { block: Block; index?: number }) {
  const { ui } = useI18n();
  switch (block.t) {
    case "p":
      return (
        <p>
          <InlineText text={block.text} />
        </p>
      );
    case "h2":
      return <h2 id={slug(block.text)}>{block.text}</h2>;
    case "h3":
      return <h3 id={slug(block.text)}>{block.text}</h3>;
    case "ul":
      return (
        <ul>
          {block.items.map((x) => (
            <li key={x}>
              <InlineText text={x} />
            </li>
          ))}
        </ul>
      );
    case "ol":
      return (
        <ol>
          {block.items.map((x) => (
            <li key={x}>
              <InlineText text={x} />
            </li>
          ))}
        </ol>
      );
    case "code":
      return <CodeBlock lang={block.lang} text={block.text} caption={block.caption} />;
    case "table":
      return (
        <section
          className="table-wrap"
          // A scrollable region must be reachable from the keyboard.
          // biome-ignore lint/a11y/noNoninteractiveTabindex: keyboard access to a scrollable table
          tabIndex={0}
          aria-label={`Table ${index + 1}: ${block.caption ?? block.head.join(", ")}`}
        >
          <table>
            {block.caption ? <caption>{block.caption}</caption> : null}
            <thead>
              <tr>
                {block.head.map((h) => (
                  <th key={h} scope="col">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((r) => (
                <tr key={r.join("|")}>
                  {r.map((c, i) => (
                    <td key={block.head[i]}>
                      <InlineText text={c} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      );
    case "callout":
      return <Callout kind={block.kind} text={block.text} />;
    case "demo":
      return (
        <section
          aria-labelledby={`demo-${block.name}`}
          className="rounded-lg border border-dashed border-line p-4"
        >
          <h3 id={`demo-${block.name}`} className="mt-0!">
            Demo: {block.title}
          </h3>
          <p className="mt-2">
            <StatusBadge status="planned" /> {ui.demo.planned}
          </p>
        </section>
      );
    case "cli":
      return <CommandHelp />;
    case "doctor":
      return <DoctorChecks />;
    case "configdefaults":
      return <CodeBlock lang="toml" text={generated.config_defaults} caption={ui.cli.regenerate} />;
  }
}
