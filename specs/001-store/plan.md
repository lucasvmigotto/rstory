# Implementation Plan: Store & history

**Branch**: `feat/store` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-store/spec.md`

## Summary

A pure `domain` module holds the history rules (de-dup by BLAKE3, the 20-entry count cap with pins counted but never evicted, the 10-pin cap, 7-day expiry, clear and clear-all). A `store` module persists them as `index.json` plus write-once `content/<id>` files using temp + fsync + rename, enforces `0700`/`0600`, sweeps orphans on open, and offers an exclusive (daemon) and a shared read-only (client) handle.

## Technical Context

**Language/Version**: Rust, edition 2024; MSRV = stable from 6 months before v0.1 (architecture AA3); toolchain pinned in `rust-toolchain.toml`

**Primary Dependencies**: serde 1, serde_json 1, blake3 1 (`default-features=false, features=["pure"]`), rustix 1 (`fs` for flock, fchmod, fsync), time 0.3 (RFC 3339), rand 0.9 (id entropy); dev: proptest 1, tempfile 3

**Storage**: Files under `$XDG_STATE_HOME/rstory` (ADR-0003)

**Testing**: Unit and proptest (invariants, crash injection through a fault-injecting fs shim), permission tests (F9)

**Target Platform**: Linux x86_64/aarch64, glibc or musl; Tier-1 sessions: GNOME 48–51 Wayland (bridge), KDE Wayland, sway, X11

**Project Type**: single crate, multi-call CLI/daemon binary (ADR-0001)

**Performance Goals**: Open + list of 500 entries < 10 ms; insert (2 fsync) ≤ 20 ms on SSD; batched insert of N ≤ 20 ms + N × 1 ms (Constitution V)

**Constraints**: No content in logs; real deletion; never rewrite an index with an unknown version

**Scale/Scope**: 20 entries by default, up to 500 configured; ≤ 1 MiB each

**Dev environment**: `devcontainer:setup` provides the tools container (Rust toolchain, Xvfb, headless sway with XWayland, dbus-daemon, gjs). **Delivery**: CI/CD from `devsecops:pipeline` (spec 013); test layers from `qa:strategy`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Result: PASS (pre-research and post-design).**

| Principle | Status | Evidence in this plan |
|---|---|---|
| I. Zero runtime dependencies | ✅ | pure-Rust deps only (blake3 `pure`) |
| II. Tests required per story | ✅ | proptest + unit tests precede implementation in tasks |
| III. Privacy by default | ✅ | perms, real unlink, no content logging (F9, F11) |
| IV. Single writer, versioned protocol | ✅ | exclusive lock for the daemon; read-only handle exposes no write API (F1) |
| V. Budgets stated up front | ✅ | open/insert budgets above, bench in `benches/store.rs` |
| VI. CLI parity and scriptability | N/A | N/A (no UI) |
| VII. Accessible interfaces | N/A | N/A (no UI) |
| VIII. Diagnosable, honest failures | ✅ | `store_too_open` and `unknown_version` errors with hints for doctor |
| IX. Versioning and review | ✅ | index `version` field; format change = ADR |

## Project Structure

### Documentation (this feature)

```text
specs/001-store/
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

**Structure Decision**: single project (ADR-0001). This feature owns the paths annotated `(001)` above; shared modules are touched only through their public API.

## Complexity Tracking

No violations.
