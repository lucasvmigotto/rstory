# Research: SNI tray

Phase 0 for `009-tray`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Library

- **Decision**: ksni 0.3 on zbus with tokio (ADR-0006)
- **Alternatives rejected**: hand-rolled SNI+DBusMenu (≈1k lines), libappindicator (C)

## R2. Icons

- **Decision**: ARGB32 pixmaps generated at build time from `assets/icons/*.png`
- **Alternatives rejected**: icon theme names (theme-dependent, may not exist)

## R3. Confirm in a menu

- **Decision**: swap the item for a submenu with Confirm/Cancel
- **Alternatives rejected**: dialogs (no GUI toolkit by design)
