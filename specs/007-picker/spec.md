# Feature Specification: Terminal picker

**Feature Branch**: `feat/picker`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Picker), J1, K2, §11 Accessibility; ADR-0007; Constitution V, VII.

> Tests are **required** for every story (Constitution II): snapshot tests per state and size, keystroke journeys, and a first-frame benchmark (F6).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find and restore a recent copy (Priority: P1)

`rstory pick`, bound to a key, shows the history newest first. Typing narrows it with fuzzy matching, a preview shows the selected entry, and Enter restores it and exits.

**Why this priority**: This is K2, the primary journey.

**Independent Test**: Keystroke test against a fake daemon client with entries B and A: type "reb", press Enter, and assert `restore(A)` was sent and the picker exited 0.

**Acceptance Scenarios**:

1. **Given** entries B, A, **When** the picker opens, **Then** the first frame lists B, A with relative times and B selected, within 100 ms (p95).
2. **Given** the list, **When** the user types `reb`, **Then** only matching entries remain, ranked by match score then recency, with matched characters marked (bold or underline, not color alone).
3. **Given** a selected entry, **When** the preview pane is visible (width ≥ 80), **Then** it shows the full content, wrapped, with control characters escaped. On narrower terminals the preview toggles with `Tab`.
4. **Given** a selected entry, **When** the user presses Enter, **Then** the picker sends `restore`, waits for `ok`, restores the terminal, and exits 0.
5. **Given** the picker is open, **When** the user presses Esc or Ctrl-C, **Then** it exits 130 (Ctrl-C) or 1 (Esc) without touching the clipboard.

---

### User Story 2 - Manage entries from the picker (Priority: P2)

From the picker the user pins or unpins and deletes the selected entry.

**Why this priority**: Parity with the CLI and the tray (Constitution VI).

**Independent Test**: Keystroke tests that assert the `pin`, `unpin` and `rm` requests and the refreshed list.

**Acceptance Scenarios**:

1. **Given** a selected unpinned entry, **When** the user presses `Ctrl-T`, **Then** it's pinned (or unpinned if it was pinned) and shows the pin glyph. At the cap, a status line shows "pin limit reached (10)".
2. **Given** a selected entry, **When** the user presses `Ctrl-D`, **Then** it asks "Delete? y/n" in the status line, and `y` removes it.

---

### User Story 3 - Works without color, without a daemon, and with screen readers (Priority: P2)

The picker is usable in plain mode, under `NO_COLOR`, and read-only when the daemon is down.

**Why this priority**: Constitution VII and BR14.

**Independent Test**: Snapshot tests in `--plain` and `NO_COLOR` modes. With no daemon, Enter shows the hint and exits 3.

**Acceptance Scenarios**:

1. **Given** `--plain`, **When** the picker opens, **Then** it renders inline (no alternate screen, no box drawing), marking the selected line with `>` and pinned entries with `[pinned]`.
2. **Given** no daemon, **When** the picker opens, **Then** it lists entries read-only with a banner. Enter shows "daemon not running: …" and exits 3 without touching the clipboard.
3. **Given** an empty history, **When** the picker opens, **Then** it shows "No entries yet".

### Edge Cases

- Terminal resize: re-layout without crashing. The minimum is 20×5, and smaller shows "terminal too small".
- A panic or signal always restores the terminal (raw mode off, cursor shown).
- History changes while the picker is open (a `subscribe` event): the list refreshes and the selection stays on the same id when it's still present.
- Very long single-line entries: the list preview is truncated with `…`, and the preview pane wraps.
- stdout not a TTY: exit 2 with "rstory pick needs a terminal".
- In read-only mode the picker takes a snapshot (entries plus the content it previews) and drops the store handle immediately. A preview of another entry reopens the store briefly. The picker never holds the shared lock while waiting for input, so a daemon can start (spec 004).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The picker MUST open in under 100 ms p95 to the first frame with 20 entries, and with 500 configured.
- **FR-002**: It MUST use fuzzy matching (case-insensitive by default, smart-case when the query has uppercase).
- **FR-003**: Keymap: Up/Down and Ctrl-P/Ctrl-N move the selection; PgUp/PgDn and Home/End jump; Enter restores; Tab toggles the preview; Ctrl-T toggles the pin; Ctrl-D deletes (with confirm); Esc and Ctrl-C quit; typing edits the query (Backspace, Ctrl-U clears, Ctrl-W deletes a word).
- **FR-004**: The pin key MUST be `Ctrl-T`, not `Ctrl-P`, which is "previous" in readline convention.
- **FR-005**: State MUST be distinguishable without color: selected (`>` plus reverse video), pinned (`📌`, or `[p]` in plain mode), matched characters (bold or underline).
- **FR-006**: `--plain` MUST render inline without box drawing. `NO_COLOR` MUST disable colors.
- **FR-007**: The picker MUST use the protocol client (spec 004) and fall back to a read-only store open.
- **FR-008**: The terminal MUST be restored on exit, error, panic and SIGTERM/SIGINT/SIGHUP.
- **FR-009**: Exit codes: 0 restored; 1 dismissed (Esc); 130 interrupted (Ctrl-C); 2 no TTY; 3 daemon not running at restore.

### Key Entities

- **Picker state**: query, filtered list, selection (by id), preview visibility, mode (normal, confirm delete), read-only flag.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: First frame p95 under 100 ms (F6) with 20 and 500 entries.
- **SC-002**: A keystroke-to-redraw p95 under 16 ms with 500 entries.
- **SC-003**: Snapshot coverage: every state (normal, filtered, empty, read-only, confirm, too small, plain) at 80×24 and 40×10.

## Assumptions

- The pin key is `Ctrl-T` (FR-004), to stay compatible with readline navigation.
