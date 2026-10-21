# TUI/CLI layer: Doctor output

```
rstory doctor
✓ Session     Wayland (GNOME Shell 48.7)
✓ Backend     x11 via XWayland bridge (no data-control on this compositor)
✓ Store       private (0700/0600), 8 entries
✓ Daemon      running, recording
! Extension   installed 0.1.0, not enabled
              → gnome-extensions enable rstory@lucasvmigotto.github.io
✓ Picker      ghostty
✓ Autostart   ~/.config/autostart/rstory.desktop

1 warning.
```

The last line is `doctor.all_ok` when everything passes, else `{n} warning(s), {m} failure(s).` ASCII: `ok`/`warn`/`FAIL`, `->`. Colors: ok uses the default color, warn the warning role, fail the danger role, and the words carry the meaning. `--json` per `/contracts/cli.md`.

Tests: snapshots of the healthy run and each failure row (8), in mono and ASCII.
