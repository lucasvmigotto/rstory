# rstory control protocol v1

Status: **skeleton** (written by `project:spec`; `backend:spec` makes it canonical)
Constitution IV · ADR-0002 · Machine-readable schema: `protocol.schema.json`

## Transport

- A Unix stream socket at `$XDG_RUNTIME_DIR/rstory/control.sock` (the directory is `0700`).
- The server checks `SO_PEERCRED`: the uid must equal the daemon's uid, otherwise it closes without replying.
- Framing: one JSON object per line (UTF-8, `\n`-terminated). Maximum line length 2 MiB. Longer lines get `too_large`, then the server closes.
- Multiple requests per connection are allowed. Responses carry the request's `id`. Events (after `subscribe`) carry no `id`.
- Unknown fields are ignored. An unknown `v` gets `unsupported_version`.

## Envelope

```json
{"v":1,"id":7,"op":"list","args":{}}
{"v":1,"id":7,"ok":true,"result":{...}}
{"v":1,"id":7,"ok":false,"error":{"code":"not_found","message":"no entry ab12"}}
{"v":1,"event":"history_changed","data":{"entries":20,"top":"01JB…"}}
```

## Operations

| op | args | result | errors | Used by |
|---|---|---|---|---|
| `list` | `{limit?: int (default all)}` | `{entries: [EntrySummary]}` (newest first) | — | CLI `ls`, picker, extension |
| `show` | `{ref: string}` (index or short id) | `{entry: EntrySummary, content: string}` | `not_found`, `ambiguous` | CLI `show`, picker preview |
| `restore` | `{ref: string}` | `{}` (after ownership is confirmed) | `not_found`, `ambiguous`, `inactive`, `backend_error` | CLI `copy`, picker, tray, extension |
| `pin` | `{ref}` | `{entry: EntrySummary}` | `not_found`, `ambiguous`, `pin_limit` | CLI, picker |
| `unpin` | `{ref}` | `{entry: EntrySummary}` | `not_found`, `ambiguous` | CLI, picker |
| `rm` | `{ref}` | `{}` | `not_found`, `ambiguous` | CLI, picker |
| `clear` | `{all?: bool (default false)}` | `{removed: int}` | — | CLI, tray, extension (`all` from the CLI only) |
| `pause` | `{for?: string ("50s","2m","1h"; 1s–24h)}` | `{state: State}` | `bad_request` | CLI, tray, extension |
| `resume` | `{}` | `{state: State}` | — | CLI, tray, extension |
| `status` | `{}` | `Status` | — | CLI, tray, extension, doctor |
| `open_picker` | `{}` | `{argv: [string]}` | `no_terminal`, `picker_command_failed` | tray, extension |
| `subscribe` | `{events?: ["history_changed","pause_changed"]}` | `{}`, then events | — | extension, picker |

Generic errors: `bad_request`, `too_large`, `unsupported_version`, `internal`.

## Types

- **EntrySummary**: `{id: string (26-char time-ordered), short_id: string (≥4), index: int, size: int (bytes), created: RFC3339, last_copied: RFC3339, pinned: bool, preview: string (≤256 chars, control characters escaped)}`
- **State**: `{state: "recording"|"paused"|"inactive", paused_until?: RFC3339|null, reason?: string}`
- **Status**: `State & {backend: "wayland"|"x11"|"none", mode: "native"|"xwayland_bridge"|null, entries: int, pinned: int, max_entries: int, max_pinned: int, counters: {skipped_sensitive, skipped_oversize, skipped_paused, dropped_burst: int}, tray: "active"|"no_host"|"no_bus"|"disabled", picker_available: bool, version: string, protocol: 1}`

## Events

- `history_changed`: `{entries: int, top: string|null}`. Clients re-`list` when they need more.
- `pause_changed`: `State`.

## Invariants

- Content never appears in events, only in `show` results.
- A `restore` response is sent only after the backend has confirmed clipboard ownership.
- Every mutation that changes the history emits exactly one `history_changed`.
