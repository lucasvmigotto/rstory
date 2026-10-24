// Checks the built site: the footer's versions, no source link to blob/main, and that every link in
// llms.txt resolves to a file in dist/. Run by `bun run build` after the LLM files are written.
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const dist = resolve(root, "dist");
const siteUrl = "https://docs.lucasvmigotto.me/rstory/";
const cargo = readFileSync(resolve(root, "../../Cargo.toml"), "utf8");
const version = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "";
const docsVersion = (
  JSON.parse(readFileSync(resolve(root, "package.json"), "utf8")) as { version: string }
).version;

const problems: string[] = [];
const walk = (dir: string): string[] =>
  readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? walk(p) : [p];
  });
const files = walk(dist);

// Footer: every versioned part, then docs.
const js = files
  .filter((f) => f.endsWith(".js") && f.includes("/assets/"))
  .map((f) => readFileSync(f, "utf8"))
  .join("\n");
if (!js.includes(`rstory@v`) || !js.includes(`docs@v${docsVersion}`))
  problems.push("footer versions missing from the bundle");
if (!new RegExp(`rstory@v(${version.replaceAll(".", "\\.")}|\\d+\\.\\d+\\.\\d+)`).test(js))
  problems.push("rstory version missing from the bundle");

// Source links follow the build's ref.
for (const f of files) {
  if (!/\.(js|md|txt|html)$/.test(f)) continue;
  if (readFileSync(f, "utf8").includes("blob/main")) problems.push(`${f} links to blob/main`);
}

// llms.txt links resolve.
const llms = readFileSync(join(dist, "llms.txt"), "utf8");
for (const m of llms.matchAll(/\]\((https:[^)]+)\)/g)) {
  const url = m[1] ?? "";
  if (!url.startsWith(siteUrl)) continue;
  if (!existsSync(join(dist, url.slice(siteUrl.length))))
    problems.push(`llms.txt link does not resolve: ${url}`);
}
if (!readFileSync(join(dist, "index.html"), "utf8").includes('type="text/markdown"')) {
  problems.push("index.html does not advertise the Markdown alternate");
}

if (problems.length > 0) {
  console.error(problems.join("\n"));
  process.exit(1);
}
console.log(`dist ok: rstory ${version}, docs ${docsVersion}, ${files.length} files`);
