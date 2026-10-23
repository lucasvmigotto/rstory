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

## KDE Plasma Wayland manual check

Why manual: headless sway (CI) covers the data-control protocol, but only a real Plasma session
has KWin's own data-control and the clients that set `x-kde-passwordManagerHint` (secrets
managers). This covers SC-003 of spec 003. Use only throwaway text you typed for the check, never
real clipboard contents.

Repeat on each supported Plasma 6 release (Wayland session). Setup: start `rstory daemon` from a
terminal with `RUST_LOG=rstory=debug`. The log must say `backend=wayland mode=native`. Keep
another terminal for `rstory ls`.

| # | Step | Expected |
|---|---|---|
| 1 | Copy a line from a GUI app (Kate, a Dolphin rename box, Firefox) | New entry at the top of `rstory ls`, text correct |
| 2 | Copy with `wl-copy "wayland-native copy"` | New entry, text correct |
| 3 | Copy a selection in Konsole with Ctrl+Shift+C | New entry |
| 4 | Copy the same text twice | One entry, moved to the top (BR1) |
| 5 | Copy an entry from a secrets manager (it offers the sensitive hint) | No entry recorded; `skipped_sensitive` rises |
| 6 | Restore an entry, then paste into Kate and into an XWayland app (`xterm`) | Pasted text equals the entry |
| 7 | Restore a value of about 1 MiB and paste into Kate | Complete paste, no hang |
| 8 | Copy something else after a restore | The new copy is captured; rstory stops serving (no stale paste) |
| 9 | Copy from a GUI app after the screen locked and unlocked | New entry |

Record per Plasma version: version, distribution, result per row, and anything odd. A failing
row blocks the release.

Quick CI cross-check before the manual run:
`make test-integration TEST_ARGS="--test wayland_capture"` and `--test wayland_serve`.

## Autostart re-login check (GNOME, KDE Plasma, sway)

Why manual: only a real login runs the XDG autostart entry and the systemd user unit. This covers
SC-001 of spec 012. Install the release binary to `~/.local/bin` first, so the entry does not
point at a temporary path.

Repeat on GNOME (Wayland), KDE Plasma (Wayland) and sway, once for each autostart kind.

| # | Step | Expected |
|---|---|---|
| 1 | `rstory install-autostart`, then log out and back in | `rstory status` answers; the daemon is running |
| 2 | Run `rstory install-autostart` again | Same file, exit 0, nothing else changes |
| 3 | `rstory install-autostart --systemd`, then log out and back in | `systemctl --user status rstory` is active; no `~/.config/autostart/rstory.desktop` is left |
| 4 | `pkill rstory` while the systemd unit is installed | The daemon is back within a few seconds (`Restart=on-failure`) |
| 5 | `rstory install-autostart --uninstall`, then log out and back in | No daemon is running; `rstory status` exits 3 |

Record per desktop: version, distribution, result per row. A failing row blocks the release.
