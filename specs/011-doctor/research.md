# Research: Doctor

Phase 0 for `011-doctor`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Shell version

- **Decision**: `gnome-shell --version` when on PATH, else from `org.gnome.Shell` D-Bus `ShellVersion` property
- **Alternatives rejected**: parse distro files

## R2. Extension enabled

- **Decision**: `gnome-extensions info <uuid>` if present; else read `org.gnome.shell enabled-extensions` via `gsettings` when available; else `unknown`
- **Alternatives rejected**: linking GSettings (C)
