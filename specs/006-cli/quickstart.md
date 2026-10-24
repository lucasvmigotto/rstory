# Quickstart: Command-line interface

How to exercise this feature once built.

```bash
cargo run -- ls ; cargo run -- ls --json | jq .
cargo run -- show 0 ; cargo run -- copy 1
cargo run -- pause --for 50s && cargo run -- status
cargo test --test cli_browse   # in the tools container (make shell); also cli_manage, cli_readonly, cli_status, cli_exit_codes, cli_completions, contract
cargo bench --bench ls   # the ls budget, 500 entries, socket and read-only
```
