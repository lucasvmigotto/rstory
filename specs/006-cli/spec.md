# Feature Specification: Command-line interface

**Feature Branch**: `feat/cli`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (CLI), J3, K2, K6, BR14; Constitution VI.

> Tests are **required** for every story (Constitution II). `--json` output is validated against `contracts/cli.md`'s schemas.

## Clarifications

### Session 2026-10-21

- Q: Should a bulk wipe of pinned entries exist? → A: Yes, `clear --all`, with its own confirmation prompt (or `--yes`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse and restore from the shell (Priority: P1)

The user lists the history, shows one entry, and restores one by index or short id.

**Why this priority**: J1 and J3 from a terminal or a script.

**Independent Test**: With a daemon on a fake backend, `rstory ls` prints entries newest first, `rstory show 1` prints the raw content, and `rstory copy 1` restores it.

**Acceptance Scenarios**:

1. **Given** entries B (0) and A (1), **When** `rstory ls` runs, **Then** it prints one line per entry: index, short id, a pin glyph if pinned, relative time, and a one-line preview truncated to the terminal width.
2. **Given** the same history, **When** `rstory ls --json` runs, **Then** it prints a JSON array matching the `entry_list` schema.
3. **Given** entry A has short id `ab12`, **When** `rstory show ab12` runs, **Then** stdout is exactly A's bytes, with no trailing newline added.
4. **Given** the daemon is running, **When** `rstory copy 1` runs, **Then** A is restored and the command exits 0 silently.
5. **Given** an unknown reference, **When** any command takes it, **Then** it exits 2 with "no entry <ref>". An ambiguous short id lists the candidates on stderr and exits 2.

---

### User Story 2 - Manage the history (Priority: P1)

The user pins, unpins, removes and clears entries, and pauses or resumes recording from the shell.

**Why this priority**: CLI parity (Constitution VI), J4, J6 and J6b.

**Independent Test**: Each command against a daemon with a fake backend. Assert the resulting history or state and the exit code.

**Acceptance Scenarios**:

1. **Given** an unpinned entry, **When** `rstory pin 0` runs, **Then** it's pinned. When it's already at the pin cap, the command exits 6 with "pin limit reached (10)".
2. **Given** pinned and unpinned entries, **When** `rstory clear` runs in a terminal, **Then** it asks "Delete N unpinned entries? [y/N]". `--yes` skips the prompt, and without a TTY `--yes` is required (otherwise exit 2). `rstory clear --all` asks "Delete ALL N entries, including M pinned? [y/N]".
3. **Given** recording, **When** `rstory pause --for 50s` runs, **Then** it exits 0 and `status` shows the end time. `rstory resume` ends it.
4. **Given** no daemon, **When** `rstory pause` runs, **Then** it exits 3 with "daemon not running: start it with `rstory daemon` or `rstory install-autostart`", so `passc` stops before `pass` copies.

---

### User Story 3 - Read the history while the daemon is down (Priority: P2)

`ls` and `show` work without the daemon by reading the store read-only.

**Why this priority**: Decision 2026-10-21 (BR14).

**Independent Test**: With no daemon and a prepared store, `ls` and `show` work and `copy` exits 3.

**Acceptance Scenarios**:

1. **Given** no daemon and a store with entries, **When** `rstory ls` runs, **Then** it lists them and prints "(daemon not running, read-only)" on stderr.
2. **Given** no daemon, **When** `rstory copy 0` runs, **Then** it exits 3 with the hint.

---

### User Story 4 - Status and configuration (Priority: P2)

`rstory status` reports the daemon state and counters. `rstory config` shows the effective configuration and its path.

**Why this priority**: Diagnosability and scriptability.

**Independent Test**: `status` and `status --json` against a daemon. `config` with and without a config file.

**Acceptance Scenarios**:

1. **Given** a running daemon, **When** `rstory status --json` runs, **Then** it prints the `status` schema.
2. **Given** no config file, **When** `rstory config` runs, **Then** it prints the effective defaults and the path where a file would be read. `rstory config --path` prints only the path.

### Edge Cases

- `NO_COLOR` set, or stdout not a TTY: no colors, no relative-time padding, and glyphs still shown.
- A preview containing control characters prints them escaped (`\n`, `\t`, `\x1b`).
- A narrow terminal (< 40 cols): previews are truncated, and index and short id are always shown.
- `show` of a non-UTF-8-safe entry writes the raw bytes as stored.
- The read-only fallback (`ls`, `show`) opens the store, reads what it needs, and drops the handle before printing, so its shared lock never blocks a daemon starting (spec 004).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST provide `ls`, `show <ref>`, `copy <ref>`, `pin <ref>`, `unpin <ref>`, `rm <ref>`, `clear [--all] [--yes]`, `pause [--for <duration>]`, `resume`, `status`, `config [--path]`, plus the subcommands owned by other specs (`daemon` 004, `pick` 007, `doctor` 011, `install-autostart` 012, `install-gnome-extension` 010).
- **FR-002**: A `<ref>` MUST accept an index (a non-negative integer) or a short id (≥ 4 chars, a unique prefix).
- **FR-003**: `ls`, `show` and `status` MUST support `--json`, with schemas in `contracts/cli.md`. Each JSON document carries `"schema": 1`.
- **FR-004**: Durations MUST accept `<n>s|m|h` (for example `50s`, `2m`, `1h`).
- **FR-005**: Exit codes MUST be: 0 ok; 1 unexpected error; 2 usage or not found; 3 daemon not running; 4 already running (daemon); 5 environment error; 6 limit reached; 7 inactive (no capture backend).
- **FR-006**: Write commands MUST go through the socket. `ls` and `show` MUST fall back to a read-only store open when the socket is absent.
- **FR-007**: Errors MUST go to stderr, and machine output to stdout.
- **FR-008**: The CLI MUST offer `--help` for every command, and shell completions through `rstory completions <bash|zsh|fish>`.
- **FR-009**: A parity table in `contracts/cli.md` MUST map every tray, picker and extension action to a CLI command.

### Key Entities

- **Entry reference**: index or short id.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `rstory ls` completes in under 50 ms with 500 entries (via the socket and read-only).
- **SC-002**: 100% of `--json` outputs validate against their schemas in the tests.
- **SC-003**: Every exit code in FR-005 is covered by at least one test.

## Assumptions

- `rm` doesn't prompt (single entry), and `clear` does (bulk).
