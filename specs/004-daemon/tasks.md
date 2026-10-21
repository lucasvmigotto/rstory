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

- [ ] T001 [P] Create shared protocol fixtures (one NDJSON file per op/event, valid + invalid) in `tests/fixtures/protocol/` — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T002 Implement protocol v1 types + codec (serde, line framing, 2 MiB limit) in `src/ipc/protocol.rs`, `src/ipc/codec.rs` with schema-validation tests against `/contracts/protocol.schema.json` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Record the clipboard in the background (Priority: P1) 🎯 MVP

**Independent Test**: Daemon + FakeBackend: 3 changes → 3 entries + 3 events

### Tests for User Story 1 ⚠️

- [ ] T003 [P] [US1] contract tests: every fixture validates; server responses validate (SC-004) in `tests/contract/protocol.rs` — Executor: builder · Reviewer: thinker
- [ ] T004 [P] [US1] integration: single instance (exit 4), inactive state, capture→list < 100 ms p95 (F7), burst 500/s bounded + `dropped_burst` in `tests/daemon_capture.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T005 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Serve a restore (Priority: P1)

**Independent Test**: restore{id} → serve called before ok; self_owned → entry moves to top

### Tests for User Story 2 ⚠️

- [ ] T006 [P] [US2] restore tests (ok after ownership, not_found, inactive, self_owned → top) in `tests/daemon_restore.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T007 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Pause and timed pause (Priority: P1)

**Independent Test**: Paused time: pause{for:50s} skips then resumes; survives restart

### Tests for User Story 3 ⚠️

- [ ] T008 [P] [US3] pause tests with tokio paused clock (indefinite, timed, resume early, persisted across restart, past end resumes) in `tests/daemon_pause.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [ ] T009 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: User Story 4 - Status and housekeeping (Priority: P2)

**Independent Test**: status shape; exactly one armed timer; expiry on time

### Tests for User Story 4 ⚠️

- [ ] T010 [P] [US4] status schema + single-timer + idle wake-up (F10) tests in `tests/daemon_status.rs` — Executor: builder · Reviewer: thinker
- [ ] T011 [P] [US4] peercred rejection, malformed line, oversize line, stale socket, SIGTERM cleanup tests in `tests/daemon_socket.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 4

- [ ] T012 [US4] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 4 passes its tests independently.

---

## Phase 7: Polish & Cross-Cutting

- [ ] T013 RSS check after 10 000 captures (SC-002) in `tests/daemon_rss.rs` — Executor: builder · Reviewer: thinker
- [ ] T014 [P] F11 canary: capture a canary at RUST_LOG=trace; assert absent from logs in `tests/fitness/canary.rs` — Executor: builder · Reviewer: thinker
- [ ] T015 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T016 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
