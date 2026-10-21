# Quickstart: Daemon & control socket

How to exercise this feature once built.

```bash
cargo run -- daemon --backend x11 &           # under Xvfb or GNOME
printf '{"v":1,"id":1,"op":"status"}\n' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/rstory/control.sock
cargo test --test contract
cargo test --test daemon_integration
```
