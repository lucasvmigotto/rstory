#!/usr/bin/env bash
# Fixture-driven test of .github/scripts/no-publish-token.sh (SC-001): the real .github passes,
# a clean fixture passes, and each forbidden pattern is caught on its own. Pure bash, no network.
set -euo pipefail
cd "$(dirname "$0")/.."
check=.github/scripts/no-publish-token.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
sha=0123456789abcdef0123456789abcdef01234567
n=0 bad=0

# fixture NAME EXTRA-LINES: a release.yml with the auth action, plus EXTRA-LINES as a job env.
fixture() {
  local d="$tmp/$1"
  mkdir -p "$d/workflows"
  cat > "$d/workflows/release.yml" <<Y
name: release
jobs:
  publish:
    runs-on: ubuntu-24.04
    steps:
      - id: auth
        uses: rust-lang/crates-io-auth-action@$sha
      - run: make publish
        env:
          CARGO_REGISTRY_TOKEN: \${{ steps.auth.outputs.token }}
$2
Y
}
expect() { # name want(0|1) 
  n=$((n + 1))
  local rc=0
  "$check" "$tmp/$1" >/dev/null 2>&1 || rc=$?
  if [ "$rc" -ne "$2" ]; then echo "FAIL $1: exit $rc, wanted $2"; bad=$((bad + 1)); else echo "ok   $1"; fi
}

n=$((n + 1))
if "$check" .github >/dev/null; then echo "ok   real .github"; else echo "FAIL real .github"; bad=$((bad + 1)); fi

fixture clean ''
fixture clean-comments '          # secrets.FOO and toJSON(secrets) and vars.API_TOKEN in a comment only'
fixture clean-trailing-comment '          FOO: bar # secrets: inherit'
expect clean 0; expect clean-comments 0; expect clean-trailing-comment 0

fixture secrets-dot '          X: ${{ secrets.CARGO_TOKEN }}'
fixture secrets-bracket "          X: \${{ secrets['CARGO_TOKEN'] }}"
fixture secrets-tojson '          ALL: ${{ toJSON(secrets) }}'
fixture secrets-inherit '    secrets: inherit'
fixture secrets-hidden-by-comment '          X: ${{ secrets.T }} # steps.auth.outputs.token'
fixture vars-token '          X: ${{ vars.CRATES_TOKEN }}'
fixture vars-registry '          CARGO_REGISTRY_TOKEN: ${{ vars.PUBLISH }}'
fixture registries-var '          CARGO_REGISTRIES_CRATES_IO_TOKEN: ${{ steps.other.outputs.t }}'
fixture registries-secret '          CARGO_REGISTRIES_CRATES_IO_TOKEN: ${{ secrets.T }}'
fixture registry-literal '          CARGO_REGISTRY_TOKEN: abc'
for f in secrets-dot secrets-bracket secrets-tojson secrets-inherit secrets-hidden-by-comment vars-token \
  vars-registry registries-var registries-secret registry-literal; do expect "$f" 1; done

# The docs deploy exemption: only secrets.CLOUDFLARE_R2_<NAME>, only in workflows/docs.yml.
docs() { # NAME ENV-LINE: a clean fixture plus a docs.yml whose deploy job has ENV-LINE
  fixture "$1" ''
  printf 'name: docs\njobs:\n  deploy:\n    runs-on: ubuntu-24.04\n    env:\n%s\n    steps:\n      - run: true\n' "$2" > "$tmp/$1/workflows/docs.yml"
}
docs docs-r2-allowed '      AWS_SECRET_ACCESS_KEY: ${{ secrets.CLOUDFLARE_R2_ACCOUNT_SECRET }}'
docs docs-other-secret '      X: ${{ secrets.CARGO_TOKEN }}'
docs docs-r2-plus-other '      X: ${{ secrets.CLOUDFLARE_R2_ACCOUNT_SECRET }}${{ secrets.T }}'
fixture r2-outside-docs '          X: ${{ secrets.CLOUDFLARE_R2_ACCOUNT_SECRET }}'
expect docs-r2-allowed 0
for f in docs-other-secret docs-r2-plus-other r2-outside-docs; do expect "$f" 1; done

fixture no-auth ''
sed -i '/crates-io-auth-action/d' "$tmp/no-auth/workflows/release.yml"
expect no-auth 1

echo "$((n - bad))/$n cases passed"
[ "$bad" -eq 0 ]
