# TUI/CLI layer: Command-line interface

Layer added by `frontend:spec` (2026-10-21). The source of truth for commands, JSON shapes and exit codes is `/contracts/cli.md`. This file specifies the **output design**.

## Screens (non-interactive output)

`rstory ls` on a TTY at 80 cols:

```
 0  01JBD8Z  3m      https://github.com/lucasvmigotto/rstory
 1  01JBD8X  2h  📌  git rebase -i HEAD~3
 2  01JBC2K  1d      cargo test -p rstory --test daemon_restore -- --nocapture…
```

At 120 cols the previews get the extra width. When piped (not a TTY): no colors and no truncation beyond 256 chars, tab-separated `index\tshort_id\tpinned\tage_seconds\tpreview`, for `cut`/`awk`.

`rstory status`:

```
Recording · backend x11 (XWayland bridge) · 8 entries (1 pinned) of 20
Skipped: 2 sensitive · 0 oversize · 5 while paused
Tray: no host · Picker: ghostty
```

Prompts (`clear`), on a TTY only: `Delete 12 unpinned entries? This can't be undone. [y/N] `

## Keymap

N/A (non-interactive). Prompts accept `y`/`yes` (case-insensitive). Anything else, including EOF, means no.

## States

| Command | Daemon up | Daemon down | Inactive | Empty history | Error |
|---|---|---|---|---|---|
| `ls` | list | list + stderr `cli.readonly` | list | no output, exit 0 | — |
| `show` | bytes | bytes (read-only) | bytes | `cli.not_found`, exit 2 | ambiguous → `cli.ambiguous`, exit 2 |
| `copy` | silent, exit 0 | `cli.daemon_down`, exit 3 | `cli.inactive`, exit 7 | `cli.not_found`, exit 2 | — |
| `pin` | silent | exit 3 | silent (works) | exit 2 | `cli.pin_limit`, exit 6 |
| `clear` | prompt → `cli.clear.done` | exit 3 | works | `Removed 0 entries.` | no TTY and no `--yes` → `cli.needs_yes`, exit 2 |
| `pause`/`resume` | silent | exit 3 | works | — | bad duration → usage error, exit 2 |
| `status` | text/JSON | `cli.daemon_down`, exit 3 | shows Inactive | — | — |

## Data

`ls`→`list` · `show`→`show` · `copy`→`restore` · `pin`/`unpin`/`rm`/`clear`/`pause`/`resume`/`status` → the same-named operations (`/contracts/protocol.md`). The fallback for `ls`/`show` is the read-only store.

## CLI equivalents

This feature *is* the CLI. See the parity table in `/contracts/cli.md`.

## Tests

insta snapshots of `ls` (TTY at 80/120 and piped), `status`, the prompts and every error line. assert_cmd for exit codes. Schema validation for every `--json`.
