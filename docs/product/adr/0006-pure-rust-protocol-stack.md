---
status: proposed
date: 2026-10-21
---
# 0006 — A pure-Rust protocol stack: wayland-client (Rust backend), x11rb, zbus + ksni

## Context and problem statement
G1 forbids `libwayland`, `libxcb`, `libdbus` and every other runtime
library. The binary must link statically against musl.

## Considered options
- Wayland: **wayland-client 0.31 with its default pure-Rust backend** plus
  wayland-protocols 0.32 (`staging`: `ext-data-control-v1`) and
  wayland-protocols-wlr 0.3, versus smithay-client-toolkit, versus
  hand-written wire code.
- X11: **x11rb 0.14 (`RustConnection`, `xfixes` feature)** versus the xcb
  crate (links libxcb).
- D-Bus / tray: **ksni 0.3 on zbus (tokio)** versus our own SNI on zbus
  versus libappindicator (C).

## Decision outcome
Chosen: the options in bold. Pin minor versions. Keep each library
behind its module (`backend::wayland`, `backend::x11`, `tray`).

### Consequences
- Good: a static musl build with no C toolchain at runtime. F3 enforces
  it.
- Bad: smaller communities than the C libraries. Protocol gaps (INCR,
  data-control versions) may need in-tree code (architecture §11).

### Confirmation
F2 and F3 are green. Capture and restore integration tests pass on sway
and Xvfb.
