# Implementation Plan: Daemon & control socket

**Branch**: `feat/daemon` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-daemon/spec.md`

## Summary

`rstory daemon` on a tokio current-thread runtime: single instance (store lock + socket), backend selection, the capture pipeline (bounded queue, batching), restore ownership, pause/timed pause persisted in the index, one-shot timers for expiry and pause end, counters, and the NDJSON protocol v1 server with `SO_PEERCRED` checks and `subscribe` events.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: tokio 1 (`rt`, `net`, `time`, `signal`, `io-util`, `macros`, `sync`, `fs`), serde/serde_json, tracing 0.1 + tracing-subscriber 0.3 (`fmt`, `env-filter`), rustix 1 (`net` for SO_PEERCRED); dev: jsonschema validator crate for contract tests (dev-only)

**Storage**: via spec 001

**Testing**: Unit (state machine, timers with tokio paused time), contract tests (schema-validated fixtures), integration (daemon + fake backend; daemon on Xvfb/sway), burst test, idle wake-up test (F10)

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: Capture-to-listed p95 < 100 ms; RSS ≤ 20 MiB; idle: timers only

**Constraints**: Queue bound 256; line limit 2 MiB; peer uid check

**Scale/Scope**: < 100 requests/day; bursts of 50–500 captures/s

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | tokio + rustix only |
| II. Tests required per story | ✅ | contract + integration tests first |
| III. Privacy by default | ✅ | peercred, 0700 socket dir, no content in logs/events |
| IV. Single writer, versioned protocol | ✅ | the daemon is the single writer; protocol v1 schema is the seam |
| V. Budgets stated up front | ✅ | F7, F10, RSS check |
| VI. CLI parity and scriptability | ✅ | every op exposed to CLI (006) |
| VII. Accessible interfaces | N/A | N/A |
| VIII. Diagnosable, honest failures | ✅ | `inactive` state with reason; counters in status |
| IX. Versioning and review | ✅ | protocol `v` field |

## Project Structure

### Documentation (this feature)

```text
specs/004-daemon/
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

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(004)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
