---
description: "Task list for Daemon & control socket"
---

# Tasks: Daemon & control socket

**Input**: Design documents from `/specs/004-daemon/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [x] T001 [P] Create shared protocol fixtures (one NDJSON file per op/event, valid + invalid) in `tests/fixtures/protocol/` — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T002 Implement protocol v1 types + codec (serde, line framing, 2 MiB limit) in `src/ipc/protocol.rs`, `src/ipc/codec.rs` with schema-validation tests against `/contracts/protocol.schema.json` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Record the clipboard in the background (Priority: P1) 🎯 MVP

**Independent Test**: Daemon + FakeBackend: 3 changes → 3 entries + 3 events

### Tests for User Story 1 ⚠️

- [x] T003 [P] [US1] contract tests: every fixture validates; server responses validate (SC-004) in `tests/contract/protocol.rs` — Executor: builder · Reviewer: thinker
- [x] T004 [P] [US1] integration: single instance (exit 4), inactive state, capture→list < 100 ms p95 (F7), burst 500/s bounded + `dropped_burst` in `tests/daemon_capture.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T005 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Serve a restore (Priority: P1)

**Independent Test**: restore{id} → serve called before ok; self_owned → entry moves to top

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] restore tests (ok after ownership, not_found, inactive, self_owned → top) in `tests/daemon_restore.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T007 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Pause and timed pause (Priority: P1)

**Independent Test**: Paused time: pause{for:50s} skips then resumes; survives restart

### Tests for User Story 3 ⚠️

- [x] T008 [P] [US3] pause tests with tokio paused clock (indefinite, timed, resume early, persisted across restart, past end resumes) in `tests/daemon_pause.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [x] T009 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: User Story 4 - Status and housekeeping (Priority: P2)

**Independent Test**: status shape; exactly one armed timer; expiry on time

### Tests for User Story 4 ⚠️

- [x] T010 [P] [US4] status schema + single-timer + idle wake-up (F10) tests in `tests/daemon_status.rs` — Executor: builder · Reviewer: thinker
- [x] T011 [P] [US4] peercred rejection, malformed line, oversize line, stale socket, SIGTERM cleanup tests in `tests/daemon_socket.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 4

- [x] T012 [US4] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 4 passes its tests independently.

---

## Phase 7: Polish & Cross-Cutting

- [x] T013 RSS check after 10 000 captures (SC-002) in `tests/daemon_rss.rs` — Executor: builder · Reviewer: thinker
- [x] T014 [P] F11 canary: capture a canary at RUST_LOG=trace; assert absent from logs in `tests/fitness/canary.rs` — Executor: builder · Reviewer: thinker
- [ ] T015 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T016 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.

---

## Backend

Appended by `backend:spec`. Built by `backend:build` (ticks only this section). Tests first.


### Backend Setup

- [x] T017 Add tokio (`rt`,`net`,`time`,`signal`,`io-util`,`macros`,`sync`,`fs`), tracing-subscriber, rustix `net`; dev: `jsonschema` validator — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: `cargo build` and `cargo test` green; setup pieces covered by unit tests.

### Backend Foundational

- [x] T018 Socket server: runtime dir resolution, 0700 dir, bind/stale handling, `SO_PEERCRED` check, line codec with 2 MiB limit in `src/ipc/server.rs`, `src/ipc/peercred.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T019 Error → protocol code mapping and correlation ids in `src/ipc/error.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: `cargo build` and `cargo test` green; foundational pieces covered by unit tests.

### Backend US1 — Record the clipboard in the background

- [x] T020 [US1] Daemon main: single instance (store LOCK_EX), backend selection, inactive state, signal handling in `src/daemon/mod.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T021 [US1] Capture pipeline: bounded queue, filter hook (005), batched `Store::apply`, `history_changed` fan-out in `src/daemon/pipeline.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T022 [US1] Handlers: list, show, status, subscribe in `src/daemon/handlers.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US2 — Serve a restore

- [x] T023 [US2] Restore: resolve → touch → backend serve → ack after ownership; self_owned handling in `src/daemon/restore.rs` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the US2 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US3 — Pause and timed pause

- [x] T024 [US3] RecordingState aggregate (pause, timed pause, persisted, resume on load) in `src/domain/pause.rs` + handlers pause/resume — Executor: builder · Reviewer: thinker
- [x] T025 [US3] Deadline timer (`min(next_expiry, paused_until)`) in `src/daemon/timers.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US3 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US4 — Status and housekeeping

- [x] T026 [US4] Counters + status read model; reconnect policy for bridge mode in `src/daemon/state.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US4 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend Polish

- [x] T027 Burst tuning, RSS verification, logging review against F11 — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the budgets in backend.md are met; `/speckit-converge` (Backend) reports converged.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [ ] T028 `tests/e2e/E2E-K1.rs`, `E2E-K4.rs`, `E2E-PEER.rs` (pty/expectrl harness + session launchers in `tests/e2e/support.rs`) — Executor: builder · Reviewer: reviewer · Effort: high
- [ ] T029 Burst and soak harness `tests/perf/{burst,soak}.rs` per the load profile — Executor: builder · Reviewer: reviewer · Effort: high
- [ ] T030 Charter session → `docs/product/qa/charters/004.md` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.

---

## Convergence (Backend) — 2026-10-23

Converged. FR-001–FR-011 map to `src/daemon/{mod,start,pipeline,restore,handlers,state,timers}.rs`, `src/ipc/{protocol,codec,error,server,peercred}.rs` and `src/domain/pause.rs`. Protocol v1 is checked against `contracts/protocol.schema.json` by `tests/contract`. The daemon drives the X11 backend (Xvfb test) and the Wayland backend (headless sway test, added after the rebase onto 003); a Wayland connect failure reports `no_data_control` or `wayland_unreachable` and keeps retrying. `fmt-check`, `lint`, `test` and `test-integration` are green. Review: focused thinker review (unsafe = 0, logs carry decisions, sizes and error kinds only). A terminal-list test that would have spawned a real `kitty` on a host where it is installed now checks resolution only. Caveats: the red commits for US1–US4 are red against a stubbed `Daemon::run`; SC-002 is asserted as growth over 10 000 captures (at most 8 MiB) plus a 64 MiB whole-process bound, not the daemon's absolute 20 MiB; F7 and F10 are checked with the fake backend and tokio's paused clock only; the tray status is always `disabled` until 009. T015 (quickstart run), the QA tasks and the fresh-context review are deferred to before Verified.
