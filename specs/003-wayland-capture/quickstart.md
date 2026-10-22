# Quickstart: Wayland data-control capture

How to exercise this feature once built.

```bash
# tools container
WLR_BACKENDS=headless sway -c tests/fixtures/sway.conf &
cargo test --test wayland_capture
# manual (KDE/sway): RUST_LOG=rstory=debug cargo run -- daemon   # backend=wayland mode=native
```
