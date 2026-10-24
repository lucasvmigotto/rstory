# Backend: Wayland data-control capture

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

Internal: `WaylandBackend` implementing the `Backend` trait.

| API | Behavior | Errors |
|---|---|---|
| `connect()` | registry; bind `ext_data_control_manager_v1` (else `zwlr`), seat, data device | `NoDataControl`, `Unreachable` |
| `events()` | `data_offer` + `selection` → `SelectionChanged{types, self_owned}`; null → `SelectionCleared` | — |
| `read_text(limit)` | `receive(mime, fd)`, non-blocking read with limit and a 2 s timeout | `TooLarge`, `Timeout`, `Disconnected` |
| `serve(text)` | create source, offer MIME set, `set_selection`; `send` handled per fd in a task; `cancelled` → stop | `OwnershipFailed` |

## Authorization matrix

N/A: internal, used by the daemon only.

## Persistence

None.

## Transactions & concurrency

One connection task. `send` writes run as separate tasks over shared `Arc<[u8]>` content.

## Async work

`AsyncFd` on the Wayland socket and on each pipe.

## Integrations

Compositor: `Disconnected` → the daemon exits (session over). ACL: offers → type names.

## Observability

Span `wl.read{mime, bytes, ms}`, event `wl.selection_changed{types, self_owned}`.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution.

## Performance

Event → text p95 < 20 ms. A 1 MiB round trip < 200 ms.

## Test plan

Headless-sway integration (ext and zwlr paths, hint visible, clear, serve, cancelled, EPIPE).
