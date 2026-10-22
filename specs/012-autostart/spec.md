# Feature Specification: Autostart

**Feature Branch**: `feat/autostart`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief J8, K1, G3; architecture §6.

> Tests are **required** for every story (Constitution II).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start at every login, any init system (Priority: P1)

`rstory install-autostart` writes an XDG autostart entry so the daemon starts at login under any desktop, with or without systemd.

**Why this priority**: J8, the dotfiles maintainer's main job.

**Independent Test**: With `XDG_CONFIG_HOME` set to a temp dir, the command writes `autostart/rstory.desktop` with `Exec=<absolute path to rstory> daemon`. Running it twice is idempotent.

**Acceptance Scenarios**:

1. **Given** no entry, **When** `rstory install-autostart` runs, **Then** `$XDG_CONFIG_HOME/autostart/rstory.desktop` exists with `Type=Application`, `Name=rstory`, `Exec=<abs path> daemon`, `X-GNOME-Autostart-enabled=true` and `NoDisplay=true`.
2. **Given** an existing entry, **When** it runs again, **Then** the file is rewritten identically and the exit code is 0.
3. **Given** `--uninstall`, **When** it runs, **Then** the entry (and the systemd unit, if any) is removed.

---

### User Story 2 - systemd user unit (Priority: P2)

With `--systemd`, a user unit is written instead, with restart on failure.

**Why this priority**: Reliability (restart) where systemd is available.

**Independent Test**: The unit file content matches the expected template. With a stubbed `systemctl`, `--now` calls `systemctl --user enable --now rstory.service`.

**Acceptance Scenarios**:

1. **Given** `--systemd`, **When** it runs, **Then** `$XDG_CONFIG_HOME/systemd/user/rstory.service` is written with `ExecStart=<abs path> daemon`, `Restart=on-failure`, `RestartSec=1`, `PartOf=graphical-session.target`, `After=graphical-session.target` and `WantedBy=graphical-session.target`, and no XDG entry is left behind.
2. **Given** `--systemd` and no `systemctl --user` available, **When** it runs, **Then** it exits 5 with "systemd user instance not available, use the default XDG autostart".

### Edge Cases

- The binary was run from a temporary path (for example `./target/debug`): warn that the autostart will point at that path.
- `Exec` paths with spaces are quoted per the Desktop Entry spec.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The default MUST write an XDG autostart entry. `--systemd` MUST write a user unit instead (and remove the XDG entry).
- **FR-002**: `--now` MUST also start the daemon (`systemctl --user enable --now`, or a detached `rstory daemon` for XDG).
- **FR-003**: `--uninstall` MUST remove both kinds.
- **FR-004**: Writing MUST be atomic (temp + rename) and never need root.
- **FR-005**: `doctor` MUST be able to read which autostart is installed (spec 011).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After a re-login on GNOME, KDE and sway (manual release check), the daemon is running.
- **SC-002**: The file-content tests match the templates byte for byte.
