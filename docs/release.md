# Release runbook (draft)

> Draft. The first-release runbook and the Tier-1 manual matrix are completed by spec 013. This
> section is the part owned by spec 002 (X11 capture and the XWayland bridge).

## GNOME 48–51 manual check

Why manual: headless mutter (used in CI for bridge mode) has no keyboard seat, so no Wayland
client can own a selection there. Cross-protocol copies, Wayland app to XWayland and back, can
only be checked on a real GNOME session. This covers SC-004, brief Q7 and the confirmation of
ADR-0005. Use only throwaway text you typed for the check, never real clipboard contents.

Repeat on each GNOME Shell version: 48, 49, 50, 51 (Wayland session, no extension installed).

Setup: start `rstory daemon` from a terminal with `RUST_LOG=rstory=debug`. The log must say
`backend=x11 mode=xwayland_bridge` (`rstory doctor` should report the same). Keep another
terminal for `rstory ls`.

| # | Step | Expected |
|---|---|---|
| 1 | Copy a line from a GUI app (Text Editor, a Files rename box, Firefox) | New entry at the top of `rstory ls`, text correct |
| 2 | Copy with `wl-copy "wayland-native copy"` | New entry, text correct (a Wayland client seen through XWayland) |
| 3 | Copy a selection in Terminal with Ctrl+Shift+C | New entry |
| 4 | Copy the same text twice | One entry, moved to the top (BR1) |
| 5 | Copy from a secrets-manager entry (it offers the sensitive hint) | No entry recorded; `skipped_sensitive` rises |
| 6 | Restore an entry (`rstory restore <id>` or the picker), then paste into a Wayland-native app (Text Editor, Console) | Pasted text equals the entry |
| 7 | Paste the same restored value into an X11-only app (`xterm`, or `xclip -o -selection clipboard`) | Pasted text equals the entry |
| 8 | Restore a value of about 1 MiB and paste into a Wayland-native app | Complete paste, no hang |
| 9 | Kill XWayland (`pkill Xwayland`), then copy again | The daemon reconnects (100 ms to 5 s backoff) and the copy is captured |
| 10 | Copy from a GUI app after the screen locked and unlocked | New entry |

Record per Shell version: version, distribution, result per row, and anything odd (for example a
GUI app that does not update the selection while unfocused). A failing row blocks the release.

Quick CI cross-check before the manual run:
`make test-integration TEST_ARGS="--test bridge_capture"`.
