---
status: proposed
date: 2026-10-21
---
# 0004 — tokio `current_thread` as the daemon's event loop

## Context and problem statement
The daemon multiplexes the display connection (Wayland or X11 fd), socket
clients, the D-Bus tray, and timers (expiry, timed pause). It must not
poll (G6), and it must stay small (RSS ≤ 20 MiB).

## Considered options
1. tokio with `current_thread` (`rt`, `net` for Unix sockets, `time`,
   `fs`, `signal`, `io-util`, `macros`).
2. smol / async-io (zbus's default).
3. A hand-rolled `poll` loop (calloop or rustix).

## Decision outcome
Chosen: **option 1**. ksni and zbus both support tokio. Blocking file
writes go through `spawn_blocking`. The Wayland and X11 connections are
registered through `AsyncFd`.

### Consequences
- Good: a well-known runtime with mature `AsyncFd`, Unix sockets, timers
  and signals. One thread keeps RSS low.
- Bad: tokio is bigger than smol, by about 0.5–1 MiB of binary. That's
  acceptable under F5.
- Bad: zbus must be configured with `default-features = false,
  features = ["tokio"]`, to avoid pulling in two runtimes.

### Confirmation
`cargo tree -e features` shows a single runtime. F10 (idle wake-ups)
passes.
