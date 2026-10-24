import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const text = readFileSync(
  resolve(import.meta.dirname, "../../../../.github/workflows/docs.yml"),
  "utf8",
);
const lines = text.split("\n").filter((l) => !/^\s*#/.test(l));
const body = lines.join("\n");
const deploy = body.slice(body.indexOf("\n  deploy:"));

describe("docs workflow", () => {
  it("pins every action by full commit SHA", () => {
    const uses = [...body.matchAll(/^\s*-?\s*uses:\s*(\S+)/gm)].map((m) => m[1] ?? "");
    expect(uses.length).toBeGreaterThan(0);
    for (const u of uses) expect(u.startsWith("./") || /@[0-9a-f]{40}$/.test(u), u).toBe(true);
  });

  it("has no permissions at the top and no pull_request_target", () => {
    expect(body).toMatch(/^permissions: \{\}$/m);
    expect(body).not.toContain("pull_request_target");
  });

  it("verifies pull requests on the site's sources", () => {
    for (const p of ["docs/site/**", "docs/product/**", "specs/**", "src/**", "Cargo.toml"]) {
      expect(body).toContain(`- "${p}"`);
    }
    expect(body).toContain("make docs-check");
    expect(body).toContain("make docs-build");
  });

  it("deploys only after a successful release run for a tag push or a dispatch, never from a push or PR", () => {
    expect(body).not.toMatch(/^\s{2}push:/m);
    expect(body).toMatch(/workflow_run:\s*\n\s*workflows: \[release\]/);
    expect(deploy).toContain("github.event_name == 'workflow_run'");
    expect(deploy).toContain("github.event.workflow_run.conclusion == 'success'");
    expect(deploy).toContain("github.event.workflow_run.event == 'push'");
    expect(deploy).toContain("github.event.workflow_run.event == 'repository_dispatch'");
    expect(deploy).toContain("github.event.workflow_run.display_title");
    expect(deploy).not.toContain("head_branch");
    expect(deploy).toContain("name: rstory-docs");
    expect(deploy).toContain("[0-9]*.[0-9]*.[0-9]*");
  });

  it("scopes every R2 sync to the project prefix and reads only the R2 secret", () => {
    for (const m of deploy.matchAll(/aws s3 (sync|cp) \S+ "s3:\/\/([^"]+)"/g)) {
      expect(m[2], m[0]).toContain("${R2_PREFIX}");
    }
    // The one secrets read SC-001 allows (.github/scripts/no-publish-token.sh): the R2 key.
    const reads = [...body.matchAll(/\$\{\{\s*secrets\.([A-Z0-9_]+)\s*\}\}/g)].map((m) => m[1]);
    expect(reads).toEqual(["CLOUDFLARE_R2_ACCOUNT_SECRET"]);
    expect(body).not.toMatch(/ghp_|AKIA[0-9A-Z]{8}/);
  });
});
