# rstory — documentation site discovery

Owner stage: `project:docs` (Phase 1, Discovery). Next stage: `project:docs` Phase 2 (Scaffold), after the owner answers the open questions in §8.

This document is the evidence base for the docs site at `docs/site/`. Every claim the site makes must trace to a row here. It changes no code, scaffolds nothing, and adds no `package.json`.

- **Code ref.** Evidence is `file:line` at commit `8dd3566` (this branch, cut from `dev`). Rows marked **`dev@b4a998e`** cite `dev` after PR #79 (the extension declares GNOME Shell 48–50, and `make ext-pack`). That PR isn't on this branch, but the site will be built from `dev`/`main`, so its values win.
- **Statuses.** Implemented · Partially Implemented · Planned · Unavailable (the docs or specs mention it, the code doesn't do it) · Upstream Limitation · N/A.
- **Sources.** Statuses come from the code and its tests, never from `specs/README.md`. Where the two disagree, the row says so.
- **Release state.** `Cargo.toml:3` is `0.1.0`, there are no git tags, and nothing has been released (`README.md:5`). Every install channel is **Planned**.

Placement (Phase 0): rstory is a host application, so the site goes in `docs/site/`, which doesn't exist yet. `docs/product/`, `docs/release.md`, `contracts/` and `specs/` stay where they are and become content sources.

---

## 1. Truth table

### 1.1 Summary

| Status | Features | User stories | Surfaces | Total |
|---|---:|---:|---:|---:|
| Implemented | 12 | 34 | 134 | **180** |
| Partially Implemented | 2 | 2 | 1 | **5** |
| Planned | 0 | 1 | 15 | **16** |
| Unavailable | 0 | 0 | 1 | **1** (was 6; 5 implemented by feat/docs-gaps) |
| Upstream Limitation | 0 | 0 | 2 | **2** |
| N/A | 0 | 0 | 2 | **2** |
| **Rows** | 14 | 37 | 160 | **211** |

Where the code and `specs/README.md` disagree:

- **000 design-system:** listed as Implemented, but the code makes it Partially Implemented. The `theme` config key is never read, and both callers hard-code `ThemePref::Terminal` (`src/cli/render.rs:47`, `src/picker/mod.rs:149`). `detect_tier` returns the 16-color tier for `Terminal` before it looks at `COLORTERM` or `TERM` (`src/ui/theme.rs:114-120`). So the 256-color and truecolor tiers in `specs/000-design-system/tokens.md` can't be reached at runtime.
- **013 release:** listed as Implemented, but the code makes it Partially Implemented. The workflows exist and the CI part runs. The release path has never run: there's no tag and no release.
- **010 gnome-extension:** the brief and the dev@8dd3566 copy say GNOME Shell 48–51. `dev@b4a998e` declares 48–50, so 51 is **Unavailable** until its manual matrix passes.

### 1.2 Features (14)

| # | Feature | Status | Evidence (code and tests) |
|---|---|---|---|
| 000 | design-system | Partially Implemented | Roles and tiers `src/ui/theme.rs:8-15,111-130`. Glyph ASCII fallback `src/ui/glyphs.rs:66`. Keyed copy `src/ui/copy.rs:3-44`. Widgets `src/ui/widgets/`. Snapshots `src/ui/snapshots/`. `theme` is not wired (see §1.1). |
| 001 | store | Implemented | `src/store/mod.rs:26-28`, `src/store/index.rs:16-18`, `src/store/perms.rs:10-12`. Tests `tests/store_open.rs:47-177`, `tests/store_crash.rs:154`, `tests/store_readonly.rs:57-171`, `tests/fitness/perms.rs:46`. |
| 002 | x11-capture | Implemented | `src/backend/x11/mod.rs`, `src/backend/x11/incr.rs:1`, `src/session/mod.rs:133-175`. Tests `tests/x11_capture.rs:40-168`, `tests/x11_serve.rs:45-209`, `tests/bridge_capture.rs:30-84`. |
| 003 | wayland-capture | Implemented | `src/backend/wayland/mod.rs:1,48,251-256`. Tests `tests/wayland_capture.rs:69-133`, `tests/wayland_serve.rs:48-168`, `tests/wayland_probe.rs:20-34`. |
| 004 | daemon | Implemented | `src/main.rs:6-19`, `src/daemon/start.rs:29-52,387-440`, `src/ipc/protocol.rs`. Tests `tests/daemon_socket.rs:111-159`, `tests/daemon_status.rs:26-306`, `tests/daemon_pause.rs:18-168`. |
| 005 | privacy-filters | Implemented | `src/domain/filter.rs:13,74-78`. Tests `tests/daemon_privacy.rs:32-79`, `tests/filter_props.rs:58`, `tests/fitness/canary.rs:37`. |
| 006 | cli | Implemented | `src/cli/args.rs:17-190`, `src/cli/mod.rs:21-147`, `src/cli/commands.rs:53-242`. Tests `tests/cli_*.rs`, `tests/contract/cli_json.rs:46`. |
| 007 | picker | Implemented | `src/picker/mod.rs:33-90`, `src/picker/state.rs:523-620`. Tests `tests/picker_cli.rs:20-41`, `tests/picker_data.rs:27-136`, `tests/e2e/E2E-K2.rs:59-69`. |
| 008 | picker-launch | Implemented | `src/launch/mod.rs:12-31,72-78`. Tests `tests/daemon_status.rs:194-245`, `tests/e2e/launch_table.rs:14`. |
| 009 | tray | Implemented | `src/tray/menu.rs:10,220-250`, `src/tray/icons.rs:31`, `src/tray/client.rs:144-175`. Tests `tests/tray_dbus.rs:398-636`, `tests/e2e/E2E-K7.rs:192`. |
| 010 | gnome-extension | Implemented | `src/install/gnome.rs:17-31,112,322`, `extension/*.js`. Shell 48–50 per `dev@b4a998e`. Tests `tests/install_gnome.rs:110-270`, `extension/tests/*.test.js`. |
| 011 | doctor | Implemented | `src/doctor/mod.rs:17-48,164-170`, `src/doctor/checks.rs:90-101`. Tests `tests/doctor_cli.rs:42-153`, `src/doctor/tests.rs`. |
| 012 | autostart | Implemented | `src/install/autostart.rs:13-40,97-113,344-375`. Tests `tests/install_autostart.rs:87-309`, `tests/e2e/E2E-K1.rs:8`. |
| 013 | release | Partially Implemented | `.github/workflows/ci.yml`, `.github/workflows/release.yml:8-16,48-76,105-240`, `.github/workflows/nightly.yml`, `docs/release.md`. Never executed: `git tag` is empty. |

### 1.3 User stories (37)

| Spec · US | Story | Status | Evidence |
|---|---|---|---|
| 000 US1 | Consistent, readable surfaces in any terminal | Partially Implemented | Mono and 16-color tiers plus the ASCII fallback work (`src/ui/theme.rs:114-120`, `src/ui/glyphs.rs:66`). The 256-color and truecolor tiers can't be reached (§1.1). |
| 000 US2 | Recognisable icons in every state | Implemented | `assets/icons/*-symbolic.svg`, `build.rs:16-37`. Tests `src/tray/icons.rs:82`, `tests/install_gnome.rs:182`. |
| 001 US1 | Recorded entries survive restarts | Implemented | `tests/store_open.rs:116`, `tests/store_crash.rs:154,165` |
| 001 US2 | The history stays small and fresh | Implemented | `tests/history_props.rs:93`, `tests/store_h8.rs:38-78`, `tests/daemon_status.rs:73` |
| 001 US3 | Clients read while the daemon is down | Implemented | `tests/store_readonly.rs:57-171`, `tests/cli_readonly.rs:28-160` |
| 002 US1 | Capture on X11 | Implemented | `tests/x11_capture.rs:40-168`, `tests/daemon_x11.rs:30` |
| 002 US2 | Restore on X11 | Implemented | `tests/x11_serve.rs:45-209` |
| 002 US3 | Automatic backend selection, including the XWayland bridge | Implemented | `src/session/mod.rs:133-175`. Tests `tests/bridge_capture.rs:30,84`, `tests/wayland_probe.rs:27`. The override is the flag only (see the config table). |
| 003 US1 | Capture on data-control compositors | Implemented | `tests/wayland_capture.rs:69-133`, `tests/daemon_wayland.rs:34` |
| 003 US2 | Restore on data-control compositors | Implemented | `tests/wayland_serve.rs:48-168` |
| 004 US1 | Record the clipboard in the background | Implemented | `tests/daemon_capture.rs:18-244`, `tests/daemon_socket.rs:111-159` |
| 004 US2 | Serve a restore | Implemented | `tests/daemon_restore.rs:22-108` |
| 004 US3 | Pause and timed pause | Implemented | `tests/daemon_pause.rs:18-168`, `tests/e2e/E2E-K3b.rs:11` |
| 004 US4 | Status and housekeeping | Implemented | `tests/daemon_status.rs:26-133` |
| 005 US1 | Copies marked sensitive are never stored | Implemented | `src/domain/filter.rs:13,78`. Tests `tests/daemon_privacy.rs:32`, `tests/e2e/E2E-K3.rs:38-48`. |
| 005 US2 | Oversize copies are dropped, never truncated | Implemented | `tests/daemon_privacy.rs:60`, `tests/daemon_capture.rs:69` |
| 005 US3 | Nothing is recorded while paused | Implemented | `tests/daemon_privacy.rs:79`, `tests/daemon_pause.rs:18` |
| 006 US1 | Browse and restore from the shell | Implemented | `tests/cli_browse.rs:46-213` |
| 006 US2 | Manage the history | Implemented | `tests/cli_manage.rs:27-172` |
| 006 US3 | Read the history while the daemon is down | Implemented | `tests/cli_readonly.rs:28-160`, `tests/e2e/E2E-K6.rs:73` |
| 006 US4 | Status and configuration | Implemented | `tests/cli_status.rs:9-150`. `config` prints the caps and `picker_command`, never `tray` (`src/cli/commands.rs:138-142`). |
| 007 US1 | Find and restore a recent copy | Implemented | `src/picker/state.rs:552-553,606-620`. Test `tests/e2e/E2E-K2.rs:59,64`. |
| 007 US2 | Manage entries from the picker | Implemented | `src/picker/state.rs:554-568` (Ctrl-T, Ctrl-D). Test `tests/picker_data.rs:27,60`. |
| 007 US3 | Works without color, without a daemon, and with screen readers | Implemented | `--plain` `src/picker/mod.rs:50,153`, `src/picker/view.rs:237-239`. Read-only `tests/e2e/E2E-K2.rs:69`. |
| 008 US1 | Open the picker from a menu | Implemented | `tests/daemon_status.rs:194-245`, `tests/tray_dbus.rs:535`, `extension/tests/menu.test.js:103` |
| 009 US1 | Restore from the top bar | Implemented | `tests/tray_dbus.rs:422`, `tests/e2e/E2E-K7.rs:192` |
| 009 US2 | Pause, timed pause, open picker, clear | Implemented | `tests/tray_dbus.rs:462-546` |
| 009 US3 | No SNI host | Implemented | `tests/tray_dbus.rs:592,622,636` |
| 010 US1 | Install the extension from the binary | Implemented | `tests/install_gnome.rs:122-270` |
| 010 US2 | Panel icon and menu on GNOME | Implemented | `extension/tests/panel.test.js:39-204`, `extension/tests/menu.test.js:34-130`. Automated against a fake Shell. The real Shell is a manual matrix (`docs/release.md`). |
| 011 US1 | See what works and what's missing | Implemented | `src/doctor/checks.rs:90-101`, `tests/doctor_cli.rs:42-101` |
| 011 US2 | Every detectable failure is explained | Implemented | `tests/doctor_cli.rs:137,153`, `src/doctor/tests.rs` |
| 012 US1 | Start at every login, any init system | Implemented | `tests/install_autostart.rs:87-180,309`, `tests/e2e/E2E-K1.rs:8` |
| 012 US2 | systemd user unit | Implemented | `tests/install_autostart.rs:214-290` |
| 013 US1 | Every change is checked on Tier-1 sessions | Implemented | `.github/workflows/ci.yml:69-241` (checks, lint, test, integration, e2e, static, on `dev`/`main`) |
| 013 US2 | A tag publishes everything without a stored token | Partially Implemented | `.github/workflows/release.yml:8-10,176-240` (OIDC Trusted Publishing, `release` environment). Never run. |
| 013 US3 | The first release by hand, then token retirement | Planned | `docs/release.md` runbook. `make publish` exists (`Makefile`, the `publish` recipe). Not done yet. |

### 1.4 Surfaces (160)

**CLI commands (19).** All are Implemented. Help text comes from `src/cli/args.rs`, and dispatch is in `src/cli/mod.rs:86-146`. Every row was checked against the built binary (§3).

| Command | Evidence |
|---|---|
| `rstory daemon` | `src/main.rs:6-19`, `src/daemon/start.rs:388-405` |
| `rstory ls` | `args.rs:67-77`, `commands.rs:53` |
| `rstory show <REF>` | `args.rs:79-88`, `commands.rs:70` |
| `rstory copy <REF>` | `args.rs:90-94`, `commands.rs:86` |
| `rstory pin <REF>` | `args.rs:96-100`, `commands.rs:150` |
| `rstory unpin <REF>` | `args.rs:102-106`, `commands.rs:150` |
| `rstory rm <REF>` | `args.rs:108-112`, `commands.rs:158` |
| `rstory clear` | `args.rs:114-124`, `commands.rs:242` |
| `rstory pause` | `args.rs:126-133`, `commands.rs:174` |
| `rstory resume` | `args.rs:135-136`, `commands.rs:187` |
| `rstory status` | `args.rs:138-143`, `commands.rs:98` |
| `rstory config` | `args.rs:145-150`, `commands.rs:121-144` |
| `rstory pick` | `args.rs:152-157`, `src/picker/mod.rs:60` |
| `rstory doctor` | `args.rs:159-164`, `src/doctor/mod.rs:84` |
| `rstory tray` | `args.rs:166-167`, `src/tray/client.rs:144` |
| `rstory install-autostart` | `args.rs:169-174`, `src/install/autostart.rs:357` |
| `rstory install-gnome-extension` | `args.rs:176-181`, `src/install/gnome.rs:322` |
| `rstory completions <SHELL>` | `args.rs:183-189,212-220`, `tests/cli_completions.rs:11-68` |
| `rstory --version` / `-V` | `args.rs:20,23` (clap `version`, `propagate_version`). Prints `rstory 0.1.0`. |

**Flags (19):** 17 Implemented, 2 Unavailable.

| Flag | Status | Evidence |
|---|---|---|
| `ls --json` | Implemented | `args.rs:71-73` |
| `ls --limit <N>` (1–500) | Implemented | `args.rs:74-76` |
| `show --json` | Implemented | `args.rs:85-87` |
| `clear --all` | Implemented | `args.rs:118-120` |
| `clear --yes` / `-y` | Implemented | `args.rs:121-123`, `tests/cli_manage.rs:108` |
| `pause --for <DURATION>` (`<n>s\|m\|h`, 1s–24h) | Implemented | `args.rs:39-47,130-132`, `src/ipc/protocol.rs:22-23` |
| `status --json` | Implemented | `args.rs:140-142` |
| `config --path` | Implemented | `args.rs:147-149` |
| `pick --plain` | Implemented | `src/picker/mod.rs:33,50` |
| `doctor --json` | Implemented | `src/doctor/mod.rs:42` |
| `daemon --backend x11\|wayland` (also `--backend=`) | Implemented | `src/daemon/start.rs:221-236` |
| `install-autostart --systemd` | Implemented | `autostart.rs:97` |
| `install-autostart --now` | Implemented | `autostart.rs:98` |
| `install-autostart --uninstall` (not with `--systemd`/`--now`) | Implemented | `autostart.rs:99-105` |
| `install-gnome-extension --enable` | Implemented | `gnome.rs:99` |
| `install-gnome-extension --uninstall` (not with `--enable`) | Implemented | `gnome.rs:100-106` |
| `completions bash\|zsh\|fish` | Implemented | `args.rs:194-201` |
| `daemon --help` | **Implemented** (feat/docs-gaps; the evidence below is pre-fix) | `daemon` takes no help flag. It exits 2 with `rstory: unknown argument --help; usage: rstory daemon [--backend x11\|wayland]` (`src/daemon/start.rs:227-229,394`, observed in §3). |
| `--ascii` | **Implemented** (feat/docs-gaps; the evidence below is pre-fix) | Named in `docs/product/ux-vision.md` ("Glyphs"). `ascii_mode` is always called with `false` (`src/cli/render.rs:52`, `src/picker/mod.rs:153`). |

**Config keys (9):** 7 Implemented, 2 Unavailable. The file is `$XDG_CONFIG_HOME/rstory/config.toml`, falling back to `~/.config/rstory/config.toml` (`src/cli/env.rs:85-99`).

| Key | Status | Default / range | Evidence |
|---|---|---|---|
| `max_entries` | Implemented | 20, range 2–500 | `src/config/mod.rs:15,27,34,50-54` |
| `max_pinned` | Implemented | 10, range 0 to `max_entries - 1` | `src/config/mod.rs:17,56-68` |
| `max_entry_bytes` | Implemented | 1048576, range 1024–1048576 | `src/config/mod.rs:19,29,70-74` |
| `ttl` | Implemented | `"7d"`, from `1h` to `90d`, units `s/m/h/d` | `src/config/mod.rs:21,31,76-91,191-208` |
| `picker_command` | Implemented | unset (blank counts as unset) | `src/config/mod.rs:130-138`, `tests/daemon_status.rs:194` |
| `tray` | Implemented | `true` | `src/config/mod.rs:144-152`, `tests/tray_dbus.rs:636` |
| Unknown keys are ignored | Implemented | — | `src/config/mod.rs:39,232-239` |
| `theme` (`terminal\|rstory\|mono`) | **Implemented** (feat/docs-gaps; the evidence below is pre-fix) | — | The parser exists (`src/ui/theme.rs:30-55`), but no config reader calls it (`src/daemon/start.rs:255` reads caps, `picker_command` and `tray` only). |
| A backend override in config | **Implemented** (feat/docs-gaps; the evidence below is pre-fix) | — | `docs/product/brief.md` §6 and `src/session/mod.rs:131` say "config or `--backend`". Only `--backend` is wired (`src/daemon/start.rs:255,410-428`). |

**Environment variables (18):** 17 Implemented, 1 Unavailable.

| Variable | Status | Use | Evidence |
|---|---|---|---|
| `XDG_RUNTIME_DIR` | Implemented | socket at `<dir>/rstory/control.sock`, falling back to `/run/user/<uid>` | `src/ipc/server.rs:25-27,96-119`, `src/cli/env.rs:63-67` |
| `XDG_STATE_HOME` | Implemented | store at `<dir>/rstory` | `src/cli/env.rs:73-83`, `src/daemon/start.rs:245-252` |
| `XDG_CONFIG_HOME` | Implemented | config, autostart entry and unit | `src/cli/env.rs:89-99`, `src/install/autostart.rs:71` |
| `XDG_DATA_HOME` | Implemented | extension directory | `src/install/gnome.rs:66` |
| `HOME` | Implemented | fallback for all XDG directories and `~/.Xauthority` | `src/cli/env.rs:77`, `src/backend/x11/mod.rs:143` |
| `WAYLAND_DISPLAY` | Implemented | session detection | `src/daemon/start.rs:424`, `src/backend/wayland/mod.rs:251` |
| `DISPLAY` | Implemented | session detection and the XWayland bridge | `src/daemon/start.rs:425`, `src/backend/x11/mod.rs:163` |
| `XDG_SESSION_TYPE` | Implemented | session detection | `src/daemon/start.rs:426` |
| `XDG_CURRENT_DESKTOP` | Implemented | the "not GNOME" warning, doctor | `src/install/gnome.rs:76`, `src/doctor/probes.rs:91` |
| `XAUTHORITY` | Implemented | X11 authentication | `src/backend/x11/mod.rs:142` |
| `TERMINAL` | Implemented | first in the picker-terminal search | `src/daemon/mod.rs:58`, `src/launch/mod.rs:78` |
| `PATH` | Implemented | terminal lookup | `src/launch/mod.rs:121` |
| `NO_COLOR` (non-empty) | Implemented | monochrome | `src/ui/theme.rs:102,114` |
| `TERM` (`dumb`) | Implemented | monochrome | `src/ui/theme.rs:103,114` |
| `LC_ALL`, `LC_CTYPE`, `LANG` | Implemented | a non-UTF-8 locale switches to ASCII glyphs | `src/ui/glyphs.rs:48-66` |
| `COLUMNS` | Implemented | CLI width when the terminal size is unknown | `src/cli/render.rs:88` |
| `RUST_LOG` | Implemented | daemon log filter, default `info` | `src/daemon/log.rs:7-10` |
| `COLORTERM` | **Implemented** (feat/docs-gaps; the evidence below is pre-fix) | Read, but no effect: the `Terminal` preference returns earlier | `src/ui/theme.rs:104,117-119` |

Test-only variables (`RSTORY_SOAK_SECS`, `PROPTEST_CASES`, `CI`) and X11 atom names (`RSTORY_TARGETS`, `RSTORY_DATA`, `src/backend/x11/mod.rs:548`) aren't user surfaces. The site must not list them.

**Exit codes (10):** all Implemented.

| Code | Meaning | Evidence |
|---|---|---|
| 0 | ok | `src/cli/mod.rs:49` |
| 1 | unexpected. The picker also uses it for "dismissed" (Esc). | `src/cli/mod.rs:50`, `src/picker/state.rs:27,552` |
| 2 | usage, not found, ambiguous, no terminal | `src/cli/mod.rs:51`, `tests/cli_exit_codes.rs:57` |
| 3 | daemon not running (CLI writes, picker restore, `rstory tray`) | `src/cli/mod.rs:52`, `src/picker/state.rs:31`, `src/tray/client.rs:161` |
| 4 | daemon already running | `src/daemon/start.rs:46-52`, `tests/cli_exit_codes.rs:86` |
| 5 | environment (runtime directory, store, config, no systemd user) | `src/cli/mod.rs:54`, `src/install/autostart.rs:36` |
| 6 | limit reached (pin cap) | `src/cli/mod.rs:55`, `tests/cli_manage.rs:43` |
| 7 | inactive (no backend) | `src/cli/mod.rs:56`, `tests/e2e/E2E-K4.rs:8` |
| 130 | interrupted (Ctrl-C) | `src/cli/mod.rs:57`, `src/picker/state.rs:35,525-527` |
| doctor: 0 / 1 / 2 | everything ok / warnings / failures or a usage error | `src/doctor/mod.rs:161-170` |

**Control protocol v1 (16):** all Implemented. NDJSON over a Unix socket, schema in `contracts/protocol.schema.json`, contract tests in `tests/contract/protocol.rs:54-297`.

| Item | Evidence |
|---|---|
| op `list` (`limit` 1–500) | `src/ipc/protocol.rs:306-317` |
| op `show` | `protocol.rs:318` |
| op `restore` | `protocol.rs:319` |
| op `pin` | `protocol.rs:320` |
| op `unpin` | `protocol.rs:321` |
| op `rm` | `protocol.rs:322` |
| op `clear` (`all`) | `protocol.rs:323-329` |
| op `pause` (`for`, 1s–24h) | `protocol.rs:330-343` |
| op `resume` | `protocol.rs:344` |
| op `status` | `protocol.rs:345`, result shape `protocol.rs:91-118` |
| op `open_picker` | `protocol.rs:346` |
| op `subscribe` (`events`) | `protocol.rs:347-360` |
| event `history_changed` | `protocol.rs:120-139` |
| event `pause_changed` | `protocol.rs:120-139` |
| framing: `v` = 1, lines up to 2 MiB | `protocol.rs:16-19,284-289` |
| peer credentials: another uid is closed without a byte | `src/ipc/peercred.rs:7-12`, `tests/e2e/E2E-PEER.rs:43` |

**Protocol errors (11):** all Implemented. Defined at `src/ipc/error.rs:13-53`, CLI mapping at `error.rs:59-70`, test `tests/contract/protocol.rs:297`. Each row gives the wire code and its CLI exit code.

| Code | CLI exit |
|---|---|
| `bad_request` | 2 |
| `too_large` | 1 |
| `unsupported_version` | 1 |
| `internal` | 1 |
| `not_found` | 2 |
| `ambiguous` | 2 |
| `pin_limit` | 6 |
| `inactive` | 7 |
| `backend_error` | 1 |
| `no_terminal` | none (1) |
| `picker_command_failed` | none (1) |

**Sessions and backends (5):** 4 Implemented, 1 Upstream Limitation.

| Path | Status | Evidence |
|---|---|---|
| X11 session → `x11`, native | Implemented | `src/session/mod.rs:163-165`, `tests/daemon_x11.rs:30` |
| Wayland with `ext-data-control-v1` → `wayland` | Implemented | `src/backend/wayland/mod.rs:1,48`, `tests/wayland_capture.rs:87` |
| Wayland with `zwlr_data_control_manager_v1` (fallback) | Implemented | `tests/wayland_capture.rs:69,75` |
| Wayland without data-control but with XWayland (GNOME) → `x11` via the XWayland bridge | Implemented | `src/session/mod.rs:167-175`, `tests/bridge_capture.rs:30-84`, `tests/e2e/E2E-K3.rs:48` |
| Wayland with neither data-control nor XWayland → inactive | Upstream Limitation | `src/session/mod.rs:171-175`, `tests/daemon_capture.rs:103`, `tests/e2e/E2E-K4.rs:8` |

**Privacy (6):** 5 Implemented, 1 Upstream Limitation.

| Item | Status | Evidence |
|---|---|---|
| Sensitive hint `x-kde-passwordManagerHint` (exact, case-sensitive), checked before reading | Implemented | `src/domain/filter.rs:13,74-78,160-162` |
| Size cap: drop, never truncate, `skipped_oversize` | Implemented | `tests/daemon_privacy.rs:60` |
| Pause gate, checked before the hint, `skipped_paused` | Implemented | `tests/daemon_privacy.rs:79` |
| No content in logs, errors or doctor output | Implemented | `tests/fitness/canary.rs:37`, `tests/doctor_cli.rs:113` |
| Store directory `0700`, files `0600`; a store open to others is refused | Implemented | `src/store/perms.rs:10-12`, `tests/fitness/perms.rs:46-109` |
| Apps that set no hint (for example `pass`) are recorded | Upstream Limitation | `README.md` "Threat model". Workaround: the `passc` wrapper (`README.md`, `args.rs:127`). |

**Picker (4):** all Implemented.

| Item | Evidence |
|---|---|
| Keys: Enter restore · Tab preview · Ctrl-T pin/unpin · Ctrl-D delete (y/n) · Esc quit · Ctrl-C interrupt · F1 keymap · Up/Down/Ctrl-P/Ctrl-N · PageUp/PageDown/Home/End · Backspace/Ctrl-U/Ctrl-W | `src/picker/state.rs:523-605`, `src/ui/copy.rs:7-9` |
| Read-only without a daemon (banner; Enter exits 3) | `src/picker/state.rs:612-618`, `tests/e2e/E2E-K2.rs:69` |
| `--plain` inline mode | `src/picker/view.rs:228-239` |
| Live updates (`subscribe`) and "too small" below 20×5 | `src/picker/mod.rs:155-159`, `src/picker/view.rs:49-53` |

**Tray (5):** all Implemented.

| Item | Evidence |
|---|---|
| SNI item with pixel icons and a recording tooltip | `tests/tray_dbus.rs:398`, `src/tray/icons.rs:31` |
| Menu: last 10 entries, Pause/Resume, Pause for 1 minute, Open picker (hidden with no terminal), Clear history… with a confirm | `src/tray/menu.rs:10,220-250`, `tests/tray_dbus.rs:422-546` |
| Icon shape changes with state (recording, paused) | `src/tray/icons.rs:82`, `tests/tray_dbus.rs:462` |
| No host or no bus: the daemon keeps running (`no_host`, `no_bus`) | `tests/tray_dbus.rs:592,622` |
| `tray = false`, plus a separate `rstory tray` process | `tests/tray_dbus.rs:636`, `src/tray/client.rs:144-175` |

**GNOME extension (5):** 4 Implemented, 1 Unavailable.

| Item | Status | Evidence |
|---|---|---|
| UUID `rstory@lucasvmigotto.me`, installed to `$XDG_DATA_HOME/gnome-shell/extensions/<uuid>` | Implemented | `src/install/gnome.rs:20,112`, `tests/install_gnome.rs:110` |
| Panel icon and menu over the control socket; disconnected icon; no process spawning, no D-Bus | Implemented | `extension/tests/panel.test.js:39-204`, `extension/tests/hygiene.test.js:30-39` |
| GNOME Shell 48, 49, 50 | Implemented | `dev@b4a998e`: `extension/metadata.json:5`, `src/doctor/checks.rs:20`, `src/install/gnome.rs:309` |
| `make ext-pack` builds the extensions.gnome.org zip (dev tooling) | Implemented | `dev@b4a998e`: `Makefile` `ext-pack`, `scripts/ext-pack.py` |
| GNOME Shell 51 | **Unavailable** | Out of `SHELL_SUPPORTED` on `dev@b4a998e`. Doctor warns "the panel icon needs GNOME Shell 48 to 50". |

**Doctor checks (10):** all Implemented. Order at `src/doctor/checks.rs:90-101`. Ids, in order: `session` (`:111`), `backend` (`:130`), `store` (`:167`), `daemon` (`:202`), `pause` (`:271`), `tray` (`:285`), `gnome_shell` (`:321`), `gnome_extension` (`:331`), `picker` (`:369`), `autostart` (`:392`).

**Autostart (4):** all Implemented.

| Item | Evidence |
|---|---|
| XDG entry `$XDG_CONFIG_HOME/autostart/rstory.desktop` | `src/install/autostart.rs:13-16,33,109`, `tests/install_autostart.rs:87` |
| systemd user unit `$XDG_CONFIG_HOME/systemd/user/rstory.service` | `autostart.rs:20-34,113`, `tests/install_autostart.rs:214` |
| `--now` starts the daemon now | `tests/install_autostart.rs:240,309` |
| `--uninstall` removes both kinds | `tests/install_autostart.rs:132,290` |

**Release channels (8):** 1 Implemented, 1 Partially, 5 Planned, 1 N/A.

| Channel | Status | Evidence |
|---|---|---|
| CI gate matrix (x86_64 and aarch64) | Implemented | `.github/workflows/ci.yml:99-241` |
| Release workflow on plain `X.Y.Z` tags (no `v`) | Partially Implemented | `.github/workflows/release.yml:8-10`. Never run. |
| GitHub Release `rstory-X.Y.Z-<arch>-unknown-linux-musl.tar.gz` | Planned | `release.yml:109-125,212-240` |
| `SHA256SUMS` asset | Planned | `release.yml:134-155` |
| crates.io (`cargo install rstory`) | Planned | `release.yml:176-210`, `docs/release.md` |
| extensions.gnome.org | Planned | `dev@b4a998e`: `docs/release.md` "Publish the extension on extensions.gnome.org (after 0.1.0)" |
| Distribution packages (AUR, deb, rpm, apk, Nix) | Planned | brief §6 "Release & distribution", tagged Later |
| The `runtime` stage of the Containerfile | N/A | `Containerfile`, `FROM scratch AS runtime`. Not a published channel; nothing pushes it. |

**Other (1):** an HTTP API or OpenAPI contract is **N/A**. rstory has no network surface; the seam is the control socket (`specs/README.md` "Shared artifacts").

**Roadmap (10, all Planned).** These are from `specs/README.md` "Next and Later" and brief §6, and none is specified yet: 016 GNOME popup list (Next) · 017 images `image/png` (Next) · 018 primary selection, opt-in (Later) · 019 encrypted store (Later) · 020 pinned export/import (Later) · 021 source app and ignore list (Later) · 022 rich text and file lists (Later) · 023 portal or extension-fed capture (Later) · interface translations (Later) · paste injection (Later).

---

## 2. No-invent list

The site may show only these strings. Anything else needs a new row here, traced to code, first. Build-time tests in Phase 5 check every code block and table against this list.

**Binary and commands.** `rstory`, plus each command in §1.4. Use the usage lines exactly as the binary prints them (§3).

**Flags.** `--json`, `--limit <N>`, `--all`, `--yes`, `-y`, `--for <DURATION>`, `--path`, `--plain`, `--backend x11|wayland`, `--backend=<x11|wayland>`, `--systemd`, `--now`, `--uninstall`, `--enable`, `-h`, `--help`, `-V`, `--version`, and the shells `bash`, `zsh`, `fish`.

Never show `--ascii`, `daemon --help`, `rstory help <pick|doctor|install-*>` (it prints clap's pass-through `[ARGS]...` form, not the real usage), or `--debug`/`--verbose`, which don't exist.

**References.** `REF` is an index (`0` is the newest) or a short id of at least 4 characters (`args.rs:24,52`). Durations: config `ttl` takes `s|m|h|d`; `pause --for` takes `s|m|h` (`protocol.rs` `parse_duration`, `args.rs:42`).

**Config keys.** `max_entries`, `max_pinned`, `max_entry_bytes`, `ttl`, `picker_command`, `tray`. Never `theme` or `backend`, until §1.4 changes.

**Environment variables.** The 17 Implemented rows of §1.4. `TERMINAL` is a lookup input, not a setting rstory owns. Never `COLORTERM` as a switch.

**Paths.**

- `$XDG_CONFIG_HOME/rstory/config.toml` (`~/.config/rstory/config.toml`)
- `$XDG_STATE_HOME/rstory/` (`~/.local/state/rstory/`) with `index.json`, `store.lock` and `content/` (`src/store/index.rs:16`, `src/store/mod.rs:26-28`)
- `$XDG_RUNTIME_DIR/rstory/control.sock` (`/run/user/<uid>/…` fallback)
- `$XDG_CONFIG_HOME/autostart/rstory.desktop`
- `$XDG_CONFIG_HOME/systemd/user/rstory.service`
- `$XDG_DATA_HOME/gnome-shell/extensions/rstory@lucasvmigotto.me/`

**Exit codes.** `0 1 2 3 4 5 6 7 130`, and doctor's `0 1 2`, with the meanings in §1.4.

**Protocol.** Ops, events and error codes as in §1.4. Fields of `status`: `state` (`recording|paused|inactive`), `paused_until`, `reason`, `backend` (`wayland|x11|none`), `mode` (`native|xwayland_bridge|null`), `entries`, `pinned`, `max_entries`, `max_pinned`, `counters.{skipped_sensitive,skipped_oversize,skipped_paused,dropped_burst}`, `tray` (`active|no_host|no_bus|disabled`), `picker_available`, `version`, `protocol` (`src/ipc/protocol.rs:52-118`).

JSON outputs carry `schema: 1`, and read-only listings add `read_only: true` (observed: `{"entries":[],"read_only":true,"schema":1}`).

**Messages.** Quote these verbatim, with `{placeholders}` shown as such:

- every constant in `src/ui/copy.rs:3-44`
- `ConfigError` (`src/config/mod.rs:166-187`): `config.toml: …`, `{key} = {value} is out of range; allowed: {allowed}`, `{key} = {value:?} is invalid: {reason}`
- `XDG_RUNTIME_DIR is not set and {0} does not exist or is not owned by this user; set XDG_RUNTIME_DIR to a private directory` (`src/ipc/server.rs:91`)
- `already running (pid {pid})` (`src/daemon/start.rs:33`)
- `The daemon did not answer in time. Run \`rstory doctor\` for details.` (`src/cli/error.rs:75`)
- installer messages (`src/install/gnome.rs:27-31,282-300`, `src/install/autostart.rs:97-105`)
- doctor messages and hints (`src/doctor/checks.rs`)
- the parse errors `not an index (0, 1, ...) or a short id of at least 4 characters`, `use a number and a unit: 50s, 2m or 1h`, `a pause lasts from 1s to 24h` (`args.rs:36,42,44`)

**Product facts.**

- Default caps: 20 entries, 10 pins, 1 MiB, 7 days.
- Terminal search order: `$TERMINAL` → `xdg-terminal-exec` → `ghostty kitty alacritty foot wezterm ptyxis kgx gnome-terminal konsole xfce4-terminal xterm` (`src/launch/mod.rs:16-31`).
- Sensitive hint `x-kde-passwordManagerHint`.
- GNOME Shell 48–50.
- Tier-1 sessions: as `docs/release.md` lists them.
- License `GPL-3.0-or-later` (`Cargo.toml:5`).
- Targets `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`.
- Version from `Cargo.toml` only.

**Never show.** Real R2 bucket, account or endpoint values; tokens; `CARGO_REGISTRY_TOKEN` values; real clipboard content in demos (only typed throwaway text); invented install commands (for example `curl | sh`, `apt install rstory`, `cargo install --git`).

---

## 3. Command-reference plan

The CLI reference is generated from the built binary and never written by hand.

**What discovery observed.** The binary was built with `make TARGET_VOLUME=rstory-tools-target-docs build` and run in a `--rm` scratch container off the tools image, with `--network=none`, `NO_COLOR=1` and `COLUMNS=100`.

- `rstory --help` and every clap subcommand's `--help` exit 0. Each lists its options and the `Examples:` after-help.
- `pick --help`, `doctor --help`, `install-autostart --help` and `install-gnome-extension --help` print their own hand-written help (`src/picker/mod.rs:33,64`, `src/doctor/mod.rs:17-26`, `src/install/autostart.rs:344-355`, `src/install/gnome.rs:309-318`) and exit 0.
- `rstory daemon --help` exits **2** with a usage line on stderr (`src/daemon/start.rs:227-229`).
- `rstory help pick` prints clap's pass-through stub (`Usage: rstory pick [ARGS]...`), which differs from `rstory pick --help`.
- `install-gnome-extension --help` says "Shell 48 to 51" on this branch and "48 to 50" on `dev@b4a998e`. That's the case for generating at the release ref, not copying text.

**Pipeline (Phase 2+).**

1. **`make docs-reference`** runs in the **tools** image, offline:
   - `cargo build --locked --offline` into the caller's `TARGET_VOLUME`.
   - Then `docs/site/scripts/cli-help.sh` (POSIX sh, run in the container) with `NO_COLOR=1`, `COLUMNS=100`, `LANG=C.UTF-8`, `TZ=UTC`, and an empty `HOME`, `XDG_*` and session in a tmpfs.
2. **The command list is discovered, not hard-coded.** The script parses the `Commands:` block of `rstory --help` (dropping clap's `help`), then runs `rstory <cmd> --help` for each. It records stdout, stderr and the exit code.
3. **It also captures:**
   - `rstory --version`
   - `rstory config` with no file, which gives the defaults table: `max_entries = 20`, `max_pinned = 10`, `max_entry_bytes = 1048576`, `ttl = "7d"`
   - `rstory doctor --json` with no session, which gives the check ids and hint strings
   - `rstory completions <shell> | head` (existence only)
4. **The output** is `docs/site/src/content/generated/cli-help.json`, with `{ "_generated": "make docs-reference — do not hand-edit", "rstory": "<version>", "commands": [{ "name", "usage", "help", "exit" }] }`. The site renders it in a `CommandHelp` component as preformatted text, and pulls `Examples:` blocks out as copyable code. The same JSON feeds `llms.txt` and the `.md` pages.
5. **Staleness gate.** CI runs `make docs-reference` and fails on `git diff --exit-code docs/site/src/content/generated/`, so changing help text without regenerating breaks the PR.
6. **The `daemon` gap.** Either `rstory daemon --help` gets a real help (a small amendment to spec 004, recommended), or the generator records `daemon`'s usage line from its stderr and marks it `"exit": 2`. The site never paraphrases it. This is question Q2.

---

## 4. Demo plan

Four `vhs` tapes live in `docs/site/demos/`. Each is followed by a text transcript on the page, so the content doesn't depend on video.

| Tape | Shows | Setup (in a `Hide` block) |
|---|---|---|
| `picker-recall.tape` | `rstory pick`: type a query, the matches narrow, Tab previews, Enter restores, `xclip -o -selection clipboard` prints the restored text | `Xvfb :99`, `rstory daemon &`, three throwaway copies with `printf … \| xclip -selection clipboard` |
| `cli.tape` | `rstory ls`, `rstory show 1`, `rstory copy 1`, `rstory ls --json \| jq '.entries[].short_id'` (from the help's own examples) | as above |
| `pause-passc.tape` | `rstory pause --for 50s`, then `rstory status` showing "Paused — resumes in …", a copy that `ls` doesn't list, the `skipped_paused` counter, then `resume` | daemon on Xvfb. `pass` itself needs a GPG keyring, so the tape shows the `passc` definition and a stand-in `xclip` copy, and says so on screen (Q6). |
| `doctor.tape` | `rstory doctor` on a working Xvfb session with the daemon, then `rstory doctor --json \| jq '.checks[] \| select(.status != "ok")'` | daemon on Xvfb, `install-autostart` in a tmp `HOME` |

**Rendering in a container.** The tapes need the tools image's session stack (Xvfb, xclip, sway) plus vhs, ttyd, ffmpeg and a headless Chromium. Neither image has both. The proposal is a `demo` stage in `Containerfile`, `FROM tools`:

- `vhs` and `ttyd` from their GitHub releases, checked against pinned SHA-256 sums the way `deps` fetches cargo-deny
- `ffmpeg` and `chromium` from trixie apt
- Reference: `ghcr.io/charmbracelet/vhs:v0.12.1@sha256:ea49a6a1c529be83153e88321892b5585964418f0b9055e8c1e0d732194234a3` (multi-arch index, created 2026-09-24) is the version to match.
- `make demo-image` builds it, and `make docs-demos` runs `vhs` on each tape with `--network=none` and writes `docs/site/public/demos/<name>.webm`, a `.gif` fallback and a first-frame `.png` poster.

Adding this image is an owner decision (Q5). Discovery created no image.

**Determinism.**

- Fixed `Set Width/Height/FontSize/TypingSpeed/Theme`, `TZ=UTC`, `NO_COLOR` unset with `TERM=xterm-256color`.
- The daemon runs with a tmp `XDG_*`, so there's no host state.
- Short ids are random per run, so rendered media aren't byte-stable. They're **not committed**; only the `.tape` files are.

**CI.** The docs workflow's verify job builds the `demo` stage with the BuildKit layer cache and runs `make docs-demos` into `dist/demos/`.

- Its paths filter covers `src/**`, `docs/site/demos/**` and `Containerfile`.
- `vhs validate` runs on every tape in `docs-check`, so a broken tape fails fast.
- The deploy uses the same artifact, so demos always match the deployed binary.
- A demo failure fails the build: a stale or broken demo never ships.
- For pages that need no video, picker stills can come from the existing insta snapshots (`src/picker/snapshots/`), the way rusteams renders `tui-shots.json` with a `--check` gate.

**Accessibility.** No autoplay; controls shown; `prefers-reduced-motion` gets the poster plus the transcript; every demo has a caption and a text alternative.

---

## 5. Information architecture

Routes are hash routes (`#/…`) under `https://docs.lucasvmigotto.me/rstory/`. Every page carries a maturity badge. Terms follow the brief's glossary (§9) verbatim. The words to avoid are clip, item, snippet, buffer, paste and service.

| Section · page | Route | Sources | Maturity |
|---|---|---|---|
| **Overview** | `#/` | README summary and privacy, brief §1 and §3 | Implemented (states "0.1.0 not released yet") |
| **Get started** · Install | `#/install` | `release.yml`, `docs/release.md`, `make build-static` | **Planned** (crates.io, release binary). "Build from source with `make build-static`" is Implemented. |
| Get started · First run | `#/first-run` | `install-autostart`, `doctor`, `install-gnome-extension` help (generated) | Implemented |
| Get started · Your desktop | `#/desktops` | `src/session/mod.rs`, brief §11 Tier 1, `docs/release.md` matrix | Implemented. The no-XWayland row is labelled Upstream Limitation. |
| **Guides** · Recall with the picker | `#/guides/picker` | picker keys, `picker_command`, `picker-recall.tape` | Implemented |
| Guides · Script with the CLI | `#/guides/cli` | `--json`, exit codes, contract tests, `cli.tape` | Implemented |
| Guides · Pause and `passc` | `#/guides/pause` | README `passc`, `pause-passc.tape` | Implemented |
| Guides · Pins and clearing | `#/guides/pins` | caps, BR9–BR10 | Implemented |
| Guides · Tray and GNOME panel icon | `#/guides/tray` | spec 009 and 010 code, menu copy | Implemented (GNOME 48–50) |
| Guides · Troubleshoot with doctor | `#/guides/doctor` | doctor checks and hints, `doctor.tape` | Implemented |
| **Concepts** · Entries and history | `#/concepts/history` | brief glossary and BR1–BR15, domain-model §5 | Implemented |
| Concepts · Privacy and threat model | `#/concepts/privacy` | README "Privacy", spec 005 code, domain-model §11 | Implemented. The no-hint row is Upstream Limitation. |
| Concepts · Daemon and control socket | `#/concepts/daemon` | ADR-0002, ADR-0004, `src/ipc` | Implemented |
| Concepts · Backends and the XWayland bridge | `#/concepts/backends` | ADR-0005, ADR-0006, session tests | Implemented |
| **Reference** · CLI | `#/reference/cli` | `generated/cli-help.json` | Implemented |
| Reference · Configuration | `#/reference/config` | `src/config/mod.rs`, `rstory config` output | Implemented (6 keys) |
| Reference · Environment variables | `#/reference/env` | §1.4 | Implemented |
| Reference · Exit codes | `#/reference/exit-codes` | `src/cli/mod.rs`, `contracts/cli.md` | Implemented |
| Reference · Files and paths | `#/reference/files` | §2 paths | Implemented |
| Reference · Picker keys | `#/reference/keys` | `src/picker/state.rs` | Implemented |
| Reference · Doctor checks | `#/reference/doctor` | `doctor --json` (generated) | Implemented |
| Reference · Control protocol v1 | `#/reference/protocol` | `contracts/protocol.md`, `protocol.schema.json` | Implemented (contract-tested) |
| **Project** · Architecture and decisions | `#/project/architecture` | `docs/product/architecture.md`, ADR-0001…0009 | Implemented where the fitness tests confirm it (`tests/fitness/`) |
| Project · Roadmap | `#/project/roadmap` | `specs/README.md` Next/Later, brief §6 | **Planned** (every item) |
| Project · Releases and versions | `#/project/releases` | `release.yml`, `docs/release.md` | **Planned** until 0.1.0 |
| Project · Limitations | `#/project/limitations` | Upstream Limitation and Unavailable rows | as labelled |
| Project · Accessibility | `#/project/accessibility` | ux-vision "Accessibility targets", `--plain`, `NO_COLOR` | Implemented. The `--ascii` mention is dropped. |
| Project · Development | `#/project/development` | README "Development", `Makefile help` | Implemented |
| Project · Glossary | `#/glossary` | brief §9 | — |

**Navigation.** The header holds the sections, search (the Search component), the language switcher (en-US only for now, built so locales can be added) and the theme toggle. The footer holds `rstory@v0.1.0 · docs@v0.1.0`, the license (linked at the build's ref) and the theme switch.

**LLM output.** `llms.txt`, `llms-full.txt` and `/docs/en-US/<slug>.md`, generated from the same typed content.

---

## 6. Reference patterns

I read three siblings and changed nothing in them: `~/codes/rusteams/docs/site`, `~/codes/ai-gent/docs/site` and `~/codes/docs` (the hub).

| Area | Reuse | Adapt | Fix (don't copy) |
|---|---|---|---|
| **Visual family** | Dark-first token layout, restrained motifs (both sites) | Tokens come from rstory's own design system: accent teal `#2AA198` (dark) / `#1F7A73` (light), muted `#8A8A8A`/`#6C6C6C`, pinned and warning `#D7AF00`/`#8A6D00`, danger `#D70000`/`#AF0000` (`specs/000-design-system/tokens.md`). The favicon and OG motif is the clipboard-with-three-lines (`assets/icons/recording-symbolic.svg`). Monospace for terminal content. Status is always a word, never color alone (the ux-vision rule). | — |
| **Router** | `HashRouter`, Vite `base: "/rstory/"`, **no** router `basename` (hub ADR-0004 and ADR-0005; `ai-gent/docs/site/src/App.tsx:19`) | — | — |
| **Test harness** | ai-gent: vitest + happy-dom + jest-axe; tests for i18n, LLM output, meta and version; Selenium smoke in `selenium/standalone-chrome:153.0@sha256:7efe71e7e4a83bdf574b26bd354690928075e8f443223d2ced16a2c208eae1d7` | Every run goes through `make` in containers, not host Bun | ai-gent's e2e is advisory: make it a blocking PR gate. rusteams runs jsdom under a host Node: use happy-dom under Bun instead. |
| **Locale architecture** | ai-gent's typed `LOCALES = ["en-US"] as const` with en-US fallback | `ui` + `docs` namespaces per locale, behind one shared content interface, so a missing key is a type error | rusteams keeps two parallel systems (`src/i18n/locales.ts` with `"en"` plus `src/i18n/content/`) and uses `en` instead of `en-US`. Unify into one. |
| **R2 deploy and cache** | ai-gent `docs-ci.yml`: hashed `assets/` immutable first; `index.html` `no-cache`; `llms*.txt` as `text/plain; charset=utf-8` and `no-cache`; `docs/**/*.md` as `text/markdown; charset=utf-8` and `no-cache`; old assets pruned last; prefix guard (`R2_PREFIX`); endpoint validation; secrets only as `CLOUDFLARE_R2_ACCOUNT_ID`, `CLOUDFLARE_R2_ACCOUNT_SECRET`, `CLOUDFLARE_R2_ENDPOINT_S3_CLIENT`, `CLOUDFLARE_R2_BUCKET_ID` | Environment `rstory-docs`, url `https://docs.lucasvmigotto.me/rstory/` | rusteams syncs everything as `immutable` (favicon and OG image too), deploys on both push and `workflow_run`, pins actions by tag (`@v7`), and tracks a `.env` in git: don't copy any of that. ai-gent's deploy job **rebuilds** instead of deploying the verified artifact: build once, and deploy that artifact from the same run. |
| **Deploy trigger** | Deploy on the release workflow's completion | rstory's `release.yml` runs on **tag** pushes (`release.yml:8-10`), so `workflow_run.head_branch` is the tag, and the siblings' `branches: [main]` filter would **never match**. Filter instead with `if: conclusion == 'success' && github.event.workflow_run.event == 'push'` (this skips `workflow_dispatch` dry runs). Check out `workflow_run.head_sha` with `fetch-depth: 0`, and check the tag is `X.Y.Z`. | — |
| **Version footer** | rusteams `scripts/footer-versions.ts`: a pure function with a test per row | Tag pattern `[0-9]*.[0-9]*.[0-9]*` (plain, no `v`). Display `rstory@v0.1.0 · docs@v0.1.0`; past the tag, `rstory@v0.1.0+abc1234`. Links go to `/tree/<ref>`, and source links to `/blob/<ref>/…`, never `blob/main`. Git facts are passed **from the host** as env (`RSTORY_GIT_TAG`, `RSTORY_GIT_SHA`), because inside a container a worktree's `.git` file points at a gitdir that isn't mounted. | — |
| **LLM output** | ai-gent `scripts/generate-llms.ts` and `llms-pages.ts` | Generated from the typed content and `cli-help.json` | rusteams has none |
| **TUI stills** | rusteams `tui-shots.json` with a `--check` CI gate | Render from rstory's insta snapshots | — |
| **Hub** | `~/codes/docs/site/src/apps.ts` and `public/sitemap.xml` list each project | An `rstory` row in both, as a PR in the hub repo (Q7) | — |
| **Bun location** | — | — | Every sibling runs Bun on the runner host. rstory runs it in a container through `make`, the way every other recipe runs (§7). |

---

## 7. Build environment

**Image.** `docker.io/oven/bun:1.4.2@sha256:9114c058aeae42162ee16dd5084b95fe9473970bb6bcb5b232ab1630f0546895`

- This is the multi-arch index digest (amd64 and arm64), verified with `podman manifest inspect`. The image was created 2026-09-05, so it's 7 weeks old, which clears the two-week minimum.
- Bun 1.4.2 matches ai-gent's pinned CI version.
- I didn't pick `dhi.io/bun:1-debian13-dev`: it's a floating tag, its build (1.4.3, 2026-10-10) sits right at the two-week line, and it's 14 days old.
- The image runs as user `bun` (uid 1000), which matches the repo's `keep-id` mapping.
- Pin it by digest in the Makefile, with the tag as a comment. Dependabot's `docker` ecosystem can't see Makefile variables, so add a `docs/site/Containerfile` stub (`FROM …@sha256:…`) for Dependabot to read. This is Q12.

**Makefile recipes** (proposed; added in Phase 2):

```make
# Docs site (docs/site): Bun runs only in this pinned container, never on the host.
DOCS_IMAGE  := docker.io/oven/bun:1.4.2@sha256:9114c058aeae42162ee16dd5084b95fe9473970bb6bcb5b232ab1630f0546895
DOCS_GIT    := -e RSTORY_GIT_SHA=$(shell git rev-parse HEAD 2>/dev/null) \
               -e RSTORY_GIT_TAG=$(shell git describe --tags --exact-match --match '[0-9]*.[0-9]*.[0-9]*' 2>/dev/null)
DOCS_RUN    := $(ENGINE) run --rm --init $(USERNS) --cpus $(CPUS) --memory $(MEMORY) --pids-limit 2048 \
               -v "$(CURDIR)":/src:Z -v rstory-docs-node-modules:/src/docs/site/node_modules \
               -v rstory-docs-bun-cache:/home/bun/.bun/install/cache -w /src/docs/site \
               -e TZ=UTC -e LANG=C.UTF-8 $(DOCS_GIT)

docs-install: ## Install the docs site's packages into a volume (bun install --frozen-lockfile; the only networked docs step)
	$(DOCS_RUN) $(DOCS_IMAGE) bun install --frozen-lockfile
docs-dev: ## Serve the docs site with hot reload on http://127.0.0.1:5173/rstory/
	$(DOCS_RUN) -p 127.0.0.1:5173:5173 $(TTY) $(DOCS_IMAGE) bun run dev --host 0.0.0.0 --port 5173 --strictPort
docs-check: ## Lint (Biome), typecheck, unit + a11y + LLM-output tests, generated-reference freshness, offline
	$(DOCS_RUN) $(OFFLINE) $(DOCS_IMAGE) sh -c 'bun run lint && bun run typecheck && bun run test'
docs-build: ## Build docs/site/dist (+ llms.txt, .md pages), offline
	$(DOCS_RUN) $(OFFLINE) $(DOCS_IMAGE) bun run build
docs-reference: image ## Regenerate the CLI reference from the built binary's --help (tools image), offline
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'cargo build --locked --offline && sh docs/site/scripts/cli-help.sh /target/debug/rstory > docs/site/src/content/generated/cli-help.json'
```

A few notes on these recipes:

- They follow the repo's conventions: `--rm --init`, `keep-id`, CPU, memory and pids limits, `--network=none` for everything but install, and named volumes so `node_modules` never lands in the repo (as `EXT_MOUNT` does).
- `docs-dev` needs the network to publish its port, so it's the one recipe not marked offline.
- The two new volumes (`rstory-docs-node-modules`, `rstory-docs-bun-cache`) go into `clean-cache`.
- `docs-e2e` (Selenium, with the pinned Chrome image) and `docs-demos` (§4) follow in Phase 5.
- CI calls these same recipes.

---

## 8. Open questions for the owner

1. **Deploys before 0.1.0.** "After each release" means nothing goes live until the first tag. Should a preview deploy from `main` run before then, labelled as unreleased (footer `rstory@v0.1.0+<sha>`)?
2. **`rstory daemon --help`.** It exits 2. Should it get a real help (a spec 004 amendment) before the reference is generated, or should the reference show the stderr usage line?
3. **`theme`, truecolor and `--ascii`.** These are documented in ux-vision and the design system, but not wired. Should they be wired (this would bring 000 up to Implemented), or dropped from ux-vision? Until then the site describes the 16-color and mono tiers only.
4. **Backend override in config.** The brief says "config or flag", and only `--backend` exists. Should a config key be added, or the brief corrected?
5. **The demo image.** Should a `demo` stage go into `Containerfile` (tools + vhs + ttyd + ffmpeg + chromium, sums pinned), with media rendered in CI and never committed?
6. **The `passc` demo.** Should it use real `pass` with a throwaway GPG key in the container, or a labelled stand-in copy?
7. **Hub registry.** Should I open a PR to `lucasvmigotto/docs` adding `rstory` to `site/src/apps.ts` and `site/public/sitemap.xml`?
8. **Web palette.** Is the teal accent (`#2AA198` on dark, `#1F7A73` on light) from tokens.md right for the site, with `recording-symbolic.svg` as the favicon?
9. **Docs versioning.** Should `docs/site/package.json` start at `0.1.0`? And who raises it: a step in the `docs/release.md` "Every later release" checklist, or a release tool? rstory has no `release.json` flow.
10. **The Install page before release.** Is "Planned: `cargo install rstory` and release tarballs from 0.1.0; today: build from source" acceptable wording?
11. **GNOME Shell 51.** Should it be shown as Unavailable ("not declared until its manual matrix passes"), matching `dev@b4a998e`?
12. **The CI runner.** Should the docs workflow call `make docs-*` (Podman or Docker on the runner, consistent with `ci.yml`'s tools action), rather than `oven-sh/setup-bun` on the host? And should a stub `docs/site/Containerfile` be added so Dependabot can track the Bun image?

## Owner decisions (2026-10-24)

- **Gaps:** implement them before the site, rather than trimming the docs: a real `rstory daemon --help`, the `theme` config key with the color tiers it unlocks, `--ascii`, and a `backend` config key. The site then documents them as Implemented.
- **Deploy:** nothing goes live before 0.1.0. CI builds and tests the site on every PR; the first deploy is the 0.1.0 release, so the Install page is real from day one.
- **Docs hub:** rstory is added to the hub (`apps.ts` and the sitemap) by a small PR when the site first deploys.
- **Already decided:** the site lives in `docs/site/`, in English only, hosted on R2 at `docs.lucasvmigotto.me/rstory/`. The extension declares GNOME Shell 48–50.

## Update 2026-10-24: gaps implemented (feat/docs-gaps)

`rstory daemon --help`/`-h`, the global `--ascii` flag (also on `pick` and `doctor`), the `theme` config key (`terminal`, the default, plus `rstory` and `mono`) with the 256-color and truecolor tiers it unlocks (`COLORTERM` now has an effect under `theme = "rstory"`), and the `backend` config key (the `--backend` flag wins; doctor reports a config override; an unknown value exits 5 naming the key). `contracts/cli.md` gains `[--help]` on `daemon`, `[--ascii]` on `pick`/`doctor` and a global-flag line. The only Unavailable surface left is GNOME Shell 51. Phase 2 regenerates the truth table's evidence from the code.

