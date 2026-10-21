# Feature Specification: Picker launch (terminal detection)

**Feature Branch**: `feat/picker-launch`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Open picker), S8, Q2 (resolved); ADR-0008 (the daemon spawns the picker for the extension).

> Tests are **required** for every story (Constitution II).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open the picker from a menu (Priority: P1)

Choosing "Open picker" in the tray or the GNOME panel menu opens `rstory pick` in a terminal emulator.

**Why this priority**: It's the tray's and panel's path to the picker (J7). Specs 009 and 010 depend on it.

**Independent Test**: With a fake `PATH` containing only `kitty`, the `open_picker` request spawns `kitty -e rstory pick` (detached) and returns `ok`.

**Acceptance Scenarios**:

1. **Given** `picker_command = "foot -e rstory pick"` in the config, **When** `open_picker` is requested, **Then** that command is spawned as is (shell-word split, no shell).
2. **Given** no `picker_command`, `$TERMINAL=alacritty` and alacritty on `PATH`, **When** `open_picker` is requested, **Then** `alacritty -e rstory pick` is spawned.
3. **Given** no `picker_command`, no `$TERMINAL` and `xdg-terminal-exec` on `PATH`, **When** requested, **Then** `xdg-terminal-exec rstory pick` is spawned.
4. **Given** none of the above, **When** requested, **Then** the first terminal on `PATH` from the list `ghostty, kitty, alacritty, foot, wezterm, ptyxis, kgx, gnome-terminal, konsole, xfce4-terminal, xterm` is used with its known exec syntax.
5. **Given** no terminal found, **When** `open_picker` is requested, **Then** the error is `no_terminal`, and `status` reports `picker_available: false` so menus hide the item.

### Edge Cases

- A spawned terminal must not inherit the daemon's stdio or signal handlers, and it gets the daemon's environment (`DISPLAY`, `WAYLAND_DISPLAY`).
- `picker_command` pointing at a missing binary: error `picker_command_failed` with the program name.
- Exec syntax per terminal: `ghostty -e`, `kitty`, `alacritty -e`, `foot`, `wezterm start --`, `ptyxis --`, `kgx --`, `gnome-terminal --`, `konsole -e`, `xfce4-terminal -x`, `xterm -e`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The launcher MUST resolve, in order: `picker_command` (config) › `$TERMINAL` › `xdg-terminal-exec` › the fixed list (FR-003).
- **FR-002**: It MUST spawn detached (a new session, stdio set to null) without a shell.
- **FR-003**: The fixed list and its exec syntax MUST live in one table, with a test per row.
- **FR-004**: The daemon MUST expose `open_picker` over the protocol and `picker_available` in `status`.
- **FR-005**: The resolution result (found command or searched list) MUST be available to `doctor` (spec 011).

### Key Entities

- **Picker launch plan**: `{source: config | env | xdg | list, argv}` or `{searched: [names]}`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every row of the terminal table has a test that asserts the argv.
- **SC-002**: On the owner's machine (ghostty installed), "Open picker" opens a ghostty window running the picker.

## Assumptions

- The picker's own window title is set to `rstory` (an OSC escape) so window rules can float it.
