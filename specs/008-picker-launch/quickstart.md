# Quickstart: Picker launch

How to exercise this feature once built.

```bash
TERMINAL=foot cargo run -- doctor | grep -i picker
printf '{"v":1,"id":1,"op":"open_picker"}\n' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/rstory/control.sock
cargo test launch::
```
