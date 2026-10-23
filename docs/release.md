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

## Tray manual check (KDE Plasma and sway with waybar)

Why manual: the D-Bus tests use a stub `StatusNotifierWatcher`; only a real Plasma panel or
waybar's tray module shows the icon, draws the menu and renders the pixmaps. This covers SC-001
of spec 009. Use only throwaway text you typed for the check, never real clipboard contents.

Run once on KDE Plasma 6 (Wayland) and once on sway with waybar (`tray` module enabled). Setup:
start `rstory daemon` from a terminal with `RUST_LOG=rstory=info`. The log must show
`tray.state` with `state="active"`, and `rstory status --json` must say `"tray": "active"`.
Copy three short throwaway lines first.

| # | Step | Expected |
|---|---|---|
| 1 | Look at the panel | The rstory icon (clipboard outline) is visible and sharp at the panel size |
| 2 | Open the menu | The three lines, newest first, then a separator, `Recording`, `Pause recording`, `Pause for 1 minute`, `Open picker` (when a terminal exists), a separator, `Clear history…` |
| 3 | Click the third line, then paste into a text field | The pasted text equals that line, and the menu now lists it first |
| 4 | Click `Pause recording` | The icon changes to the paused icon (two bars, a different shape, not only another color); the menu shows `Paused` and `Resume recording`; copying something new adds no entry |
| 5 | Click `Resume recording` | The normal icon returns; the next copy is recorded |
| 6 | Click `Pause for 1 minute`, reopen the menu after about 20 s | `Paused — resumes in 0:4x`; after a minute the icon is back to normal by itself |
| 7 | Click `Clear history…` | The menu offers `Delete N unpinned entries` and `Cancel`; nothing is deleted yet; `Cancel` leaves the history as it was |
| 8 | Click `Clear history…` then `Delete N unpinned entries` | Only the unpinned entries go (`rstory ls`); pinned ones stay |
| 9 | Stop the panel's tray host (restart `plasmashell`, or `pkill waybar`) and start it again | The daemon keeps running; the icon reappears within a second or two; `status.tray` goes `no_host`, then `active` |
| 10 | Start the daemon with `tray = false` in `config.toml` | No icon; `status.tray` is `disabled`. `rstory tray` in another terminal shows the icon instead |

Light panels: the embedded icons are the variant for dark panels (light glyph). On a light panel
check that the icon is still legible and record it as a finding (see the spec's Doubts).

Record per desktop: version, result per row, and anything odd. A failing row blocks the release.

CI cross-check before the manual run: `make test TEST_ARGS=` runs `tests/tray_dbus.rs` against a
private `dbus-daemon` and a stub watcher inside the tools container.

## GNOME Shell extension manual matrix (48, 49, 50, 51)

Why manual: the panel button, the Shell's extension loader and its theme only exist in a real
GNOME Shell. The gjs fixture tests (`make ext-test`) cover the client, the menu model and the
cleanup in `disable()` against fakes. This covers SC-001 of spec 010. Use only throwaway text you
typed for the check, never real clipboard contents.

Repeat on each GNOME Shell version: 48, 49, 50 and 51, on a Wayland session. Run it on the light
and the dark panel (Settings > Appearance, or `gsettings set org.gnome.desktop.interface
color-scheme prefer-dark`). Install the release binary to `~/.local/bin` first.

Setup: `rstory install-autostart --now` so a daemon is running. Keep a second terminal for
`rstory ls` and `rstory status`. For the log: `journalctl --user -f /usr/bin/gnome-shell | grep -i
rstory`. Nothing in that log may contain entry text; only error codes.

| # | Step | Expected |
|---|---|---|
| 1 | `rstory install-gnome-extension` | Exit 0; prints `GNOME extension installed. Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.github.io`; no stderr warning |
| 2 | Log out and back in, then `gnome-extensions enable rstory@lucasvmigotto.github.io` (or `rstory install-gnome-extension --enable` on the next login) | `gnome-extensions info rstory@lucasvmigotto.github.io` shows `Enabled: Yes` and a State other than ERROR |
| 3 | Look at the top bar | An rstory clipboard icon (recording shape: three lines). It follows the panel color on light and dark |
| 4 | Copy three different lines of text, then open the menu | The newest entries appear first, one per line, at most 10; the status line says `Recording`; Pause recording, Pause for 1 minute and Open picker are present; `Clear history…` is last |
| 5 | Copy a two-line text | Its menu line shows `⏎` where the newline is |
| 6 | Click an entry, then paste into a text editor | The pasted text equals the entry |
| 7 | `rstory pin 0`, then open the menu | The pinned entry shows the `📌 ` prefix |
| 8 | Click `Pause recording` | The icon changes to the paused shape (two bars); the menu shows `Paused` and `Resume recording`; `Pause for 1 minute` is hidden; `rstory status` says paused |
| 9 | Click `Resume recording` | The icon returns to the recording shape |
| 10 | Click `Pause for 1 minute`, keep the menu open | The status line counts down (`Paused — resumes in 0:59`, ...) and the icon returns to recording by itself at zero |
| 11 | `rstory pause` in a terminal while the menu is closed, then open it | The icon is already the paused shape (the change came from the daemon, not the click) |
| 12 | Click `Open picker` | A terminal with the picker opens; the extension spawned nothing itself (the process parent is the daemon) |
| 13 | Click `Clear history…` then `Cancel` | The menu closes; `rstory ls` is unchanged |
| 14 | Click `Clear history…` then `Delete N unpinned entries` | N matches the unpinned count in `rstory status`; pinned entries remain |
| 15 | `pkill rstory` with the menu open | The icon becomes the disconnected shape (clipboard with a slash); the menu shows only `rstory daemon not running` |
| 16 | Start the daemon again (`rstory daemon &`) | Within 30 s (1 s, 2 s, 4 s, ... backoff) the icon is back to recording and the entries are listed |
| 17 | `rstory install-gnome-extension` again | Same version: the plain `installed` message. After a version bump: `GNOME extension updated to X. Log out and back in to load it.` |
| 18 | `rstory install-gnome-extension --uninstall`, then log out and back in | The icon is gone; `~/.local/share/gnome-shell/extensions/rstory@lucasvmigotto.github.io` does not exist |

Record per Shell version: version, distribution, panel theme, result per row, and anything odd
(for example the icon not following the panel color, or a Shell error in the journal on enable).
A failing row blocks the release. A Shell outside 48 to 51 must refuse to load the extension
because of `metadata.json`; `rstory doctor` explains it.

Quick CI cross-check before the manual run: `make ext-test ext-typecheck ext-lint` and
`make test` (the install tests in `tests/install_gnome.rs`).
