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

## Build notes (added by the build, 2026-10-23)

- **zbus version**: ksni 0.3.7 depends on zbus 5, so the crate uses zbus 5 (plan.md said 6). One zbus in the tree.
- **Icons (T001, T017)**: the design system's `assets/icons/{recording,paused,disconnected}-symbolic.svg` and `png/<size>/dark/*.png` are the sources; no separate `normal.svg`. `build.rs` decodes the `dark` variant (light glyph, for dark panels) into ARGB32. `disconnected` is used for the inactive state. A light-panel variant is open (see the report's Doubts).
- **Tray state**: `status.tray` starts as `disabled` and settles to `active`, `no_host` or `no_bus` once the first registration attempt ends (ksni `assume_sni_available`, watcher callbacks).
- **Clear confirm**: `Clear history…` is an item until activated; it then becomes a submenu with `Delete N unpinned entries` and `Cancel`. Any history or pause event, and any other action, returns it to the plain item.
- **Remaining pause time** is computed when the menu model is built (on events and when the menu is about to show); it does not tick while the menu stays open.
- **`rstory tray`**: the same tray over a control-socket bridge (`src/tray/client.rs`), for a daemon started with `tray = false`.
- **Tests**: `tests/tray_dbus.rs` re-executes its own test binary as the daemon-with-tray, with `DBUS_SESSION_BUS_ADDRESS` set to a private `dbus-daemon`, because ksni reads that variable and `set_var` is unsafe. The parent talks to the private bus by explicit address only.
