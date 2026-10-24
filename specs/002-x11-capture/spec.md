# Feature Specification: Session detection & X11 capture (incl. XWayland bridge)

**Feature Branch**: `feat/x11-capture`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Capture & backends), BR5b, spike S1; ADR-0005, ADR-0006.

> Tests are **required** for every story (Constitution II). The X11 backend is tested against Xvfb, and the bridge path against a headless compositor with XWayland.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture on X11 (Priority: P1)

On an X11 session, every clipboard change made by any application is delivered to rstory with its offered types and text.

**Why this priority**: X11 is Tier 1, and the same code is GNOME's path.

**Independent Test**: Under Xvfb, start the backend in watch mode. An X11 client sets `CLIPBOARD` to "hello". The backend emits one change event with targets including `UTF8_STRING`, and reads "hello".

**Acceptance Scenarios**:

1. **Given** a running X server, **When** a client takes `CLIPBOARD` ownership offering `UTF8_STRING`, **Then** the backend emits `SelectionChanged{types}` and can read the text as UTF-8.
2. **Given** an owner that offers only `STRING` (Latin-1), **When** the text is read, **Then** it's converted to UTF-8.
3. **Given** a 900 KiB selection (above the INCR threshold), **When** it's read, **Then** the full content arrives through INCR.
4. **Given** the owner exits without handing the selection over, **When** that happens, **Then** the backend emits `SelectionCleared` and nothing is recorded.

---

### User Story 2 - Restore on X11 (Priority: P1)

rstory can become the `CLIPBOARD` owner and serve an entry's text to any application that pastes.

**Why this priority**: Restoring is the second half of J1.

**Independent Test**: Under Xvfb, the backend serves "restored" as the owner. A separate client converts `CLIPBOARD` to `UTF8_STRING` and gets "restored". `TARGETS` lists the served types.

**Acceptance Scenarios**:

1. **Given** the backend is asked to serve text T, **When** it acquires ownership, **Then** it reports success only after `GetSelectionOwner` returns its window.
2. **Given** rstory owns the selection, **When** a client requests `TARGETS`, `UTF8_STRING`, `STRING`, `TEXT` or `text/plain;charset=utf-8`, **Then** each conversion succeeds. Large values use INCR.
3. **Given** rstory owns the selection, **When** another client takes ownership, **Then** the backend stops serving and emits `SelectionChanged` for the new owner (normal capture).
4. **Given** rstory owns the selection, **When** the backend's own ownership change is observed, **Then** it's flagged `self_owned`, so the daemon applies BR2 instead of recording a new entry.

---

### User Story 3 - Automatic backend selection, including the XWayland bridge (Priority: P1)

The session is detected at start, and the right backend is picked: the X11 backend on X11, and also on Wayland sessions without data-control when XWayland is available (GNOME).

**Why this priority**: G2 and the v0.1 GNOME gate (G7).

**Independent Test**: Run the selection logic against fake environments and fake registry results. Under headless mutter (no data-control, XWayland on demand), selection picks `x11` / `xwayland_bridge`, and capture works through Mutter's XWayland. Cross-protocol copies (Wayland app ↔ X11) are covered by the manual GNOME matrix, because headless mutter has a seat with no keyboard, so no Wayland client can own a selection there (corrected 2026-10-22 by `devcontainer:setup`).

**Acceptance Scenarios**:

1. **Given** `XDG_SESSION_TYPE=x11` and `DISPLAY` set, **When** rstory selects a backend, **Then** it picks `x11`.
2. **Given** a Wayland session whose registry offers `ext_data_control_manager_v1` or `zwlr_data_control_manager_v1`, **When** rstory selects, **Then** it picks `wayland` (spec 003).
3. **Given** a Wayland session with no data-control global but with `DISPLAY` set, **When** rstory selects, **Then** it picks `x11` with mode `xwayland_bridge`.
4. **Given** a Wayland session with no data-control and no `DISPLAY`, **When** rstory selects, **Then** the result is `none` with the reason `no_data_control_no_xwayland`.
5. **Given** `backend = "x11"` in the config or `--backend x11`, **When** rstory selects, **Then** the override wins, even when detection would choose differently.

### Edge Cases

- XWayland is started on demand by the compositor. The first connection may take up to 1 s, so selection waits up to 3 s before reporting failure.
- The X server connection drops (an XWayland restart). The backend reports `Disconnected`, and the daemon reconnects with backoff (spec 004).
- The owner refuses a conversion (sensitive-only offers, like the S1 observation). The read fails cleanly and is reported as `ConversionRefused`.
- Text with invalid UTF-8 is decoded lossily, and the replacement count is logged (never the content).
- `DISPLAY` is set but unreachable: selection reports `x11_unreachable`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: rstory MUST define a `Backend` interface with: start watching; a stream of events (`SelectionChanged{types, self_owned}`, `SelectionCleared`, `Disconnected`); `read_text(limit)`; `serve(text)`; and `stop_serving`. The interface MUST NOT expose display-server types to callers.
- **FR-002**: The X11 backend MUST use XFIXES `SelectSelectionInput` on `CLIPBOARD` (set-owner, window-destroy, client-close) and MUST NOT poll.
- **FR-003**: Reads MUST request `TARGETS` first, expose the target names as types, then convert the best text target in the order `UTF8_STRING` › `text/plain;charset=utf-8` › `STRING` › `TEXT`.
- **FR-004**: Reads MUST enforce a byte limit passed by the caller and abort the transfer once it's exceeded (reported as `TooLarge`).
- **FR-005**: The backend MUST support INCR for reading and for serving.
- **FR-006**: Serving MUST answer `TARGETS`, `TIMESTAMP`, `UTF8_STRING`, `STRING`, `TEXT` and `text/plain;charset=utf-8`, and refuse everything else.
- **FR-007**: Session detection MUST use `WAYLAND_DISPLAY`, `DISPLAY` and `XDG_SESSION_TYPE`, plus a Wayland registry probe for the data-control globals, and apply the rule in BR5b.
- **FR-008**: The selection result MUST carry `{backend, mode (native | xwayland_bridge), reason}` for `status` and `doctor`.
- **FR-009**: An override (`backend` config key or `--backend` flag on `rstory daemon`) MUST win over detection.
- **FR-010**: The backend MUST NOT log content. Types, sizes and timings only.

### Key Entities

- **Backend selection**: `{backend: wayland | x11 | none, mode, reason}`.
- **Selection change**: offered types plus the `self_owned` flag.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Under Xvfb, 100 consecutive copies produce 100 change events with correct text. Event-to-text p95 is under 20 ms.
- **SC-002**: A 1 MiB value round-trips (serve, then read by another client) byte for byte.
- **SC-003**: Selection logic chooses correctly in 100% of the table-driven environment cases (≥ 8 cases).
- **SC-004**: On GNOME Shell 48–51 (manual release check), copies from a GUI app and from `wl-copy` are captured, and restored values paste into a Wayland-native app.

## Assumptions

- XWayland mirrors the clipboard both ways regardless of focus (S1 on GNOME 48). The other versions are verified at release (brief Q7).
- Primary selection is out of scope (Later).
