#!/usr/bin/env bash
# SC-001 (spec 013): no workflow or local action references a stored publishing token.
# Publishing authenticates only through crates.io Trusted Publishing (OIDC), so, in every
# *.yml and *.yaml under the directory (comments stripped first, so a comment cannot hide a
# reference and cannot trigger one):
#   1. no expression reads the secrets context in any form (secrets.X, secrets['X'],
#      toJSON(secrets), fromJSON(secrets...)), and no job passes `secrets: inherit`;
#   2. no expression reads the vars context under a credential-like name, and none feeds a
#      cargo registry variable;
#   3. CARGO_REGISTRY_TOKEN and CARGO_REGISTRIES_<name>_TOKEN are only ever set from the OIDC
#      exchange (steps.auth.outputs.token);
#   4. the release workflow does use rust-lang/crates-io-auth-action, pinned by SHA.
# One narrow exemption: the docs deploy (workflows/docs.yml) may read secrets.CLOUDFLARE_R2_<NAME>,
# its R2 credential. SC-001 is about a stored *publishing* token; the docs site isn't published to a
# registry. Those exact references are blanked in docs.yml before the checks; anything else is not.
# Tested by scripts/test-no-publish-token.sh (make sc001-test).
set -euo pipefail
dir=${1:-.github}
fail=0

# "file:line:content" for every non-comment line of every workflow or action file.
mapfile -t files < <(find "$dir" -type f \( -name '*.yml' -o -name '*.yaml' \) | sort)
lines=$(for f in "${files[@]}"; do
  awk -v f="$f" '{ sub(/^[[:space:]]*#.*/, ""); sub(/[[:space:]]#.*$/, "") } NF { print f ":" NR ":" $0 }' "$f"
done | sed -E '/^[^:]*\/workflows\/docs\.yml:/ s/\$\{\{[[:space:]]*secrets\.CLOUDFLARE_R2_[A-Z0-9_]+[[:space:]]*\}\}/R2_SECRET/g')

report() { # pattern, message
  if hits=$(printf '%s\n' "$lines" | grep -E -- "$1"); then
    echo "$hits"
    echo "::error::$2"
    fail=1
  fi
}

report '\$\{\{[^}]*\bsecrets\b|^[^:]*:[0-9]+:[[:space:]]*secrets:[[:space:]]*inherit\b' \
  'a workflow or action reads the secrets context or passes secrets: inherit (SC-001: no stored token)'
report '\$\{\{[^}]*\bvars\b[^}]*(TOKEN|KEY|SECRET|PASSWORD|CREDENTIAL)' \
  'a workflow uses a repository variable as a credential (SC-001: no stored token)'
report 'CARGO_REGISTR(Y|IES)_[A-Za-z0-9_]*TOKEN[[:space:]]*:[^#]*\$\{\{[^}]*\bvars\b' \
  'a cargo registry token is read from a repository variable (SC-001: no stored token)'

if bad=$(printf '%s\n' "$lines" | grep -E 'CARGO_REGISTR(Y|IES)_[A-Za-z0-9_]*TOKEN' | grep -v 'steps\.auth\.outputs\.token'); then
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
