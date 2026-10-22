# Feature Specification: Daemon & control socket

**Feature Branch**: `feat/daemon`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Daemon), BR2–BR4b, BR13, BR14, journeys K1, K3b, K4; ADR-0002, ADR-0004.

> Tests are **required** for every story (Constitution II). Protocol messages are validated against `contracts/protocol.schema.json`.

## Clarifications

### Session 2026-10-21

- Q: Where does a restored entry go? → A: To the top (index 0, `last_copied` refreshed).
- Q: Does a pause survive a daemon restart? → A: Yes. An indefinite pause stays until `resume`, and a timed pause keeps its original end time.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Record the clipboard in the background (Priority: P1)

`rstory daemon` runs once per user, takes every clipboard change from the selected backend through the filters into the store, and tells subscribers about it.

**Why this priority**: This is the engine of J1. Nothing else works without it.

**Independent Test**: Run the daemon with a fake backend and a temporary store. Emit three changes and assert that the store has three entries and that a `subscribe` client got three `history_changed` events.

**Acceptance Scenarios**:

1. **Given** no daemon running, **When** `rstory daemon` starts, **Then** it locks the store, binds `$XDG_RUNTIME_DIR/rstory/control.sock` (dir `0700`), selects a backend, and enters *recording*.
2. **Given** a daemon running, **When** a second `rstory daemon` starts, **Then** it exits with code 4 and "already running (pid N)".
3. **Given** *recording*, **When** the backend reports a change, **Then** the entry is in the store and visible through `list` within 100 ms (p95).
4. **Given** backend selection returns `none`, **When** the daemon starts, **Then** it enters *inactive*, keeps serving the socket, and `status` reports the reason.
5. **Given** a burst of 500 changes in 1 s, **When** they're processed, **Then** the daemon stays responsive, the queue is bounded, and drops are counted in `dropped_burst`.

---

### User Story 2 - Serve a restore (Priority: P1)

A client asks for an entry by id. The daemon becomes the clipboard owner and serves it until replaced.

**Why this priority**: The restore half of J1.

**Independent Test**: Fake backend. Send `restore{id}` and assert that `serve(text)` was called, and that the response arrived only after the serve succeeded.

**Acceptance Scenarios**:

1. **Given** an entry with id X, **When** `restore{id:X}` is sent, **Then** the backend serves its content and the response is `ok`.
2. **Given** the daemon is serving X, **When** the backend reports the resulting change as `self_owned`, **Then** no new entry is created, and the restored entry moves to index 0 with a refreshed `last_copied` (BR2).
3. **Given** an unknown id, **When** `restore` is sent, **Then** the response is the error `not_found`.
4. **Given** the daemon is *inactive*, **When** `restore` is sent, **Then** the response is the error `inactive`.

---

### User Story 3 - Pause and timed pause (Priority: P1)

The user pauses recording indefinitely, or for a duration that ends on its own (`pass` gap, J6, J6b).

**Why this priority**: The privacy controls are part of G4, and the timed pause is the owner's chosen answer to S2.

**Independent Test**: With a fake clock, send `pause{for:"50s"}`, emit a change (it's skipped and counted), advance 50 s, emit a change (it's recorded).

**Acceptance Scenarios**:

1. **Given** *recording*, **When** `pause{}` is sent, **Then** the state is `paused` with no end time, subscribers get `pause_changed`, and changes are counted in `skipped_paused`.
2. **Given** *recording*, **When** `pause{for:"50s"}` is sent, **Then** the state is `paused_until = now + 50 s`, and recording resumes at that instant without any client action.
3. **Given** a timed pause, **When** `resume` is sent before the end, **Then** recording resumes immediately.
4. **Given** a pause (timed or not), **When** the daemon restarts, **Then** the pause is still in force: an indefinite pause stays until `resume`, and a timed pause keeps its original end time (resuming immediately if that time has passed).

---

### User Story 4 - Status and housekeeping (Priority: P2)

Clients can ask for the daemon state and counters, and the daemon expires entries on time without polling.

**Why this priority**: Diagnosability (Constitution VIII) and expiry (G4).

**Independent Test**: Fake clock and store. Check that `status` reports the state, backend, entry count and counters, and that an entry expires exactly at its expiry instant through a single armed timer.

**Acceptance Scenarios**:

1. **Given** a running daemon, **When** `status` is sent, **Then** the result has `state`, `backend`, `mode`, `reason`, `entries`, `pinned`, `paused_until`, `counters{skipped_sensitive, skipped_oversize, skipped_paused, dropped_burst}`, `version`, `protocol`.
2. **Given** the earliest expiry is in 3 h, **When** the daemon idles, **Then** exactly one timer is armed for that instant, and there are no other periodic wake-ups.

### Edge Cases

- A stale socket file from a crashed daemon: if the connection is refused and no lock is held, unlink it and bind.
- `$XDG_RUNTIME_DIR` unset: fall back to `/run/user/<uid>` if it exists and is owned by the user, otherwise exit with code 5 and a clear message.
- A client whose uid doesn't match (`SO_PEERCRED`) is disconnected without a response.
- A malformed JSON line gets an `bad_request` error, and the connection stays open.
- A line longer than 2 MiB gets `too_large`, and the connection is closed.
- The X11 connection is lost on Wayland (XWayland restart): reconnect with backoff 100 ms → 5 s. The state is *inactive* until it reconnects.
- SIGTERM or SIGINT: stop serving, flush the store, remove the socket, exit 0.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The daemon MUST enforce a single instance per user through the store's exclusive lock plus the socket bind.
- **FR-002**: The control socket MUST live at `$XDG_RUNTIME_DIR/rstory/control.sock` in a `0700` directory, and MUST check `SO_PEERCRED` uid equality on every connection.
- **FR-003**: The protocol MUST be NDJSON v1 as defined in `contracts/protocol.md`: `list`, `show`, `restore`, `pin`, `unpin`, `rm`, `clear`, `pause`, `resume`, `status`, `open_picker` (spec 008), `subscribe`.
- **FR-004**: The capture pipeline MUST be: backend event → filters (spec 005) → store insert (batched when more than one is pending) → `history_changed` event. The queue is bounded at 256, and on overflow the oldest pending changes are dropped and counted.
- **FR-005**: Restore MUST reply only after the backend confirms ownership.
- **FR-006**: The daemon MUST support `pause` (indefinite), `pause{for}` (a duration of 1 s–24 h) and `resume`, emit `pause_changed` events, and persist the pause state in the store's index so it survives restarts.
- **FR-012**: The protocol MUST offer `clear{all:true}` (removes pinned entries too). Plain `clear` keeps pinned entries.
- **FR-007**: Expiry and timed-pause ends MUST use one-shot timers re-armed after each change, with no periodic polling.
- **FR-008**: States MUST be `recording`, `paused` or `inactive`, exposed in `status`.
- **FR-009**: Signals MUST be handled for a clean shutdown (flush, unlink the socket).
- **FR-010**: Logs MUST go to stderr through `tracing` (`RUST_LOG`, default `info`) and MUST contain no content.
- **FR-011**: Exit codes: 0 clean, 4 already running, 5 environment error (runtime dir, store too open), 1 unexpected.

### Key Entities

- **Daemon state**: `recording | paused{until?} | inactive{reason}`.
- **Counters**: skipped_sensitive, skipped_oversize, skipped_paused, dropped_burst (since start).
- **Subscription**: a client connection receiving `history_changed` and `pause_changed`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Capture-to-listed p95 under 100 ms (fitness F7) on headless sway and Xvfb.
- **SC-002**: Daemon RSS ≤ 20 MiB after 10 000 captures with the default caps.
- **SC-003**: Idle for 60 s: no wake-ups beyond the armed timers (fitness F10).
- **SC-004**: 100% of protocol messages in the contract tests validate against the schema.

## Assumptions

- The tray (spec 009) runs inside the daemon process and uses the same internal API as socket requests.
