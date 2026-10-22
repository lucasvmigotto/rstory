# Implementation Plan: Session detection & X11 capture (incl. XWayland bridge)

**Branch**: `feat/x11-capture` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-x11-capture/spec.md`

## Summary

Define the `Backend` trait and its event stream, implement the X11 backend on x11rb's pure-Rust connection (XFIXES selection notifications, TARGETS-first reads, INCR, serving as owner), and implement session detection that applies BR5b, including the XWayland-bridge mode used on GNOME.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: x11rb 0.14 (`default-features=false`, features `xfixes`; RustConnection), wayland-client 0.31 (registry probe only, pure-Rust backend), tokio 1 (`AsyncFd`); dev: Xvfb + headless sway in the tools container

**Storage**: N/A

**Testing**: Unit (selection table, target ranking, Latin-1 conversion), integration under Xvfb (capture, serve, INCR 1 MiB), integration under headless mutter with XWayland (bridge mode)

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: Event → text p95 < 20 ms; no polling (Constitution V)

**Constraints**: Read limit enforced mid-transfer; no content in logs

**Scale/Scope**: One selection, ≤ 1 MiB

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | x11rb RustConnection, no libxcb; wayland-client without `client_system` |
| II. Tests required per story | ✅ | Xvfb/sway integration tests written first |
| III. Privacy by default | ✅ | types-only logging; reads honor limit |
| IV. Single writer, versioned protocol | ✅ | backend is used only by the daemon (F1) |
| V. Budgets stated up front | ✅ | event→text budget measured in integration test |
| VI. CLI parity and scriptability | N/A | N/A |
| VII. Accessible interfaces | N/A | N/A |
| VIII. Diagnosable, honest failures | ✅ | selection carries reason (`no_data_control_no_xwayland`, `x11_unreachable`) |
| IX. Versioning and review | ✅ | trait change = spec change |

## Project Structure

### Documentation (this feature)

```text
specs/002-x11-capture/
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

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(002)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
