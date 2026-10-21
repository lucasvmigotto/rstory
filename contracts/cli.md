# rstory CLI contract

Status: **skeleton**. Owner: spec 006 (`project:spec` → `frontend:spec` for `tui.md`). Constitution VI.

## Commands

| Command | Daemon needed | Output | Notes |
|---|---|---|---|
| `rstory daemon [--backend wayland\|x11]` | is the daemon | logs to stderr | spec 004 |
| `rstory ls [--json] [--limit N]` | no (read-only fallback) | lines, or `entry_list` JSON | spec 006 |
| `rstory show <ref> [--json]` | no (read-only fallback) | raw bytes, or `entry` JSON | spec 006 |
| `rstory copy <ref>` | yes | silent | spec 006 |
| `rstory pin <ref>` / `unpin <ref>` | yes | silent | spec 006 |
| `rstory rm <ref>` | yes | silent | spec 006 |
| `rstory clear [--all] [--yes]` | yes | "removed N" on stderr | prompt on a TTY |
| `rstory pause [--for <dur>]` / `resume` | yes | silent | `<dur>` = `<n>s\|m\|h` |
| `rstory status [--json]` | yes (exits 3 otherwise) | text, or `status` JSON | |
| `rstory config [--path]` | no | effective TOML or the path | |
| `rstory pick [--plain]` | no (read-only fallback) | TUI | spec 007 |
| `rstory doctor [--json]` | no | report | spec 011 |
| `rstory install-autostart [--systemd] [--now] [--uninstall]` | no | messages | spec 012 |
| `rstory install-gnome-extension [--enable] [--uninstall]` | no | messages | spec 010 |
| `rstory completions <bash\|zsh\|fish>` | no | script | spec 006 |

## Exit codes (stable within a major version)

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | unexpected error (picker: dismissed with Esc) |
| 2 | usage error, not found or ambiguous ref, no TTY where one is needed |
| 3 | daemon not running |
| 4 | daemon already running |
| 5 | environment error (runtime dir, store too open, no systemd user) |
| 6 | limit reached (pin cap) |
| 7 | inactive (no capture backend) |
| 130 | interrupted (Ctrl-C) |

Doctor uses its own scale: 0 ok/info, 1 warn, 2 fail (spec 011).

## JSON shapes (`"schema": 1`)

- `entry_list`: `{"schema":1,"entries":[EntrySummary…],"read_only":bool}`
- `entry`: `{"schema":1,"entry":EntrySummary,"content":string}`
- `status`: `{"schema":1, …Status}` (see `protocol.md`)
- `doctor`: `{"schema":1,"checks":[{"id","status":"ok|info|warn|fail","message","hint"}]}`

## Parity (Constitution VI)

| Action | Picker | Tray (009) | GNOME panel (010) | CLI |
|---|---|---|---|---|
| Restore an entry | Enter | entry item | entry item | `copy <ref>` |
| Pin / unpin | Ctrl-T | — | — | `pin` / `unpin` |
| Delete one | Ctrl-D | — | — | `rm <ref>` |
| Clear unpinned | — | Clear history… | Clear history… | `clear` |
| Clear all | — | — | — | `clear --all` |
| Pause / resume | — | Pause/Resume | Pause/Resume | `pause` / `resume` |
| Pause 1 minute | — | Pause for 1 minute | Pause for 1 minute | `pause --for 1m` |
| Open picker | — | Open picker | Open picker | `pick` |
| Search | typing | — | — | `ls \| grep`, `ls --json \| jq` |
