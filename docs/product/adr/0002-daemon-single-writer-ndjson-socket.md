---
status: proposed
date: 2026-10-21
---
# 0002 — Daemon as the single writer; NDJSON protocol v1 over a Unix socket for every client

## Context and problem statement
The CLI, picker, tray and GNOME extension all have to read and change the
history and the pause state. Both clipboard protocols need a live owner to
serve a restore. The owner decided that UIs hold no logic and that the
extension uses the control socket (brief, 2026-10-21).

## Decision drivers
- Store reliability: a single writer avoids lock contention and
  corruption (driver 3)
- Privacy: only the user may connect (driver 2)
- GJS must be able to speak the protocol without native modules
- Scripts and debugging should be able to use the protocol by hand
  (`socat`)

## Considered options
1. NDJSON over a Unix stream socket, versioned (`"v":1`), with
   request/response plus a `subscribe` event stream.
2. A D-Bus session service (`io.github.lucasvmigotto.rstory`).
3. A binary protocol (postcard or protobuf) over a Unix socket.
4. Shared store access with file locks and no daemon API.

## Decision outcome
Chosen: **option 1**. The socket lives at
`$XDG_RUNTIME_DIR/rstory/control.sock` in a `0700` directory. Every
connection is checked with `SO_PEERCRED` (uid must match). One JSON object
per line, at most 2 MiB per line (1 MiB of content plus escaping).
Unknown fields are ignored. A `v` the daemon doesn't know gets a
`unsupported_version` error. Clients may read the store read-only when the
daemon is down (brief BR14).

### Consequences
- Good: GJS (`Gio.SocketClient` + `Gio.DataInputStream.read_line`), shell
  tools and Rust all speak it trivially.
- Good: one auth model and one schema to version.
- Bad: JSON costs a little CPU, which doesn't matter at fewer than
  100 requests a day.
- Bad: no D-Bus activation or introspection. Trigger E5 adds them if
  asked for.

### Confirmation
Contract tests (spec 005) validate every message against the JSON Schema.
The extension and the CLI share the same fixtures.
