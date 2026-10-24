# Backend: Store & history

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Technical decisions (first backend feature)

| Concern | Choice | Rationale |
|---|---|---|
| Errors | `thiserror` 2 for typed errors per module; no `anyhow` in library code (only in `main.rs`) | stable error kinds map to protocol codes and exit codes |
| Logging | `tracing` + `tracing-subscriber` (`fmt`, `env-filter`), stderr; `Redacted<T>` newtype for anything content-derived | Constitution III, VIII |
| Persistence | no ORM/DB: hand-written `store` over `Fs` trait (ADR-0003) | real deletion, pure Rust |
| Time | `time` 0.3 (UTC, RFC 3339 serde) | no chrono, smaller |
| Testing | std test, `proptest` 1, `insta` 1, `tempfile` 3, `assert_cmd` 2 | per plans |
| Lints | `#![forbid(unsafe_code)]`, `clippy::pedantic` warn subset, `cargo deny` | Constitution I |

## Endpoints / interfaces

Internal API (Rust), consumed by the daemon (004) and, read-only, by the CLI (006), picker (007) and doctor (011). No protocol operation is defined here.

| API | Maps to (domain) | Validation | Side effects | Errors |
|---|---|---|---|---|
| `Store::open_exclusive(dir, caps)` | load the History + RecordingState | perms ≤ 0700/0600, index `version` == 1 | sweep orphans; holds LOCK_EX | `StoreTooOpen`, `UnknownVersion`, `Locked`, `Io` |
| `Store::open_read_only(dir)` | snapshot | same perms check (warn only) | LOCK_SH, non-blocking | `DaemonRunning` (LOCK_EX held), `UnknownVersion`, `Io` |
| `Store::apply(batch: Vec<Mutation>)` | History commands (record, touch, pin, unpin, remove, clear, clear_all, expire, set_pause) | per H1–H10 | content files written first, then one index commit, then unlinks | `PinLimit`, `NotFound`, `TooLarge`, `Io` |
| `Store::content(id)` | read entry content | id exists | — | `NotFound`, `Io` |
| `History::list() / resolve(ref) / next_deadline()` | read models | — | — | `NotFound`, `Ambiguous` |

## Authorization matrix

N/A: an internal API. The access control is the filesystem (0700/0600) and the flock.

## Persistence

Files (ADR-0003), all `0600`, directory `0700`:

| File | Content | Constraints |
|---|---|---|
| `store.lock` | empty; flock target | — |
| `index.json` | `{version:1, entries:[…], state:{pause}}` | written only via temp+fsync+rename+dirfsync; unknown `version` → refuse |
| `content/<id>` | raw bytes | written once (temp+rename); never modified; unlinked on delete |

**Migrations**: `version` field. v1 is the initial format. A future v2 ships a forward migration that runs on open (read v1 → write v2 atomically), keeps a `index.v1.bak` until the next successful start, and is tested on a populated v1 fixture.

## Transactions & concurrency

Single writer (LOCK_EX for the daemon's lifetime). Readers take LOCK_SH and see a consistent snapshot thanks to the atomic rename. One `apply` = one transaction (one index commit).

## Async work

None in this module. The daemon arms timers from `next_deadline()`.

## Integrations

Filesystem only. A full disk or I/O error fails the `apply` without changing the previous index. The daemon reports it and keeps running.

## Observability

`tracing` spans `store.apply{mutations, entries_after}` and `store.open{entries, swept}`, plus a duration histogram for the evolution trigger E3. Never content or previews.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. Real deletion (H8), and perms enforced on open (FR-004).

## Performance

open+list (500) < 10 ms; `apply` single insert ≤ 20 ms (2 fsync); batch of N ≤ 20 ms + N × 1 ms.

## Test plan

Unit + proptest on `History` (H1–H10), crash injection through the fault `Fs` (SC-001), permissions (F9), orphan sweep, unknown version, read-only lock semantics, a bench.
