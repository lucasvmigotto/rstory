#!/usr/bin/env bash
# Shape test of the release automation (spec 013): tag.yml tags a version bump on main and
# dispatches the release; release.yml accepts that dispatch and still guards the tag; every job
# runs the tagged commit. Greps the real workflows (comments stripped); pure bash, no network.
set -euo pipefail
cd "$(dirname "$0")/.."
wf=.github/workflows
bad=0 n=0
strip() { awk '{ sub(/^[[:space:]]*#.*/, ""); sub(/[[:space:]]#.*$/, "") } NF' "$1"; }
ok() { # description, command...
  local d=$1; shift
  n=$((n + 1))
  if "$@" >/dev/null 2>&1; then echo "ok   $d"; else echo "FAIL $d"; bad=$((bad + 1)); fi
}
has() { printf '%s\n' "$2" | grep -Eq -- "$1"; }
lacks() { ! printf '%s\n' "$2" | grep -Eq -- "$1"; }

if [ -f "$wf/tag.yml" ]; then tag=$(strip "$wf/tag.yml"); else tag=''; fi
rel=$(strip "$wf/release.yml")
docs=$(strip "$wf/docs.yml")
ci=$(strip "$wf/ci.yml")
semver='\^\[0-9\]\+\\\.\[0-9\]\+\\\.\[0-9\]\+\$'

ok "tag.yml exists" test -f "$wf/tag.yml"
ok "tag.yml runs on push to main and workflow_dispatch" has '^  push:' "$tag"
ok "tag.yml: branches [main]" has 'branches: \[main\]' "$tag"
ok "tag.yml: workflow_dispatch" has '^  workflow_dispatch:' "$tag"
ok "tag.yml: no pull_request_target" lacks 'pull_request_target' "$tag"
ok "tag.yml: permissions {} at the top" has '^permissions: \{\}$' "$tag"
ok "tag.yml: contents: write exactly once" test "$(printf '%s\n' "$tag" | grep -c 'contents: write')" -eq 1
ok "tag.yml: no id-token" lacks 'id-token' "$tag"
ok "tag.yml: no secrets context" lacks '\bsecrets\b' "$tag"
ok "tag.yml: one run at a time, no cancel" has 'cancel-in-progress: false' "$tag"
ok "tag.yml: every action pinned by SHA" bash -c '! grep -E "^\s*-?\s*uses:" <<<"$1" | grep -Ev "@[0-9a-f]{40}"' _ "$tag"
ok "tag.yml: annotated tag" has 'git .*tag -a|tag -a|-a "?\$TAG' "$tag"
ok "tag.yml: strict version check" has 'A-Za-z0-9|\[0-9\]\+\\.\[0-9\]\+\\.\[0-9\]\+' "$tag"
ok "tag.yml: dispatches the release event" has 'event_type=release' "$tag"
ok "tag.yml: payload has tag and sha" has 'client_payload\[tag\]' "$tag"
ok "tag.yml: payload sha" has 'client_payload\[sha\]' "$tag"
ok "tag.yml: dispatch goes through gh api" has 'gh api .*dispatches' "$tag"
ok "tag.yml: fails when the tag is on another commit" has 'another commit' "$tag"

ok "release.yml: repository_dispatch types [release]" has 'types: \[release\]' "$rel"
ok "release.yml: still tag-push" has 'tags: \["\[0-9\]\+\.\[0-9\]\+\.\[0-9\]\+"\]' "$rel"
ok "release.yml: strict tag regex" has '\^\[0-9\]\+\\\.\[0-9\]\+\\\.\[0-9\]\+\$' "$rel"
ok "release.yml: payload sha validated" has '\[0-9a-f\]\{40\}' "$rel"
ok "release.yml: payload only through env or run-name" bash -c '! grep client_payload <<<"$1" | grep -Ev "^\s+[A-Z_]+: \\$\{\{|format\('"'"'release"' _ "$rel"
ok "release.yml: run-name carries tag and sha" has '^run-name:' "$rel"
ok "release.yml: tag is on main guard" has 'merge-base --is-ancestor' "$rel"
ok "release.yml: tag resolves to the payload sha" has 'does not resolve to' "$rel"
ok "release.yml: tag equals Cargo.toml guard" has 'does not match Cargo.toml' "$rel"
ok "release.yml: manual runs stay dry" has 'manual runs are dry runs only' "$rel"
ok "release.yml: jobs check out the tag" test "$(printf '%s\n' "$rel" | grep -c 'ref: \${{ needs.verify.outputs.ref }}')" -ge 4
ok "release.yml: publish stays here, in the release environment" has 'environment: release' "$rel"
ok "release.yml: crates-io-auth-action stays here" has 'rust-lang/crates-io-auth-action@[0-9a-f]{40}' "$rel"
ok "release.yml: ci called with the tag ref" has 'ref: \${{ needs.verify.outputs.ref }}' "$(printf '%s\n' "$rel" | sed -n '/^  ci:/,/^  build:/p')"
ok "release.yml: failure reporter runs for the dispatch too" has 'repository_dispatch' "$(printf '%s\n' "$rel" | sed -n '/^  report-failure:/,$p')"

ok "ci.yml: workflow_call takes a ref" has '^      ref:' "$ci"
ok "ci.yml: checkouts use the ref input" test "$(printf '%s\n' "$ci" | grep -c 'ref: \${{ inputs.ref }}')" -eq "$(printf '%s\n' "$ci" | grep -c 'actions/checkout@')"

ok "docs.yml: deploy accepts repository_dispatch releases" has "workflow_run.event == 'repository_dispatch'" "$docs"
ok "docs.yml: deploy still accepts tag pushes" has "workflow_run.event == 'push'" "$docs"
ok "docs.yml: tag read from the run's display_title" has 'workflow_run.display_title' "$docs"
ok "docs.yml: no head_branch as the tag" lacks 'head_branch' "$docs"

echo "$((n - bad))/$n release workflow checks passed"
[ "$bad" -eq 0 ]
