# Feature Specification: GNOME Shell extension

**Feature Branch**: `feat/gnome-extension`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Tray & GNOME), G7, K8, Q3 (48–51), Q6 (control socket); ADR-0008.

> Tests are **required** for every story (Constitution II): install tests in Rust, a protocol-fixture test of the JS client under `gjs` when available, and the manual matrix on Shell 48–51.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install the extension from the binary (Priority: P1)

`rstory install-gnome-extension` writes the embedded extension into the user's extensions directory and tells the user how to enable it.

**Why this priority**: The v0.1 gate on the owner's GNOME 48 needs a panel icon, with no AppIndicator.

**Independent Test**: Run with `XDG_DATA_HOME` set to a temp dir. The files appear under `gnome-shell/extensions/rstory@lucasvmigotto.github.io/`, `metadata.json` lists shell-version 48–51 and the rstory version, and the output explains the re-login.

**Acceptance Scenarios**:

1. **Given** no extension installed, **When** `rstory install-gnome-extension` runs, **Then** it writes `metadata.json`, `extension.js`, `stylesheet.css` and the icons, and prints "Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.github.io".
2. **Given** `--enable`, **When** `gnome-extensions` is on `PATH`, **Then** it also runs `gnome-extensions enable …` and reports the result.
3. **Given** an older version installed, **When** the command runs, **Then** the files are replaced and the output says a re-login is needed to load the new version.
4. **Given** `--uninstall`, **When** it runs, **Then** the directory is removed (after `gnome-extensions disable` when available).

---

### User Story 2 - Panel icon and menu on GNOME (Priority: P1)

Once enabled, the extension shows an rstory icon in the top bar with the same menu as the SNI tray, driven entirely by the daemon over the control socket.

**Why this priority**: J7 on GNOME, and G7.

**Independent Test**: The JS client replays the protocol fixtures (`status`, `list`, `subscribe` events) and checks the menu model it builds. On a real Shell, the manual matrix runs on 48–51.

**Acceptance Scenarios**:

1. **Given** the extension is enabled and the daemon is running, **When** the user clicks the icon, **Then** the menu shows the last 10 entries, the status label, Pause/Resume, Pause for 1 minute, Open picker (if `picker_available`), and Clear history… (with a confirm).
2. **Given** an entry item, **When** it's clicked, **Then** the extension sends `restore{id}`.
3. **Given** "Open picker", **When** it's clicked, **Then** the extension sends `open_picker` and never spawns processes itself.
4. **Given** the daemon is paused, **When** `pause_changed` arrives, **Then** the icon switches to the paused variant (a different shape) and the label updates.
5. **Given** the daemon isn't running, **When** the extension loads, **Then** the icon shows a "disconnected" variant, the menu says "rstory daemon not running", and it reconnects with backoff (1 s → 30 s).

### Edge Cases

- A Shell version outside 48–51: GNOME refuses to load the extension because of `metadata.json`, and `doctor` explains.
- The socket path comes from `GLib.get_user_runtime_dir()` + `/rstory/control.sock`.
- Extension disable must close the socket and remove every signal handler and timeout (GNOME review rules).
- The installed extension version differs from the binary's: `doctor` reports "extension outdated: run rstory install-gnome-extension".

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The extension source MUST live in `extension/` in the repository and be embedded in the binary with `include_dir`.
- **FR-002**: `metadata.json` MUST declare `uuid: rstory@lucasvmigotto.github.io`, `shell-version: ["48","49","50","51"]` and `version-name` equal to the crate version.
- **FR-003**: The extension MUST be an ESM extension (`export default class … extends Extension`) using `PanelMenu.Button` and `PopupMenu`.
- **FR-004**: It MUST talk only through the control socket (protocol v1), using `Gio.SocketClient` + `Gio.UnixSocketAddress` and line-based reads. No D-Bus, no subprocess.
- **FR-005**: It MUST contain no history logic: everything it shows comes from `list`, `status` and `subscribe`.
- **FR-006**: The menu MUST mirror spec 009 FR-003 (the same items, order and labels).
- **FR-007**: `install-gnome-extension` MUST support `--enable` and `--uninstall`, and write to `$XDG_DATA_HOME/gnome-shell/extensions/<uuid>/` (default `~/.local/share`).
- **FR-008**: The extension MUST clean up fully in `disable()`.

### Key Entities

- **Extension install**: path, installed version, enabled state (from `gnome-extensions info` when available).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Manual matrix: on GNOME Shell 48, 49, 50 and 51 (Wayland), the icon appears after re-login and enable, restoring works, and pause changes the icon.
- **SC-002**: The JS client passes all shared protocol fixtures.
- **SC-003**: Install, reinstall and uninstall tests pass in CI.

## Assumptions

- UUID `rstory@lucasvmigotto.github.io` (architecture §13).
