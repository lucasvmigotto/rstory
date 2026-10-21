# Implementation Plan: Command-line interface

**Branch**: `feat/cli` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-cli/spec.md`

## Summary

clap-derive command tree for every subcommand, a protocol client (sync, small) with read-only store fallback for `ls`/`show`, text and `--json` renderers validated against `contracts/cli.md`, stable exit codes, confirmation prompts for `clear`/`clear --all`, and shell completions.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: clap 4 (`derive`), clap_complete 4, serde_json, ipc client (004), store read-only API (001); dev: assert_cmd 2, insta 1

**Storage**: read-only fallback via 001

**Testing**: assert_cmd end-to-end against a daemon with fake backend, JSON schema tests, exit-code matrix, insta snapshots of text output (color and NO_COLOR)

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: `ls` < 50 ms with 500 entries (socket and read-only)

**Constraints**: stdout machine output only; stderr diagnostics

**Scale/Scope**: —

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | clap only |
| II. Tests required per story | ✅ | assert_cmd + schema tests first |
| III. Privacy by default | ✅ | previews escaped; no content in errors |
| IV. Single writer, versioned protocol | ✅ | writes via socket; read-only fallback only for ls/show (F1) |
| V. Budgets stated up front | ✅ | ls budget bench |
| VI. CLI parity and scriptability | ✅ | parity table + JSON schemas + exit codes (core of VI) |
| VII. Accessible interfaces | ✅ | glyph + text, NO_COLOR |
| VIII. Diagnosable, honest failures | ✅ | daemon-down hint on every write command |
| IX. Versioning and review | ✅ | JSON `schema` field |

## Project Structure

### Documentation (this feature)

```text
specs/006-cli/
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

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(006)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
