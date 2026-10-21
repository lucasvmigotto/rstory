# Quickstart: Release & publishing

How to exercise this feature once built.

```bash
actionlint .github/workflows/*.yml
gh workflow run release.yml -f dry_run=true
cargo publish --dry-run
```
