# Quickstart: Privacy filters

How to exercise this feature once built.

```bash
cargo test domain::filter
cargo test --test fitness the_canary_never_appears_in_trace_logs
cargo test --test filter_props --test daemon_privacy
# manual: wl-copy --type x-kde-passwordManagerHint < /dev/null; rstory status   # skipped_sensitive +1
```
