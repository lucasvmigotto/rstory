# Quickstart: Privacy filters

How to exercise this feature once built.

```bash
cargo test domain::filter
cargo test --test fitness canary_not_logged
# manual: wl-copy --type x-kde-passwordManagerHint < /dev/null; rstory status   # skipped_sensitive +1
```
