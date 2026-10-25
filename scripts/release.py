#!/usr/bin/env python3
"""Plan and prepare a release from Conventional Commits.

Reads the commits since the last plain SemVer tag (X.Y.Z, no "v") reachable from HEAD and
decides whether they warrant a release:

  feat, fix, perf, refactor     -> patch  (minor for feat once the version is 1.0.0 or later)
  "type!:" or a BREAKING CHANGE -> minor  (major once the version is 1.0.0 or later)
  anything else (docs, chore, ci, test, style, build, revert, ...) -> none

While the version is 0.x, cargo treats a minor bump as the breaking one, so breaking changes
raise the minor and everything else the patch. 1.0.0 is a deliberate step: --version 1.0.0.

Merge-commit subjects and "chore(release): ..." commits are ignored (the commits a merge brings
in are counted on their own).

It bumps, in place, the one version of the project wherever it is written (TARGETS below; each
must equal the version at the last tag, so a hand edit is never overwritten):
  Cargo.toml, Cargo.lock, extension/metadata.json (version-name) and the docs site's generated
  CLI reference (cli-help.json, which a CI check keeps equal to the binary's --version).
and CHANGELOG.md: a "## Unreleased" section is renamed to "## X.Y.Z — YYYY-MM-DD" (hand-written
notes win, nothing is added), otherwise a section is generated from the commits.

Usage (run from anywhere inside the repository):
  release.py [--dry-run] [--date YYYY-MM-DD] [--version X.Y.Z]
  release.py --notes [X.Y.Z]

Output and exit codes:
  default    writes the files; prints the new version alone on stdout, the plan on stderr; exit 0
  --dry-run  prints the plan and the changelog section on stdout, writes nothing; exit 0
  --notes    prints one release's CHANGELOG section body (default: the newest) for the GitHub
             release; exit 0
  exit 3     no release needed (nothing releasable since the last tag)
  exit 1     error (not a git repository, versions out of step, ...)
  exit 2     bad arguments
"""

import argparse
import datetime
import os
import re
import subprocess
import sys

NO_RELEASE = 3
RANK = {"none": 0, "patch": 1, "minor": 2, "major": 3}
TYPE_BUMP = {"feat": "minor", "fix": "patch", "perf": "patch", "refactor": "patch"}
HEADER_RE = re.compile(r"^(?P<type>[a-z]+)(?:\((?P<scope>[^)]*)\))?(?P<bang>!)?: (?P<desc>.+)$")
BREAKING_RE = re.compile(r"^BREAKING[ -]CHANGE: *(?P<text>.+)$", re.M)
TAG_RE = re.compile(r"^\d+\.\d+\.\d+$")
SEP_FIELD, SEP_RECORD = "\x1f", "\x1e"

# (file, pattern with three groups around the version). The first match is the version.
TARGETS = [
    ("Cargo.toml", re.compile(r'(?m)^(version\s*=\s*")(\d+\.\d+\.\d+)(")')),
    ("Cargo.lock", re.compile(r'(name = "rstory"\nversion = ")(\d+\.\d+\.\d+)(")')),
    ("extension/metadata.json", re.compile(r'("version-name"\s*:\s*")(\d+\.\d+\.\d+)(")')),
    ("docs/site/src/content/generated/cli-help.json",
     re.compile(r'("rstory"\s*:\s*")(\d+\.\d+\.\d+)(")')),
]


class ReleaseError(Exception):
    pass


def git(*args):
    result = subprocess.run(["git", *args], capture_output=True, text=True)
    if result.returncode != 0:
        raise ReleaseError(f"git {' '.join(args)}: {result.stderr.strip()}")
    return result.stdout


def parse_version(text):
    return tuple(int(part) for part in text.split("."))


def effective_bump(version, bump):
    """The level to apply: below 1.0.0 breaking is a minor and the rest a patch."""
    if parse_version(version)[0] == 0 and bump in ("major", "minor"):
        return "minor" if bump == "major" else "patch"
    return bump


def bump_version(version, bump):
    major, minor, patch = parse_version(version)
    bump = effective_bump(version, bump)
    if bump == "major":
        return f"{major + 1}.0.0"
    if bump == "minor":
        return f"{major}.{minor + 1}.0"
    if bump == "patch":
        return f"{major}.{minor}.{patch + 1}"
    return version


def last_tag():
    tags = [t for t in git("tag", "--list", "--merged", "HEAD").split() if TAG_RE.match(t)]
    return max(tags, key=parse_version) if tags else None


def classify(sha, subject, body):
    match = HEADER_RE.match(subject)
    commit = {"sha": sha, "subject": subject, "type": None, "scope": None, "desc": subject,
              "breaking": None, "bump": "none"}
    if not match:
        return commit
    commit.update(type=match["type"], scope=match["scope"], desc=match["desc"])
    footer = BREAKING_RE.search(body)
    if match["bang"] or footer:
        commit["breaking"] = footer["text"].strip() if footer else match["desc"]
        commit["bump"] = "major"
    else:
        commit["bump"] = TYPE_BUMP.get(match["type"], "none")
    return commit


def commits(since):
    """Releasable-or-not commits in since..HEAD (all history if since is None)."""
    rev = f"{since}..HEAD" if since else "HEAD"
    fmt = SEP_FIELD.join(["%H", "%P", "%s", "%b"]) + SEP_RECORD
    result = []
    for record in git("log", "--no-color", f"--format={fmt}", rev).split(SEP_RECORD):
        record = record.strip("\n")
        if not record:
            continue
        sha, parents, subject, body = (record.split(SEP_FIELD) + [""] * 4)[:4]
        if len(parents.split()) > 1:  # merge commit: its subject doesn't count
            continue
        if subject.startswith("chore(release):"):
            continue
        result.append(classify(sha, subject, body))
    return result


def max_bump(items):
    return max((c["bump"] for c in items), key=RANK.get, default="none")


def repo_root():
    return os.path.realpath(git("rev-parse", "--show-toplevel").strip())


def read_versions(root):
    """The version in each target; they must all agree."""
    found = {}
    for rel, pattern in TARGETS:
        path = os.path.join(root, rel)
        if not os.path.isfile(path):
            raise ReleaseError(f"{rel}: not found")
        match = pattern.search(open(path).read())
        if not match:
            raise ReleaseError(f"{rel}: no version to bump")
        found[rel] = match[2]
    if len(set(found.values())) != 1:
        detail = ", ".join(f"{rel} {v}" for rel, v in found.items())
        raise ReleaseError(f"the versions are out of step: {detail}")
    return next(iter(found.values()))


def generated_section(version, date, items):
    groups = [("Breaking", [c for c in items if c["breaking"]]),
              ("Added", [c for c in items if not c["breaking"] and c["type"] == "feat"]),
              ("Fixed", [c for c in items if not c["breaking"] and c["type"] == "fix"]),
              ("Changed", [c for c in items if not c["breaking"] and c["type"] in ("perf", "refactor")])]
    lines = [f"## {version} — {date}", ""]
    for title, group in groups:
        if not group:
            continue
        lines += [f"### {title}", ""]
        for c in group:
            text = c["breaking"] if title == "Breaking" else c["desc"]
            lines.append(f"- **{c['scope']}:** {text}" if c["scope"] else f"- {text}")
        lines.append("")
    return "\n".join(lines)


def updated_changelog(text, version, date, items):
    heading = f"## {version} — {date}"
    lines = text.split("\n")
    for i, line in enumerate(lines):
        if line.strip() == "## Unreleased":
            lines[i] = heading
            return "\n".join(lines), "renamed ## Unreleased"
    section = generated_section(version, date, items)
    for i, line in enumerate(lines):
        if line.startswith("## "):
            return "\n".join(lines[:i] + section.split("\n") + lines[i:]), "generated from commits"
    body = text.rstrip("\n")
    return (body + "\n\n" if body else "# Changelog\n\n") + section, "generated from commits"


def section_of(text, version=None):
    lines = text.split("\n")
    start = None
    for i, line in enumerate(lines):
        match = re.match(r"^## (\d+\.\d+\.\d+)\b", line)
        if match and (version is None or match[1] == version):
            start = i
            break
    if start is None:
        raise ReleaseError(f"CHANGELOG.md has no section for {version or 'any version'}")
    end = next((j for j in range(start + 1, len(lines)) if lines[j].startswith("## ")), len(lines))
    return "\n".join(lines[start + 1:end]).strip("\n") + "\n"


def main(argv):
    parser = argparse.ArgumentParser(description="Plan and prepare a release from Conventional Commits.")
    parser.add_argument("--dry-run", action="store_true", help="print the plan, write nothing")
    parser.add_argument("--notes", nargs="?", const="", metavar="X.Y.Z",
                        help="print a release's CHANGELOG section (default: the newest)")
    parser.add_argument("--date", help="release date, YYYY-MM-DD (default: today, UTC)")
    parser.add_argument("--version", help="release exactly X.Y.Z (for 1.0.0 or a hand-picked bump)")
    args = parser.parse_args(argv)
    if args.version and not TAG_RE.match(args.version):
        parser.error("--version must be X.Y.Z")
    if args.date and not re.fullmatch(r"\d{4}-\d{2}-\d{2}", args.date):
        parser.error("--date must be YYYY-MM-DD")

    root = repo_root()
    changelog = os.path.join(root, "CHANGELOG.md")

    if args.notes is not None:
        with open(changelog) as f:
            sys.stdout.write(section_of(f.read(), args.notes or None))
        return 0

    tag = last_tag()
    current = read_versions(root)
    if tag is not None and current != tag:
        raise ReleaseError(f"the files say {current} but the last tag is {tag}: "
                           "a release was already prepared, or the versions were edited by hand")
    items = commits(tag)
    bump = max_bump(items)
    if bump == "none" and not args.version:
        print(f"no release needed: no feat, fix, perf, refactor or breaking commits since {tag or 'the start'}",
              file=sys.stderr)
        return NO_RELEASE
    version = args.version or bump_version(current, bump)
    if parse_version(version) <= parse_version(current) and tag is not None:
        raise ReleaseError(f"{version} is not above the current version {current}")
    date = args.date or datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")

    text = open(changelog).read() if os.path.exists(changelog) else ""
    new_changelog, how = updated_changelog(text, version, date, items)

    level = effective_bump(current, bump) if bump != "none" else "chosen"
    plan = [f"release {version} ({level}, since {tag or 'the start'})",
            *(f"{rel}: {current} -> {version}" for rel, _ in TARGETS),
            f"CHANGELOG.md: {how}"]
    if args.dry_run:
        print("\n".join(plan))
        print()
        print(f"## {version} — {date}")
        print()
        print(section_of(new_changelog, version), end="")
        return 0

    for rel, pattern in TARGETS:
        path = os.path.join(root, rel)
        new_text, count = pattern.subn(lambda m: m[1] + version + m[3], open(path).read(), count=1)
        if count != 1:
            raise ReleaseError(f"{rel}: no version to bump")
        with open(path, "w") as f:
            f.write(new_text)
    with open(changelog, "w") as f:
        f.write(new_changelog)
    print("\n".join(plan), file=sys.stderr)
    print(version)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except ReleaseError as e:
        print(f"release.py: error: {e}", file=sys.stderr)
        sys.exit(1)
