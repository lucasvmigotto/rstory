# TUI layer: Terminal picker

Layer added by `frontend:spec` (2026-10-21). Components from `specs/000-design-system/components.md`. Copy keys from `docs/product/ux-vision.md`.

## Screens

**80×24, normal (preview at 40%)**

```
 > reb_                                                    2/14  Recording
 ─────────────────────────────────────────────┬────────────────────────────────
 > 3m      git rebase -i HEAD~3               │ git rebase -i HEAD~3
   2h  📌  git reset --soft HEAD~1 && git p…  │
                                              │
                                              │                        20 B
 ─────────────────────────────────────────────┴────────────────────────────────
 Enter restore  Tab preview  Ctrl-T pin  Ctrl-D delete  Esc quit
```

**120×40**: the same regions, preview at 50%, up to 34 list rows.

**Read-only** (daemon down): a `Banner` line `Read-only: daemon not running` under the query. The footer hints drop Ctrl-T/Ctrl-D.

**Confirm delete**: the footer is replaced by `Delete this entry? y/n` (danger role + the word).

**Empty**: the list area shows `No entries yet. Copy something and it will appear here.`

**Too small** (< 20×5): one line, `Terminal too small (need 20×5).`

**Plain** (`--plain`): inline, see ux-vision.

## Keymap

| Key | Action | Available |
|---|---|---|
| text, Backspace, Ctrl-U, Ctrl-W | edit the query | always (not in confirm) |
| Up/Down, Ctrl-P/Ctrl-N | move the selection | ≥ 1 visible entry |
| PgUp/PgDn, Home/End | jump | ≥ 1 visible entry |
| Enter | restore and exit 0 | a selection exists, not read-only |
| Tab | toggle the preview | always |
| Ctrl-T | pin/unpin | not read-only |
| Ctrl-D | ask to delete | not read-only |
| y / n | confirm / cancel delete | confirm mode |
| F1 | full keymap in the footer (2 lines) | always |
| Esc | cancel confirm, else quit (exit 1) | always |
| Ctrl-C | quit (exit 130) | always |

## Focus order

A single focus. The query is always editable and the list selection is always visible (`>` + reverse).

## States

| Region | Loading | Empty | Partial (filtered) | Success | Error | Offline (daemon down) | Too small |
|---|---|---|---|---|---|---|---|
| Query line | n/a (loaded before the first frame) | `0/0` | `{shown}/{total}` | — | — | `StateBadge` shows nothing (unknown) | — |
| List | — | `picker.empty` | matches only, marked | — | — | list + `Banner` | `picker.too_small` |
| Preview | — | blank | selected content | — | — | content (read-only) | hidden |
| Footer | — | hints | hints | exits | `picker.pin.limit`, error lines | short hints; Enter → `picker.readonly.restore`, exit 3 | — |

## Data

Initial: `list` (or a read-only store snapshot). Preview: `show` on selection change (debounced 30 ms, cached by id). Actions: `restore`, `pin`, `unpin`, `rm`. Live: `subscribe` → re-`list`, keeping the selection by id.

## CLI equivalents

Enter → `rstory copy <ref>` · Ctrl-T → `rstory pin|unpin <ref>` · Ctrl-D → `rstory rm <ref>` · filtering → `rstory ls --json | jq`.

## Tests

insta snapshots for each state at 80×24, 120×40 and 40×10, in 16-color, mono and ASCII. Keystroke journeys on `update()` (K2 recall, pin, pin limit, delete confirm/cancel, Esc, Ctrl-C, read-only Enter). A first-frame bench (F6). The vhs tape is optional.
