# Backend: SNI tray

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

In-process: the tray calls the daemon's command handlers (restore, pause, resume, clear, open_picker) directly. ksni `Tray` impl renders the `TrayMenu` model.

## Authorization matrix

In-process. The session bus is per-user.

## Persistence

None.

## Transactions & concurrency

The ksni service runs on the same tokio runtime. Menu updates are triggered by history and pause events.

## Async work

Watcher `NameOwnerChanged` subscription (zbus).

## Integrations

D-Bus session bus (zbus): no bus → `no_bus`. No watcher → `no_host`. Re-register when the watcher appears.

## Observability

`tray.state{state}` log event, `status.tray`.

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. Menu labels escaped and truncated previews (no control characters).

## Performance

Menu rebuild < 1 ms.

## Test plan

Menu-model unit tests, a dbus-daemon integration with a stub watcher, manual KDE/waybar.
