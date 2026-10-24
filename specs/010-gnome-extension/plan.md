# Implementation Plan: GNOME Shell extension

**Branch**: `feat/gnome-extension` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/010-gnome-extension/spec.md`

## Summary

An ESM GJS extension in `extension/` (PanelMenu.Button + PopupMenu, menu per `009/contracts/menu.md`) that talks only NDJSON over the control socket with reconnect backoff; embedded with include_dir and installed/enabled/uninstalled by `rstory install-gnome-extension`.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: Rust: include_dir 0.7; JS: GNOME Shell 48–50 APIs (`resource:///org/gnome/shell/...` ESM imports, Gio, GLib, St); dev: eslint (CI lint), gjs for fixture tests when available, TypeScript 7.0.2 as a checker only (`tsc --noEmit --checkJs`, JSDoc + `// @ts-check`, `@girs/gnome-shell` 50.0.4 types; no transpile, ADR-0008 amendment)

**Storage**: Installed files under `$XDG_DATA_HOME/gnome-shell/extensions/<uuid>/`

**Testing**: Rust install/uninstall tests (temp XDG dirs), JS client fixture tests under gjs, eslint, `make ext-typecheck`, manual matrix Shell 48–51

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: Menu open < 50 ms (data cached from subscribe)

**Constraints**: No subprocess, no D-Bus, full cleanup in disable()

**Scale/Scope**: —

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | assets embedded; no runtime install needed beyond the binary |
| II. Tests required per story | ✅ | install tests + fixture tests first |
| III. Privacy by default | ✅ | no content logged by the extension |
| IV. Single writer, versioned protocol | ✅ | socket client only; no logic (ADR-0008) |
| V. Budgets stated up front | N/A | — |
| VI. CLI parity and scriptability | ✅ | mirrors tray, maps to CLI |
| VII. Accessible interfaces | ✅ | labels; shape-distinct icons |
| VIII. Diagnosable, honest failures | ✅ | disconnected state; doctor detects outdated/disabled |
| IX. Versioning and review | ✅ | `version-name` = crate version |

## Project Structure

### Documentation (this feature)

```text
specs/010-gnome-extension/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml                 # single crate `rstory` (ADR-0001), edition 2024, GPL-3.0-or-later
rust-toolchain.toml        # pinned toolchain (AA3)
deny.toml                  # cargo-deny: bans (F3), licenses (F4), advisories
src/
├── main.rs                # clap entry → subcommand dispatch
├── lib.rs                 # module tree; #![forbid(unsafe_code)]
├── domain/                # pure: entry, ids, history rules, filter decisions, pause  (001, 005)
├── store/                 # index.json + content/, locks, perms, read-only API          (001)
├── config/                # config.toml load/validate/defaults                          (001, 006)
├── session/               # detection + backend selection (BR5b)                        (002)
├── backend/
│   ├── mod.rs             # Backend trait + events                                      (002)
│   ├── x11/               # XFIXES capture, serve, INCR                                 (002)
│   ├── wayland/           # ext/zwlr data-control capture, serve                        (003)
│   └── fake.rs            # test double (cfg(test) + feature "testing")
├── ipc/                   # protocol v1 types, codec, client, server, peercred          (004)
├── daemon/                # event loop, pipeline, restore, timers, state, counters      (004, 005)
├── tray/                  # ksni SNI item + menu model                                  (009)
├── cli/                   # commands, output, exit codes                                (006)
├── picker/                # ratatui app, state, render, keymap                          (007)
├── launch/                # picker_command + terminal table                             (008)
├── doctor/                # checks + renderers                                          (011)
└── install/               # autostart.rs (012), gnome.rs (010)
extension/                 # GNOME Shell ESM extension, embedded via include_dir         (010)
tests/
├── contract/              # protocol + CLI JSON schema tests, shared fixtures           (004, 006, 010)
├── integration/           # Xvfb, headless sway, bridge mode                            (002, 003, 004)
├── fitness/               # F1 boundaries, F9 perms, F11 canary, F10 idle               (all)
└── fixtures/protocol/     # NDJSON fixtures shared with the extension's JS tests
benches/                   # picker first frame (F6), ls, capture latency (F7)
docs/release.md            # first-release runbook + Tier-1 manual matrix                (013)
.github/workflows/         # ci.yml, release.yml                                         (013, devsecops:pipeline)
```

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(010)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
