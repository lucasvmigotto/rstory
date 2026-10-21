# Menu contract (shared by 009 tray and 010 GNOME extension)

Order and labels:
1. up to 10 entry items — label: preview (newlines → `⏎`, ≤ 48 chars, `…`), prefixed `📌 ` when pinned; activate → restore
2. separator
3. status label (insensitive): `Recording` | `Paused` | `Paused — resumes in M:SS` | `Inactive — <reason>`
4. `Pause recording` / `Resume recording`
5. `Pause for 1 minute` (hidden while paused)
6. `Open picker` (only when `picker_available`)
7. separator
8. `Clear history…` → `Delete N unpinned entries` / `Cancel`
