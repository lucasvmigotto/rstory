# Feature Specification: Wayland data-control capture

**Feature Branch**: `feat/wayland-capture`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Capture & backends), Tier 1 (KDE, sway); ADR-0006.

> Tests are **required** for every story (Constitution II). Integration tests run against headless sway (`WLR_BACKENDS=headless`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture on data-control compositors (Priority: P1)

On KDE, sway and other compositors with data-control, rstory sees every clipboard change in the background, without focus.

**Why this priority**: KDE and wlroots are Tier 1.

**Independent Test**: Headless sway plus a Wayland client that sets the selection "hello". The backend emits one change with `text/plain;charset=utf-8` among the types and reads "hello".

**Acceptance Scenarios**:

1. **Given** a compositor offering `ext_data_control_manager_v1`, **When** a client sets the selection, **Then** the backend emits `SelectionChanged{types}` and reads the text through a pipe.
2. **Given** a compositor offering only `zwlr_data_control_manager_v1`, **When** the same happens, **Then** capture works the same through the fallback.
3. **Given** an offer containing `x-kde-passwordManagerHint`, **When** the change is delivered, **Then** the types include it, so the filter can skip it without reading (spec 005).
4. **Given** the selection is cleared (a null offer), **When** that happens, **Then** `SelectionCleared` is emitted.

---

### User Story 2 - Restore on data-control compositors (Priority: P1)

rstory sets the selection through a data-control source and serves the text until another client replaces it.

**Why this priority**: The restore half of J1 on Wayland.

**Independent Test**: Under headless sway, the backend serves "restored". A separate client (with `wl-paste` as the reference) reads "restored".

**Acceptance Scenarios**:

1. **Given** a request to serve T, **When** the source is set, **Then** it offers `text/plain;charset=utf-8`, `text/plain`, `UTF8_STRING`, `STRING` and `TEXT`, and writes T to every `send` fd.
2. **Given** rstory's source is set, **When** the compositor sends `cancelled` (another owner), **Then** serving stops.
3. **Given** rstory's own source becomes the selection, **When** the change is observed, **Then** it's flagged `self_owned`.

### Edge Cases

- The compositor advertises an older data-control version: the backend binds the highest supported version, up to the one it knows.
- A reader client closes its pipe early: the write fails with `EPIPE`, which is ignored without crashing.
- The Wayland connection is lost (compositor exit): `Disconnected` is emitted, and the daemon exits because the session is over.
- A source that takes more than 2 s to write: the read is abandoned (`Timeout`), and nothing is recorded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The backend MUST implement the `Backend` interface from spec 002.
- **FR-002**: It MUST bind `ext_data_control_manager_v1` when it's offered, else `zwlr_data_control_manager_v1`, using the pure-Rust `wayland-client` backend.
- **FR-003**: It MUST read text through `receive(mime, fd)` with a non-blocking pipe registered in the event loop, preferring `text/plain;charset=utf-8` › `UTF8_STRING` › `text/plain` › `STRING` › `TEXT`.
- **FR-004**: Reads MUST enforce the caller's byte limit and a 2 s timeout.
- **FR-005**: Serving MUST create a data-control source with the types in US2-AS1 and answer `send` asynchronously, so a slow reader never blocks the loop.
- **FR-006**: The backend MUST NOT poll, and MUST NOT log content.

### Key Entities

- Same as spec 002 (Selection change, Backend selection).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Under headless sway, 100 copies produce 100 correct events, and event-to-text p95 is under 20 ms.
- **SC-002**: A 1 MiB value round-trips byte for byte through serve then read.
- **SC-003**: The capture and restore manual checks pass on KDE Plasma Wayland before each release.

## Assumptions

- wayland-protocols 0.32 exposes `ext-data-control-v1` under its `staging` feature (ADR-0006). If not, the XML is vendored with `wayland-scanner`.
