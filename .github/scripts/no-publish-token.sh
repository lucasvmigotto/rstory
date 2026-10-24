#!/usr/bin/env bash
# SC-001 (spec 013): no workflow or local action references a stored publishing token.
# Publishing authenticates only through crates.io Trusted Publishing (OIDC), so:
#   1. nothing under .github/ reads the secrets context at all;
#   2. CARGO_REGISTRY_TOKEN is only ever set from the OIDC exchange (steps.auth.outputs.token);
#   3. the release workflow does use rust-lang/crates-io-auth-action.
# The patterns are written so that this file never matches itself.
set -euo pipefail
dir=${1:-.github}
fail=0

if grep -rnE 'secrets\.[A-Za-z_]|secrets\[' "$dir"; then
  echo "::error::a workflow or action references the secrets context (SC-001: no stored token)"
  fail=1
fi

if bad=$(grep -rn 'CARGO_REGISTRY_TOKE[N]' "$dir" | grep -v 'steps\.auth\.outputs\.token'); then
  echo "$bad"
  echo "::error::the cargo registry token is set from something other than the OIDC exchange (SC-001)"
  fail=1
fi

if ! grep -q 'uses: rust-lang/crates-io-auth-action@[0-9a-f]\{40\}' "$dir/workflows/release.yml"; then
  echo "::error::release.yml does not publish through rust-lang/crates-io-auth-action pinned by SHA"
  fail=1
fi

[ "$fail" -eq 0 ] && echo "SC-001: no publishing token referenced under $dir"
exit "$fail"
