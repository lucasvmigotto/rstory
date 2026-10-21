---
status: proposed
date: 2026-10-21
---
# 0005 — Backend selection, with the XWayland bridge on Wayland without data-control

## Context and problem statement
GNOME (Mutter) offers no data-control protocol. Spike S1 (GNOME Shell
48.7, 2026-10-21) showed that an unfocused X11 client using XFIXES sees
every Wayland-native copy through XWayland, offered MIME types included
(so the sensitive hint survives), and that a value served by an X11
owner can be pasted by Wayland clients.

## Considered options
1. Rule: Wayland with data-control → `wayland`; Wayland without it but
   with `DISPLAY` → `x11` through the bridge; X11 → `x11`; an override
   wins.
2. Use the clipboard portal on GNOME (needs a RemoteDesktop session
   prompt).
3. Capture fed by the GNOME extension (`Meta.Selection` `owner-changed`).

## Decision outcome
Chosen: **option 1**. Options 2 and 3 stay possible behind the `Backend`
trait (brief feature 023, trigger E1). The daemon reconnects to the X
server with backoff (100 ms → 5 s) when XWayland restarts on demand.
`doctor` reports "x11 via XWayland bridge".

### Consequences
- Good: no prompt, no extra install, and capture works before the
  extension is installed. One backend serves X11 and GNOME.
- Good: it generalises to any Wayland compositor that has XWayland but no
  data-control.
- Bad: it keeps an on-demand XWayland alive (brief R14). It depends on
  XWayland continuing to exist (R13).
- Bad: still to verify on Shell 49–51 and with copies from GUI apps
  (brief Q7).

### Confirmation
An integration test under headless sway with data-control disabled plus
XWayland. A manual check on GNOME 48–51 before v0.1.
