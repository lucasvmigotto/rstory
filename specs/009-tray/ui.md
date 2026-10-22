# UI layer: SNI tray menu

Layer added by `frontend:spec` (2026-10-21). Menu contract: `contracts/menu.md`. Copy keys: ux-vision (`menu.*`, `state.*`). Icons: `specs/000-design-system` (recording/paused, light/dark pixmaps).

## Screens

The menu as in ux-vision. The tooltip is `rstory — {state}` (`state.*` keys).

## State matrix

| Region | Empty | Normal | Paused | Timed pause | Inactive | No terminal |
|---|---|---|---|---|---|---|
| Entries | `menu.empty` (insensitive) | top 10 | top 10 | top 10 | top 10 (clicks return `inactive`, no-op) | — |
| Status line | `state.recording` | `state.recording` | `state.paused` | `state.paused_until` (refreshed each second while the menu is open, or on the next open) | `state.inactive` | — |
| Pause item | `menu.pause` | `menu.pause` | `menu.resume` | `menu.resume` | `menu.pause` | — |
| Pause 1 min | shown | shown | hidden | hidden | shown | — |
| Open picker | if available | if available | if available | if available | if available | hidden |
| Icon | recording | recording | paused | paused | recording + tooltip "Inactive" | — |

## Interactions

Clear → a submenu with `menu.clear.confirm` / `menu.clear.cancel`. No undo, and the count is stated.

## Accessibility

Text labels on every item. State in words (status line, tooltip). The host provides keyboard navigation.

## Tests

Menu-model snapshot per column of the state matrix. The icon pixmap for each state is present.
