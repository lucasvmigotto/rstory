# Backend: Command-line interface

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

CLI backend part: the blocking protocol client (`src/ipc/client.rs`) and the read-only fallback through `Store::open_read_only`. The command surface itself is in `contracts/cli.md` (frontend layer).

## Authorization matrix

Inherits the socket's uid check (004).

## Persistence

Read-only via 001.

## Transactions & concurrency

One request at a time per invocation.

## Async work

None (blocking `UnixStream`, 2 s connect/read timeouts, except `restore`, which waits up to 5 s).

## Integrations

Daemon socket: absent → read-only fallback (ls/show) or exit 3 (writes).

## Observability

`RUST_LOG` honored. Errors to stderr with codes.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution.

## Performance

`ls` < 50 ms with 500 entries.

## Test plan

Client unit tests against a fake socket server, fallback tests.
