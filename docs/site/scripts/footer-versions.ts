// Resolves the footer's version entries from git facts and manifests: a pure function, tested per row
// of the version table (tests/unit/footer-versions.test.ts). Git facts come from the host (the Makefile
// passes RSTORY_GIT_*), because a worktree's .git is not visible inside the container.

export interface GitFacts {
  /** Full commit sha of the build, if known. */
  sha?: string;
  /** The release tag that HEAD is exactly on (plain SemVer, no `v`). */
  tag?: string;
  /** The newest release tag reachable from HEAD. */
  lastTag?: string;
}

export interface VersionEntry {
  name: string;
  /** Display text, `name@vX.Y.Z[+sha]`. */
  text: string;
  /** Repository tree of the ref the build came from; absent without git. */
  href?: string;
}

export interface VersionInput {
  repo: string;
  project: string;
  /** Version from Cargo.toml. */
  version: string;
  /** Version from docs/site/package.json. */
  docsVersion: string;
  git: GitFacts;
}

export interface Versions {
  entries: VersionEntry[];
  /** The ref source links use: the tag, else the commit; null without git. */
  ref: string | null;
}

const short = (sha: string) => sha.slice(0, 7);

export function resolveVersions(input: VersionInput): Versions {
  const { repo, project, version, docsVersion, git } = input;
  const sha = git.sha?.trim() || undefined;
  const tag = git.tag?.trim() || undefined;
  const lastTag = git.lastTag?.trim() || undefined;

  if (tag) {
    return {
      ref: tag,
      entries: [
        { name: project, text: `${project}@v${tag}`, href: `${repo}/tree/${tag}` },
        { name: "docs", text: `docs@v${docsVersion}`, href: `${repo}/tree/${tag}/docs/site` },
      ],
    };
  }
  if (sha) {
    // Past the last tag: that tag's version plus the commit. With no tag yet: the manifest's.
    const base = lastTag ?? version;
    return {
      ref: sha,
      entries: [
        { name: project, text: `${project}@v${base}+${short(sha)}`, href: `${repo}/tree/${sha}` },
        {
          name: "docs",
          text: `docs@v${docsVersion}+${short(sha)}`,
          href: `${repo}/tree/${sha}/docs/site`,
        },
      ],
    };
  }
  return {
    ref: null,
    entries: [
      { name: project, text: `${project}@v${version}` },
      { name: "docs", text: `docs@v${docsVersion}` },
    ],
  };
}

/** Link to a file in the repository at the build's ref (never `blob/main` when git is known). */
export function sourceUrl(
  repo: string,
  ref: string | null,
  path: string,
  fallback = "main",
): string {
  return `${repo}/blob/${ref ?? fallback}/${path}`;
}
