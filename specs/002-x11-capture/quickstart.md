# Quickstart: Session detection & X11 capture (incl. XWayland bridge)

How to exercise this feature once built.

```bash
Xvfb :99 & DISPLAY=:99 cargo test --test x11_capture
# bridge mode (tools container): headless mutter, no data-control, XWayland on demand
cargo test --test bridge_capture
# on GNOME (manual): run the daemon and copy from a GUI app
RUST_LOG=rstory=debug cargo run -- daemon   # logs: backend=x11 mode=xwayland_bridge
```
