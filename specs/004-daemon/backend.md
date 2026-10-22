# Backend: Daemon & control socket

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

Protocol v1 operations (canonical: `/contracts/protocol.md`).

| operationId | Actor | Domain command/query | Validation | Side effects / events | Idempotency | Errors |
|---|---|---|---|---|---|---|
| `list` | user (CLI, picker, extension) | History list | `limit` 1–500 | — | read | `bad_request` |
| `show` | user | resolve + content | `ref` pattern | — | read | `not_found`, `ambiguous` |
| `restore` | user, tray | `RestoreEntry` + backend serve | ref; backend active | `EntryRestored` → `history_changed` | yes | `not_found`, `ambiguous`, `inactive`, `backend_error` |
| `pin` / `unpin` | user | `PinEntry` / `UnpinEntry` | ref | `history_changed` | yes | `not_found`, `ambiguous`, `pin_limit` |
| `rm` | user | `RemoveEntry` | ref | `history_changed` | retry-safe | `not_found`, `ambiguous` |
| `clear` | user, tray | `ClearHistory{all}` | `all` bool | `history_changed` | retry-safe | — |
| `pause` | user, tray | `PauseRecording{until?}` | `for` 1s–24h | `pause_changed`; persisted | yes | `bad_request` |
| `resume` | user, tray | `ResumeRecording` | — | `pause_changed`; persisted | yes | — |
| `status` | user, tray, doctor | status read model | — | — | read | — |
| `open_picker` | tray, extension | launch (008) | — | spawn | no (spawns each time) | `no_terminal`, `picker_command_failed` |
| `subscribe` | extension, picker | — | event names | stream | — | `bad_request` |

## Authorization matrix

| Operation | Daemon's own uid (via socket, `SO_PEERCRED`) | Other uid | In-process (tray) |
|---|---|---|---|
| `list` | allow | deny (closed) | allow |
| `show` | allow | deny (closed) | n/a |
| `restore` | allow | deny (closed) | allow |
| `pin` | allow | deny (closed) | n/a |
| `unpin` | allow | deny (closed) | n/a |
| `rm` | allow | deny (closed) | n/a |
| `clear` | allow | deny (closed) | allow |
| `clear{all}` | allow | deny (closed) | n/a |
| `pause` | allow | deny (closed) | allow |
| `resume` | allow | deny (closed) | allow |
| `status` | allow | deny (closed) | allow |
| `open_picker` | allow | deny (closed) | allow |
| `subscribe` | allow | deny (closed) | n/a |

Every cell is a test: the "other uid" column is covered once in `tests/daemon_socket.rs` (connection closed before any op), and the allowed cells by each op's contract test.

## Persistence

Via 001. Pause state persisted in `index.json` `state.pause`. Counters are in memory.

## Transactions & concurrency

All commands are serialized on the current-thread runtime. The capture writer drains its queue (bound 256) into one `Store::apply` batch. A restore races an external copy: last owner wins (domain model §10).

## Async work

Timers: one `sleep_until(next_deadline)` (expiry and pause end), re-armed after every mutation. No periodic jobs. Reconnect loop for bridge mode.

## Integrations

Backend (002/003) via the trait. Launch (008). Tray (009) in-process. SIGTERM/SIGINT → graceful shutdown.

## Observability

Log events: `daemon.start{backend, mode, version}`, `daemon.state{state}`, `capture.decision{decision, bytes?}`, `request{op, ms, ok, code?}` with a per-connection correlation id. Counters are exposed in `status`. No content, no previews.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. Line limit 2 MiB. Malformed input → `bad_request`, never a panic. Socket dir 0700. Stale socket unlinked only while holding the store lock.

## Performance

capture → listed p95 < 100 ms (F7); request handling p95 < 5 ms (except restore, which waits for ownership); RSS ≤ 20 MiB; idle with timers only (F10).

## Test plan

Contract tests (fixtures + examples validated against the schema, responses validated), integration with `FakeBackend` (capture, restore, pause, status, burst), socket hardening, F10 idle, F11 canary, RSS.
