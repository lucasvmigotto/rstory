# Backend: Session detection & X11 capture (incl. XWayland bridge)

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

Internal: the `Backend` trait (`contracts/backend.md`) implemented by `X11Backend`; `session::select(env, probe, override) -> BackendSelection`.

| API | Behavior | Errors |
|---|---|---|
| `X11Backend::connect(display)` | RustConnection, XFIXES ≥ 2, hidden InputOnly window, `SelectSelectionInput(CLIPBOARD)` | `Unreachable`, `NoXfixes` |
| `events()` | `XfixesSelectionNotify` → `SelectionChanged{types (via TARGETS), self_owned}`; owner 0 → `SelectionCleared`; connection error → `Disconnected` | — |
| `read_text(limit)` | ConvertSelection best target, INCR, abort over limit | `TooLarge`, `ConversionRefused`, `Timeout` (2 s), `Disconnected` |
| `serve(text)` | SetSelectionOwner + verify; answer SelectionRequest (TARGETS, TIMESTAMP, UTF8_STRING, STRING, TEXT, text/plain;charset=utf-8), INCR for large values | `OwnershipFailed` |
| `session::select` | rule table in `data-model.md` | — |

## Authorization matrix

N/A: an internal API, used by the daemon only (fitness F1).

## Persistence

None.

## Transactions & concurrency

Single task owns the connection. The serve state machine handles concurrent SelectionRequests from several requestors (one INCR transfer per requestor+property).

## Async work

Connection fd registered via `AsyncFd`. Read timeout of 2 s via `tokio::time::timeout`.

## Integrations

X server / XWayland: connect timeout of 3 s (XWayland on-demand start), `Disconnected` → the daemon's reconnect policy (100 ms → 5 s, bridge mode only). ACL: atoms → type-name strings.

## Observability

Span `x11.read{target, bytes, ms}`, event `x11.selection_changed{types, self_owned}`, counters for conversion refusals. Types only.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. Refuse `MULTIPLE` and unknown targets. Never read when the filter skipped the change.

## Performance

Event → text p95 < 20 ms. A 1 MiB INCR round trip < 200 ms.

## Test plan

Xvfb integration (capture, serve, INCR, owner exit, self_owned), bridge-mode integration on headless sway, a selection table test, a Latin-1 conversion unit test.
