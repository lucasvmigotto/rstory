# rstory control protocol v1

Status: **canonical** (promoted by `backend:spec`, 2026-10-21). Changes are spec changes (Constitution IV), and a breaking change bumps `v`.
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

## Error catalog

| code | When | Retry? | CLI exit |
|---|---|---|---|
| `bad_request` | malformed JSON, missing or invalid args (e.g. `for` out of 1s–24h) | no | 2 |
| `too_large` | request line > 2 MiB (the connection is closed) | no | 1 |
| `unsupported_version` | `v` ≠ 1 | no | 1 |
| `not_found` | the ref matches no entry | no | 2 |
| `ambiguous` | the short id matches several entries; `message` lists the candidates' short ids | no | 2 |
| `pin_limit` | pinning would exceed `max_pinned` | no | 6 |
| `inactive` | restore while there's no capture backend | after the backend returns | 7 |
| `backend_error` | the backend failed to take ownership | yes (idempotent) | 1 |
| `no_terminal` | `open_picker` with no terminal resolvable | no | — |
| `picker_command_failed` | the configured `picker_command` couldn't be spawned | no | — |
| `internal` | an unexpected daemon error (logged with a correlation id, never content) | maybe | 1 |

## Idempotency & concurrency

- `restore`, `pin`, `unpin`, `resume` and `pause` (with the same args) are idempotent. `rm` and `clear` are safe to retry (a second `rm` returns `not_found`).
- The daemon applies requests strictly in arrival order on one event loop, so there are no lost updates and no version checks are needed.
- An index-based ref resolves against the history at the moment of processing. Scripts that need stability use short ids.

## Examples (one per operation)

```jsonl
{"v":1,"id":1,"op":"list","args":{"limit":2}}
{"v":1,"id":1,"ok":true,"result":{"entries":[{"id":"01JBD8Z0R5S7K3M2N1P0Q9T8V7","short_id":"01JB","index":0,"size":38,"created":"2026-10-21T14:02:11Z","last_copied":"2026-10-21T14:02:11Z","pinned":false,"preview":"https://github.com/lucasvmigotto/rstory"},{"id":"01JBD8XQ4H2G6F5E4D3C2B1A09","short_id":"01JBD8X","index":1,"size":20,"created":"2026-10-21T13:58:40Z","last_copied":"2026-10-21T13:58:40Z","pinned":true,"preview":"git rebase -i HEAD~3"}]}}
{"v":1,"id":2,"op":"show","args":{"ref":"1"}}
{"v":1,"id":2,"ok":true,"result":{"entry":{"id":"01JBD8XQ4H2G6F5E4D3C2B1A09","short_id":"01JBD8X","index":1,"size":20,"created":"2026-10-21T13:58:40Z","last_copied":"2026-10-21T13:58:40Z","pinned":true,"preview":"git rebase -i HEAD~3"},"content":"git rebase -i HEAD~3"}}
{"v":1,"id":3,"op":"restore","args":{"ref":"01JBD8X"}}
{"v":1,"id":3,"ok":true,"result":{}}
{"v":1,"id":4,"op":"pin","args":{"ref":"0"}}
{"v":1,"id":4,"ok":false,"error":{"code":"pin_limit","message":"pin limit reached (10)"}}
{"v":1,"id":5,"op":"unpin","args":{"ref":"01JBD8X"}}
{"v":1,"id":5,"ok":true,"result":{"entry":{"id":"01JBD8XQ4H2G6F5E4D3C2B1A09","short_id":"01JBD8X","index":0,"size":20,"created":"2026-10-21T13:58:40Z","last_copied":"2026-10-21T14:05:02Z","pinned":false,"preview":"git rebase -i HEAD~3"}}}
{"v":1,"id":6,"op":"rm","args":{"ref":"zzzz"}}
{"v":1,"id":6,"ok":false,"error":{"code":"not_found","message":"no entry zzzz"}}
{"v":1,"id":7,"op":"clear","args":{"all":false}}
{"v":1,"id":7,"ok":true,"result":{"removed":12}}
{"v":1,"id":8,"op":"pause","args":{"for":"50s"}}
{"v":1,"id":8,"ok":true,"result":{"state":{"state":"paused","paused_until":"2026-10-21T14:06:00Z"}}}
{"v":1,"id":9,"op":"resume","args":{}}
{"v":1,"id":9,"ok":true,"result":{"state":{"state":"recording","paused_until":null}}}
{"v":1,"id":10,"op":"status","args":{}}
{"v":1,"id":10,"ok":true,"result":{"state":"recording","paused_until":null,"backend":"x11","mode":"xwayland_bridge","entries":8,"pinned":1,"max_entries":20,"max_pinned":10,"counters":{"skipped_sensitive":2,"skipped_oversize":0,"skipped_paused":5,"dropped_burst":0},"tray":"no_host","picker_available":true,"version":"0.1.0","protocol":1}}
{"v":1,"id":11,"op":"open_picker","args":{}}
{"v":1,"id":11,"ok":true,"result":{"argv":["ghostty","-e","rstory","pick"]}}
{"v":1,"id":12,"op":"subscribe","args":{"events":["history_changed","pause_changed"]}}
{"v":1,"id":12,"ok":true,"result":{}}
{"v":1,"event":"history_changed","data":{"entries":9,"top":"01JBD9A1B2C3D4E5F6G7H8J9K0"}}
{"v":1,"event":"pause_changed","data":{"state":"paused","paused_until":null}}
```

These examples are copied verbatim into `tests/fixtures/protocol/examples.jsonl` and validated in CI.
