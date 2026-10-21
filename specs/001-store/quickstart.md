# Quickstart: Store & history

How to exercise this feature once built.

```bash
cargo test store::            # unit + proptest invariants
cargo test --test fitness perms
cargo bench --bench store     # open/list/insert budgets
# manual: inspect a store
jq . ~/.local/state/rstory/index.json
ls -l ~/.local/state/rstory/content
```
