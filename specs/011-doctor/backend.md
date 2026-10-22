# Backend: Doctor

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

Data sources: `session::select` (002), `Store` perms check (001), protocol `status` (004), `launch::plan` (008), extension install reader (010), autostart reader (012), `gnome-shell --version` / D-Bus `ShellVersion`.

## Authorization matrix

N/A (local command).

## Persistence

Read-only.

## Transactions & concurrency

Sequential checks with a 1 s overall budget (each external probe 300 ms timeout).

## Async work

None.

## Integrations

`gnome-shell`, `gnome-extensions` and `gsettings` are optional probes. Missing → `unknown`, never a failure.

## Observability

Doctor output itself.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. Read-only.

## Performance

< 1 s.

## Test plan

Failure-matrix table tests with faked probes.
