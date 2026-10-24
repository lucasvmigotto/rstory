import { describe, expect, it } from "vitest";
import { resolveVersions, sourceUrl } from "../../scripts/footer-versions";

const base = {
  repo: "https://github.com/lucasvmigotto/rstory",
  project: "rstory",
  version: "0.1.0",
  docsVersion: "0.2.0",
};
const SHA = "abc1234def5678abc1234def5678abc1234def56";

describe("footer version resolution (one case per row of the version table)", () => {
  it("HEAD is the release tag: plain version, linked to the tag", () => {
    const v = resolveVersions({ ...base, git: { sha: SHA, tag: "0.1.0", lastTag: "0.1.0" } });
    expect(v.entries.map((e) => e.text)).toEqual(["rstory@v0.1.0", "docs@v0.2.0"]);
    expect(v.entries[0]?.href).toBe(`${base.repo}/tree/0.1.0`);
    expect(v.entries[1]?.href).toBe(`${base.repo}/tree/0.1.0/docs/site`);
    expect(v.ref).toBe("0.1.0");
  });

  it("HEAD is past the tag: the tag's version plus the commit, linked to the commit", () => {
    const v = resolveVersions({ ...base, git: { sha: SHA, lastTag: "0.1.0" } });
    expect(v.entries.map((e) => e.text)).toEqual(["rstory@v0.1.0+abc1234", "docs@v0.2.0+abc1234"]);
    expect(v.entries[0]?.href).toBe(`${base.repo}/tree/${SHA}`);
    expect(v.entries[1]?.href).toBe(`${base.repo}/tree/${SHA}/docs/site`);
    expect(v.ref).toBe(SHA);
  });

  it("no release tag yet: the manifest version plus the commit", () => {
    const v = resolveVersions({ ...base, git: { sha: SHA } });
    expect(v.entries[0]?.text).toBe("rstory@v0.1.0+abc1234");
    expect(v.entries[0]?.href).toBe(`${base.repo}/tree/${SHA}`);
    expect(v.ref).toBe(SHA);
  });

  it("no git (a source tarball): the manifest version, no links", () => {
    const v = resolveVersions({ ...base, git: {} });
    expect(v.entries.map((e) => e.text)).toEqual(["rstory@v0.1.0", "docs@v0.2.0"]);
    expect(v.entries.every((e) => e.href === undefined)).toBe(true);
    expect(v.ref).toBeNull();
  });

  it("blank git facts (empty env variables) count as absent", () => {
    const v = resolveVersions({ ...base, git: { sha: "", tag: " ", lastTag: "" } });
    expect(v.ref).toBeNull();
  });
});

describe("source links", () => {
  it("follow the build's ref, never blob/main when git is known", () => {
    expect(sourceUrl(base.repo, "0.1.0", "LICENSE")).toBe(`${base.repo}/blob/0.1.0/LICENSE`);
    expect(sourceUrl(base.repo, SHA, "LICENSE")).toBe(`${base.repo}/blob/${SHA}/LICENSE`);
  });
  it("fall back to the default branch only without git", () => {
    expect(sourceUrl(base.repo, null, "LICENSE")).toBe(`${base.repo}/blob/main/LICENSE`);
  });
});
