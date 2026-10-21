# Quickstart: Terminal picker

How to exercise this feature once built.

```bash
cargo run -- pick           # needs a TTY
cargo run -- pick --plain
cargo test picker:: ; cargo insta review
cargo bench --bench picker_first_frame
```
