# CLI layer: Autostart output

| Case | Output (stdout unless noted) | Exit |
|---|---|---|
| install (XDG) | `install.autostart.done` with the path | 0 |
| install --systemd | `Autostart installed: ~/.config/systemd/user/rstory.service (systemd user unit).` | 0 |
| --now | adds `Daemon started.` | 0 |
| temp path warning | stderr: `Warning: rstory is running from {path}; autostart will point there. Install the binary to ~/.local/bin first.` | 0 |
| no systemd user | stderr: `systemd user instance not available. Run rstory install-autostart without --systemd.` | 5 |
| --uninstall | `Autostart removed.` (or `Nothing to remove.`) | 0 |

Tests: snapshots of each line.
