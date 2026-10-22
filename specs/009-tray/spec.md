# Feature Specification: SNI tray

**Feature Branch**: `feat/tray`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Tray & GNOME), J7, K7, BR16; ADR-0006 (ksni); Constitution VII.

> Tests are **required** for every story (Constitution II): menu-model unit tests, and an SNI integration test against a D-Bus session with a stub `StatusNotifierWatcher`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Restore from the top bar (Priority: P1)

On KDE or a bar with an SNI host, a tray icon shows a menu with the last 10 entries. Clicking one restores it.

**Why this priority**: J7, the secondary audience's main job.

**Independent Test**: Build the menu model from a history of 12 entries. It has 10 entry items (one-line previews, at most 48 chars) in order. Activating item 3 sends restore for the third entry.

**Acceptance Scenarios**:

1. **Given** an SNI host on the session bus, **When** the daemon starts, **Then** it registers a StatusNotifierItem whose icon is pixel data (normal state), with the tooltip "rstory — recording".
2. **Given** 12 entries, **When** the menu opens, **Then** it shows the newest 10, each labeled with its preview (escaped, truncated with `…`), plus a pin glyph for pinned ones.
3. **Given** the menu, **When** an entry item is activated, **Then** that entry is restored (same path as the protocol's `restore`).
4. **Given** the history changes, **When** the menu is next shown, **Then** it reflects the change (the menu layout's revision is bumped).

---

### User Story 2 - Pause, timed pause, open picker, clear (Priority: P1)

The menu has "Pause recording" / "Resume recording", "Pause for 1 minute", "Open picker" (only when a terminal is available), and "Clear history…" with a confirm step.

**Why this priority**: Owner decisions on the tray menu, and J6 and J6b.

**Independent Test**: Menu-model tests for each state: recording, paused, timed pause (label "Paused — resumes in 0:42"), and no terminal (item hidden). Clear goes through a confirm step before `clear` is sent.

**Acceptance Scenarios**:

1. **Given** recording, **When** "Pause recording" is activated, **Then** the state becomes paused, the icon switches to the paused variant (visibly different in shape, not only color), and the item becomes "Resume recording".
2. **Given** recording, **When** "Pause for 1 minute" is activated, **Then** a 60 s timed pause starts and the status item shows the remaining time.
3. **Given** `picker_available: false`, **When** the menu opens, **Then** "Open picker" isn't shown.
4. **Given** the menu, **When** "Clear history…" is activated, **Then** the menu is replaced by a confirm submenu ("Delete N unpinned entries" / "Cancel"). Only the confirm item sends `clear`.

---

### User Story 3 - No SNI host (Priority: P3)

Without an SNI host, the daemon runs without a tray and says so.

**Why this priority**: Honest failure (Constitution VIII), brief S5.

**Independent Test**: No watcher on the bus: the daemon starts, `status` has `tray: "no_host"`, and nothing crashes. If a watcher appears later, the item registers.

**Acceptance Scenarios**:

1. **Given** no `org.kde.StatusNotifierWatcher`, **When** the daemon starts, **Then** it continues without a tray and `status.tray = "no_host"`.
2. **Given** a watcher appears later, **When** its name is acquired, **Then** the tray registers within 1 s.

### Edge Cases

- No D-Bus session bus (`DBUS_SESSION_BUS_ADDRESS` unset and no socket): tray disabled, `status.tray = "no_bus"`.
- GNOME: the SNI item still registers when a watcher exists (for example an AppIndicator extension), but the panel icon comes from spec 010. If both exist, both work.
- Previews with newlines show them as `⏎` in menu labels.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tray MUST run inside the daemon process and call the daemon's internal API (no socket round trip).
- **FR-002**: Icons MUST be sent as ARGB32 pixel data at 16, 22, 24, 32 and 48 px for both states (normal and paused), with no icon theme lookup.
- **FR-003**: The menu MUST contain, in order: up to 10 entry items; a separator; the status label (disabled item); Pause/Resume; Pause for 1 minute; Open picker (conditional); a separator; Clear history….
- **FR-004**: Every menu item MUST carry a text label. The paused state MUST be conveyed by the icon's shape and by text.
- **FR-005**: The tray MUST follow `NameOwnerChanged` for the watcher and (re)register.
- **FR-006**: `status` MUST expose `tray: active | no_host | no_bus | disabled` (the config `tray = false` disables it).

### Key Entities

- **Tray menu model**: derived purely from (history top 10, daemon state, picker_available), and unit-testable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Manual release check: on KDE Plasma and on sway with waybar, the icon appears, restoring works, and pause changes the icon.
- **SC-002**: The menu model has 100% unit coverage of its states.

## Assumptions

- "Pause for 1 minute" is fixed at 60 s in the MVP (`tray_pause_duration` could come later).
