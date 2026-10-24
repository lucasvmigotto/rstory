# Backend: Privacy filters

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

Internal domain service `CopyFilter::decide(types, pause, caps) -> PreRead` and `CopyFilter::after_read(bytes) -> FilterDecision`, called by the daemon pipeline (004).

## Authorization matrix

N/A: internal.

## Persistence

None. Counters live in the daemon (004).

## Transactions & concurrency

Pure functions.

## Async work

None.

## Integrations

None.

## Observability

The `capture.decision{decision}` log event (in 004). Counters in `status`.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. Hinted content never read (decision on types only). Rejected content dropped immediately.

## Performance

Decision < 1 µs.

## Test plan

proptest (no hinted bytes persisted or logged), boundaries, order-of-checks table, F11 canary.
