---
status: proposed
date: 2026-10-21
---
# 0008 — GNOME extension: embedded ESM GJS, a control-socket client with no logic

## Context and problem statement
GNOME has no tray, and the owner rejected requiring AppIndicator. The
extension has to be installed by the binary itself (G1), support Shell
48–51, and survive Shell churn (brief R5).

## Considered options
1. A minimal ESM extension (`extension.js`, `metadata.json`,
   `stylesheet.css`, icons) embedded with `include_dir`, talking to the
   daemon over the control socket (ADR-0002).
2. The same extension talking D-Bus.
3. Depend on the AppIndicator extension plus the SNI tray.

## Decision outcome
Chosen: **option 1**. The extension renders `PanelMenu.Button` with the
same items as the SNI tray: last 10 entries, pause/resume, pause for
1 minute, open picker, clear with a confirm dialog. It sends
`restore`/`pause`/`resume`/`clear` requests and listens to `subscribe`
events. "Open picker" asks the daemon to run `picker_command`, so the
extension never spawns anything itself. It reconnects with backoff when
the daemon restarts. `install-gnome-extension` writes the files, and
`--enable` runs `gnome-extensions enable` when it's available.

### Consequences
- Good: no logic to break between Shell releases besides the panel
  API. The same protocol fixtures as the CLI.
- Bad: needs a re-login on Wayland to load (unavoidable). Installed
  files must be refreshed after upgrades, which `doctor` detects by
  comparing the version in `metadata.json`.

### Confirmation
A manual matrix on Shell 48–51 before tags. An ESLint/gjs syntax check in
CI. Protocol fixture tests on the JS side, run by `gjs` when it's
available in CI `[ASSUMPTION]`.
