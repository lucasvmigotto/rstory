# Implementation Plan: SNI tray

**Branch**: `feat/tray` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/009-tray/spec.md`

## Summary

A ksni StatusNotifierItem inside the daemon with pixel-data icons (normal/paused, several sizes, embedded PNG decoded at build time into ARGB), a pure menu model built from (top 10, state, picker_available), confirm submenu for clear, and watcher tracking.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: ksni 0.3 (zbus tokio backend, `default-features=false`), zbus 6 (`tokio`), build-time: png decoding in `build.rs` via the `png` crate (pure Rust) to generate ARGB arrays

**Storage**: N/A

**Testing**: Menu-model unit tests (all states), D-Bus integration test with `dbus-daemon --session` + stub watcher in the tools container, manual KDE/waybar check

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: Menu rebuild < 1 ms

**Constraints**: Labels always textual; paused icon differs in shape

**Scale/Scope**: 10 menu entries

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | ksni/zbus pure Rust; png at build time only |
| II. Tests required per story | ✅ | menu-model tests first |
| III. Privacy by default | ✅ | previews escaped/truncated; no logging |
| IV. Single writer, versioned protocol | ✅ | in-process daemon API |
| V. Budgets stated up front | N/A | — |
| VI. CLI parity and scriptability | ✅ | every item maps to CLI (parity table) |
| VII. Accessible interfaces | ✅ | text labels, shape-distinct paused icon |
| VIII. Diagnosable, honest failures | ✅ | `status.tray` reason |
| IX. Versioning and review | N/A | — |

## Project Structure

### Documentation (this feature)

```text
specs/009-tray/
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

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(009)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
