# Research: Autostart

Phase 0 for `012-autostart`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Autostart standard

- **Decision**: XDG Autostart spec (works on GNOME, KDE, most WMs with `dex`/session managers)
- **Alternatives rejected**: only systemd (excludes non-systemd distros: G3)

## R2. Unit target

- **Decision**: `graphical-session.target` (PartOf/After/WantedBy)
- **Alternatives rejected**: `default.target` (starts before a display exists)
