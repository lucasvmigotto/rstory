# UI layer: GNOME panel icon & menu

Layer added by `frontend:spec` (2026-10-21). Same menu as `specs/009-tray/contracts/menu.md` and the same state matrix as `specs/009-tray/ui.md`, plus a **Disconnected** column:

| Region | Disconnected (daemon not running) |
|---|---|
| Icon | disconnected (clipboard + slash) |
| Menu | a single insensitive item `menu.disconnected`; nothing else |

Icons: the `-symbolic.svg` variants from `assets/icons/` (they follow the panel foreground color). Implementation: `PanelMenu.Button` with `St.Icon` (`gicon` from the extension's `icons/` dir), `PopupMenu.PopupMenuItem` per entry, `PopupMenu.PopupSeparatorMenuItem`, and `PopupMenu.PopupSubMenuMenuItem` for the clear confirm.

Accessibility: `accessible_name` set on the panel button (`rstory — {state}`), and text labels on every item.

Tests: the gjs fixture test asserts the menu model for each state, including Disconnected. The manual matrix covers Shell 48–51 (light and dark panels).
