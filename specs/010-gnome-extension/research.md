# Research: GNOME Shell extension

Phase 0 for `010-gnome-extension`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Extension format

- **Decision**: ESM (GNOME 45+), `shell-version` 48–51
- **Alternatives rejected**: legacy imports (unsupported since 45)

## R2. Socket from GJS

- **Decision**: `Gio.SocketClient.connect(Gio.UnixSocketAddress)` + `Gio.DataInputStream.read_line_async`
- **Alternatives rejected**: spawning `rstory` CLI per action (slow, process churn)

## R3. Embedding

- **Decision**: `include_dir!("$CARGO_MANIFEST_DIR/extension")`
- **Alternatives rejected**: downloading (network forbidden)
