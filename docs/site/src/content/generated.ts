import data from "./generated/cli-help.json";

export interface GeneratedHelp {
  rstory: string;
  commands: { name: string; args: string[]; exit: number; help: string }[];
  config_defaults: string;
  doctor: { exit: number; checks: { id: string; status: string; message: string; hint: string }[] };
  completions: Record<string, boolean>;
}

export const generated = data as unknown as GeneratedHelp;

/** The `Examples:` block of a command's help, as lines (empty when it has none). */
export function examplesOf(help: string): string[] {
  const i = help.indexOf("\nExamples:\n");
  if (i < 0) return [];
  return help
    .slice(i + "\nExamples:\n".length)
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean)
    .filter((l) => !l.startsWith("#"));
}
