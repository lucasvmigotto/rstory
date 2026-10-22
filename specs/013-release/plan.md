# Implementation Plan: Release & publishing

**Branch**: `ci/release` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/013-release/spec.md`

## Summary

CI (PR/push) and release (tag) workflows on GitHub Actions per ADR-0009, native x86_64/arm64 runners building musl static binaries, integration jobs on headless sway/Xvfb/bridge mode, fitness checks, `cargo deny`; tag-driven Trusted Publishing to crates.io in a protected `release` environment, then GitHub release with tarballs + SHA256SUMS; `docs/release.md` runbook and Tier-1 manual matrix.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: GitHub Actions; `rust-lang/crates-io-auth-action`; cargo-deny; (pinning/hardening by `devsecops:supply-chain`)

**Storage**: N/A

**Testing**: Workflow dry-run (`workflow_dispatch` dry_run), `actionlint`, F2/F5 checks inside the workflow

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: PR CI < 15 min

**Constraints**: No stored publishing token; publish before release creation; tags only from main

**Scale/Scope**: —

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | F2/F3 enforced in CI |
| II. Tests required per story | ✅ | CI runs all suites |
| III. Privacy by default | ✅ | F11 in CI |
| IV. Single writer, versioned protocol | ✅ | contract tests in CI |
| V. Budgets stated up front | ✅ | F5/F6/F7 in CI |
| VI. CLI parity and scriptability | N/A | — |
| VII. Accessible interfaces | N/A | — |
| VIII. Diagnosable, honest failures | N/A | — |
| IX. Versioning and review | ✅ | core of IX: SemVer tags, Trusted Publishing |

## Project Structure

### Documentation (this feature)

```text
specs/013-release/
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

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(013)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
