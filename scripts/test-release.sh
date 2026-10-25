#!/usr/bin/env bash
# Tests scripts/release.py in throwaway repositories: which commits release and at which level
# (0.x and 1.x), every version file bumped together, versions out of step, CHANGELOG promotion
# or generation, ignored commits, --version, --dry-run and --notes. Pure bash and python, no
# container. Run by `make sc001-test` and the CI checks job.

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE="$REPO_DIR/scripts/release.py"
SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.test
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.test

failures=0
check() { # check <description> <command...>
  local what="$1"
  shift
  if "$@"; then
    printf '      ok    %s\n' "$what"
  else
    printf '      FAIL  %s\n' "$what"
    failures=$((failures + 1))
  fi
}

# A repository at VERSION with every file release.py bumps, tagged VERSION.
fresh() { # fresh <name> <version>
  local dir="$SANDBOX/$1"
  mkdir -p "$dir/extension" "$dir/docs/site/src/content/generated"
  git -C "$dir" init -q -b main
  printf '[package]\nname = "rstory"\nversion = "%s"\n' "$2" >"$dir/Cargo.toml"
  printf '[[package]]\nname = "rstory"\nversion = "%s"\n' "$2" >"$dir/Cargo.lock"
  printf '{ "version-name": "%s" }\n' "$2" >"$dir/extension/metadata.json"
  printf '{\n  "rstory": "%s"\n}\n' "$2" >"$dir/docs/site/src/content/generated/cli-help.json"
  git -C "$dir" add -A
  git -C "$dir" commit -q -m "chore: start"
  git -C "$dir" tag "$2"
  echo "$dir"
}
commit() { # commit <dir> <subject> [body]
  git -C "$1" commit -q --allow-empty -m "$2" ${3:+-m "$3"}
}
run() { (cd "$1" && shift && python3 "$RELEASE" "$@"); }
code() { run "$@" >/dev/null 2>&1 && echo 0 || echo $?; }

echo "release.py"

d=$(fresh docs-only 0.1.1)
commit "$d" "docs: reword"
commit "$d" "ci: pin an action"
check "docs and ci alone need no release (exit 3)" test "$(code "$d")" = 3

d=$(fresh fix 0.1.1)
commit "$d" "fix(store): sync once"
check "a fix is a patch" test "$(run "$d" --dry-run | head -1)" = "release 0.1.2 (patch, since 0.1.1)"

d=$(fresh feat 0.1.1)
commit "$d" "feat(cli): add a command"
check "a feature is a patch below 1.0.0" test "$(run "$d" --dry-run | head -1)" = "release 0.1.2 (patch, since 0.1.1)"

d=$(fresh breaking 0.1.1)
commit "$d" "feat(ipc)!: new protocol"
check "a breaking change is a minor below 1.0.0" test "$(run "$d" --dry-run | head -1)" = "release 0.2.0 (minor, since 0.1.1)"

d=$(fresh footer 0.1.1)
commit "$d" "fix(ipc): reshape" "BREAKING CHANGE: the socket moved"
check "a BREAKING CHANGE footer counts" test "$(run "$d" --dry-run | head -1)" = "release 0.2.0 (minor, since 0.1.1)"

d=$(fresh one-x 1.4.2)
commit "$d" "feat: something"
check "a feature is a minor from 1.0.0" test "$(run "$d" --dry-run | head -1)" = "release 1.5.0 (minor, since 1.4.2)"
commit "$d" "refactor!: drop a flag"
check "a breaking change is a major from 1.0.0" test "$(run "$d" --dry-run | head -1)" = "release 2.0.0 (major, since 1.4.2)"

d=$(fresh merges 0.1.1)
git -C "$d" checkout -q -b topic
commit "$d" "docs: only docs on the branch"
git -C "$d" checkout -q main
git -C "$d" merge -q --no-ff topic -m "feat: merge subjects do not count"
commit "$d" "chore(release): 0.1.2"
check "merge subjects and chore(release) are ignored" test "$(code "$d")" = 3

d=$(fresh write 0.1.1)
commit "$d" "fix: one"
check "a run prints the new version alone" test "$(run "$d" 2>/dev/null)" = "0.1.2"
check "Cargo.toml bumped" grep -q '^version = "0.1.2"' "$d/Cargo.toml"
check "Cargo.lock bumped" grep -q '^version = "0.1.2"' "$d/Cargo.lock"
check "the extension version-name bumped" grep -q '"version-name": "0.1.2"' "$d/extension/metadata.json"
check "the CLI reference bumped" grep -q '"rstory": "0.1.2"' "$d/docs/site/src/content/generated/cli-help.json"
check "CHANGELOG generated with the fix" grep -q -- '- one' "$d/CHANGELOG.md"
check "the notes are that section" test "$(run "$d" --notes 0.1.2)" = "$(printf '### Fixed\n\n- **:** one' | sed 's/\*\*:\*\* //')"
git -C "$d" add -A
git -C "$d" commit -q -m "chore(release): 0.1.2"
check "a second run before tagging refuses (files ahead of the tag)" test "$(code "$d")" = 1

d=$(fresh step 0.1.1)
commit "$d" "fix: two"
sed -i 's/0.1.1/0.1.0/' "$d/extension/metadata.json"
check "versions out of step are an error" test "$(code "$d")" = 1

d=$(fresh unreleased 0.1.1)
commit "$d" "fix: three"
printf '# Changelog\n\n## Unreleased\n\nHand-written note.\n\n## 0.1.1 — 2026-10-25\n\nold\n' >"$d/CHANGELOG.md"
run "$d" --date 2026-11-01 >/dev/null 2>&1
check "## Unreleased is renamed, the hand-written note kept" grep -q '^## 0.1.2 — 2026-11-01' "$d/CHANGELOG.md"
check "the note is kept" grep -q 'Hand-written note' "$d/CHANGELOG.md"

d=$(fresh forced 0.9.9)
check "--version releases 1.0.0 with no releasable commit" test "$(run "$d" --version 1.0.0 --dry-run | head -1)" = "release 1.0.0 (chosen, since 0.9.9)"
check "--version below the current one is refused" test "$(code "$d" --version 0.9.0)" = 1
check "--version must be X.Y.Z" test "$(code "$d" --version 1.0)" = 2

d=$(fresh dry 0.1.1)
commit "$d" "fix: four"
run "$d" --dry-run >/dev/null
check "--dry-run writes nothing" git -C "$d" diff --quiet

if [ "$failures" -ne 0 ]; then
  echo "release.py: $failures check(s) failed"
  exit 1
fi
echo "release.py: all checks passed"
