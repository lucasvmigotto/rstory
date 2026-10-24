export type Maturity = "implemented" | "partial" | "planned" | "unavailable" | "upstream" | "na";

/** Inline markup in every string: `code`, **bold**, [text](#/route | src:repo/path | https://...), [[status]]. */
export type Block =
  | { t: "p"; text: string }
  | { t: "h2"; text: string }
  | { t: "h3"; text: string }
  | { t: "ul"; items: string[] }
  | { t: "ol"; items: string[] }
  | { t: "code"; lang: string; text: string; caption?: string }
  | { t: "table"; head: string[]; rows: string[][]; caption?: string }
  | { t: "callout"; kind: "note" | "warning" | "planned"; text: string }
  | { t: "demo"; name: string; title: string }
  /** The CLI reference, rendered from generated/cli-help.json. */
  | { t: "cli" }
  /** The doctor checks, rendered from generated/cli-help.json. */
  | { t: "doctor" }
  /** What `rstory config` prints with no config file, from generated/cli-help.json. */
  | { t: "configdefaults" };

export interface PageContent {
  title: string;
  summary: string;
  blocks: Block[];
}
