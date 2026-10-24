# Feature Specification: Doctor

**Feature Branch**: `feat/doctor`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief J9, K1, K4, K8, §14 (seven failure modes); Constitution VIII.

> Tests are **required** for every story (Constitution II): one test per detectable failure, with faked environments.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See what works and what's missing (Priority: P1)

`rstory doctor` prints a short report covering the session, backend, store, daemon, tray and extension, and the picker terminal, in plain words. Each problem comes with a next step.

**Why this priority**: J9, the operator's main job, and the GNOME setup journey K8.

**Independent Test**: Run against a faked environment (all OK). Each line reports ✓, and the exit code is 0.

**Acceptance Scenarios**:

1. **Given** a healthy GNOME 48 setup, **When** `rstory doctor` runs, **Then** it prints lines like "✓ Session: Wayland (GNOME Shell 48.7)", "✓ Backend: x11 via XWayland bridge (no data-control on this compositor)", "✓ Store: private (0700/0600)", "✓ Daemon: running, recording", "✓ GNOME extension: installed 0.1.0, enabled", "✓ Picker terminal: ghostty".
2. **Given** `--json`, **When** it runs, **Then** it prints one object per check with `id`, `status` (`ok|warn|fail`), `message` and `hint`.

---

### User Story 2 - Every detectable failure is explained (Priority: P1)

Each known failure has a specific message and hint.

**Why this priority**: Brief §14 metric (7/7), and Constitution VIII.

**Independent Test**: A table-driven test with one faked environment per failure, asserting the status, message and hint.

**Acceptance Scenarios** (each a row of the test table):

1. No backend possible (Wayland, no data-control, no XWayland): fail, "This compositor offers no clipboard-monitoring protocol and no XWayland…", with the hint "enable XWayland".
2. The daemon isn't running: fail, with the hint "run `rstory daemon` or `rstory install-autostart`".
3. The store is too open: fail, with the hint "chmod 700 <dir> && chmod 600 <dir>/*".
4. The GNOME extension is missing, disabled or outdated (on GNOME): warn, with the matching command.
5. The Shell version is outside the extension's declared range (48–50, `extension/metadata.json`): warn, "panel icon not supported on GNOME Shell N; CLI and picker work".
6. No SNI host (outside GNOME): warn, "no StatusNotifierItem host: the tray is unavailable; CLI and picker work".
7. No terminal for the picker: warn, listing what was searched, with the hint "set picker_command in config.toml".
8. Pause active: info, "recording paused (resumes in …)".

### Edge Cases

- Doctor must never print entry content (Constitution III).
- Doctor must work with no daemon (read-only checks) and with no session at all (it reports "no graphical session").
- Exit code: 0 when everything is ok or info, 1 when any warn, 2 when any fail.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Doctor MUST run these checks: session, backend selection (including mode and reason), data-control availability, XWayland presence, store permissions, daemon state (via `status`), counters, tray host, GNOME extension (installed, enabled, version), Shell version, picker launch plan, autostart presence.
- **FR-002**: Each check MUST produce `{id, status, message, hint?}`, with text and `--json` renderers.
- **FR-003**: Doctor MUST be read-only: it never modifies files, the store or the daemon state.
- **FR-004**: Messages MUST be plain English with no jargon beyond the glossary terms. Every non-ok result has a hint.

### Key Entities

- **Doctor check result**: `{id, status: ok | info | warn | fail, message, hint}`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 8 failure rows have passing tests (brief §14 target 7/7, plus the pause info).
- **SC-002**: Doctor completes in under 1 s (with the daemon running).

## Notes (docs-gaps, 2026-10-24)

- A `backend` set in `config.toml` is reported by the `backend` check: `x11 (chosen with backend in config.toml)`.
- A `config.toml` value rstory rejects (`theme`, `backend`, the caps) adds a `config` check with status `fail`, a message naming the key and a hint. The check appears only then, so a healthy report keeps its ten checks.
- `rstory doctor --ascii` (or `--ascii` before the command) limits the report to ASCII glyphs. The report's colors follow the `theme` key.
