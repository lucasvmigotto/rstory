# Quickstart: Picker launch

How to exercise this feature once built.

```bash
TERMINAL=foot cargo run -- doctor | grep -i picker
printf '{"v":1,"id":1,"op":"open_picker"}\n' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/rstory/control.sock
make test-launch   # runs cargo test launch:: in the tools container
```

The `doctor` and `open_picker` commands wait for specs 011 and 004.
