# Component inventory

| Component | Purpose | Anatomy | States | Keyboard | Content rules |
|---|---|---|---|---|---|
| `QueryLine` | shows the query and counts | `> ` + query + cursor · right: `{shown}/{total}` + `StateBadge` | empty, typing, no matches (`0/{total}`) | text editing keys | query is never truncated from the left beyond the width (scrolls) |
| `EntryRow` | one entry in a list | selection marker · age · pin · preview with match marks | default, selected, pinned, matched, selected+pinned | — | one line; control characters escaped; `…` truncation; wide chars by display width |
| `PreviewPane` | full content of the selected entry | wrapped text, escaped control characters, a `size` footer line | content, empty (no selection), hidden | Tab toggles | max 1 MiB shown lazily (first 64 KiB, then "… N more bytes") |
| `Footer` | hints, status messages, confirms | one line | hints (full/short), status (pin limit, restoring), confirm (`Delete this entry? y/n`) | y/n in confirm, Esc cancels | copy keys from ux-vision |
| `Banner` | read-only notice | one line above the list, `warning` role + words | shown/hidden | — | `picker.readonly.banner` |
| `StateBadge` | recording state | word: Recording / Paused / Paused — resumes in m:ss / Inactive | 4 states | — | words always; color is extra |
| `EntryLine` (CLI `ls`) | non-interactive list line | index · short id · pin · age · preview | pinned, plain | — | width-aware on a TTY; piped output prints the full preview (≤ 256 chars) as TSV with no colors |
| `CheckLine` (doctor) | one check result | glyph · label · message · hint (indented) | ok, info, warn, fail | — | the glyph has an ASCII fallback; the hint starts with "→" (ASCII "->") |
| Menu item (tray/panel) | see `specs/009-tray/contracts/menu.md` | label | sensitive, insensitive, hidden | host-provided | ≤ 48 chars, `⏎` for newlines |
