# Quickstart: Command-line interface

How to exercise this feature once built.

```bash
cargo run -- ls ; cargo run -- ls --json | jq .
cargo run -- show 0 ; cargo run -- copy 1
cargo run -- pause --for 50s && cargo run -- status
cargo test --test cli
```
