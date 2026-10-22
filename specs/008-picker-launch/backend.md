# Backend: Picker launch (terminal detection)

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

`open_picker` operation (handled in 004 via `launch::plan()` + `launch::spawn()`), `status.picker_available`.

## Authorization matrix

`open_picker`: daemon's uid allowed (tray in-process, extension via socket).

## Persistence

None.

## Transactions & concurrency

Spawn is fire-and-forget (setsid, null stdio).

## Async work

None.

## Integrations

Terminal emulators via the table. A failed spawn → `picker_command_failed`.

## Observability

`launch.spawn{source, program}` (no args beyond the program name).

## Security

Constitution III: no content in logs or errors (`Redacted<T>`), peer uid check at the socket, no network. ASVS doesn't apply (no web surface). The local equivalent is in the constitution. No shell (argv only). `picker_command` split with shell-words.

## Performance

Resolution < 5 ms.

## Test plan

Table tests per terminal, resolution order, spawn with a stub script, protocol test for `open_picker`.
