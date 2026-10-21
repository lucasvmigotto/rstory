# CLI layer: install-gnome-extension output

| Case | Output | Exit |
|---|---|---|
| install | `install.gnome.done` | 0 |
| reinstall (older present) | `GNOME extension updated to {version}. Log out and back in to load it.` | 0 |
| --enable ok | adds `Enabled.` | 0 |
| --enable without gnome-extensions | stderr: `gnome-extensions not found; enable it in the Extensions app.` | 0 |
| --uninstall | `GNOME extension removed.` | 0 |
| not GNOME (warning) | stderr: `This session is not GNOME; installed anyway.` | 0 |
