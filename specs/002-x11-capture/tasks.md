---
description: "Task list for Session detection & X11 capture"
---

# Tasks: Session detection & X11 capture

**Input**: Design documents from `/specs/002-x11-capture/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [ ] T001 [P] Add test fixtures: Xvfb launcher helper and a minimal X11 test client in `tests/support/x11.rs` — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T002 [P] Add headless-sway-with-XWayland launcher (data-control disabled) in `tests/support/sway.rs` + `tests/fixtures/sway-bridge.conf` — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T003 Define `Backend` trait, `BackendEvent`, errors and a `FakeBackend` in `src/backend/mod.rs`, `src/backend/fake.rs` per `contracts/backend.md` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Capture on X11 (Priority: P1) 🎯 MVP

**Independent Test**: Under Xvfb an X11 client sets CLIPBOARD; one event with types and correct text

### Tests for User Story 1 ⚠️

- [ ] T004 [P] [US1] Xvfb capture tests (UTF8_STRING, STRING→UTF-8, INCR 900 KiB, owner exit → cleared) in `tests/x11_capture.rs` (SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T005 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Restore on X11 (Priority: P1)

**Independent Test**: Backend serves text; another client reads it; TARGETS correct; self_owned flagged

### Tests for User Story 2 ⚠️

- [ ] T006 [P] [US2] Xvfb serve tests (targets, 1 MiB INCR round trip, ownership loss, self_owned) in `tests/x11_serve.rs` (SC-002) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T007 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Automatic backend selection incl. XWayland bridge (Priority: P1)

**Independent Test**: Table-driven selection + bridge-mode capture under headless sway

### Tests for User Story 3 ⚠️

- [ ] T008 [P] [US3] selection table tests (≥ 8 rows incl. override, no_session, no_data_control_no_xwayland) in `src/session/tests.rs` (SC-003) — Executor: builder · Reviewer: thinker
- [ ] T009 [P] [US3] bridge-mode capture/restore test in `tests/bridge_capture.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [ ] T010 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [ ] T011 Record GNOME 48–51 manual check steps (GUI-app copies, brief Q7) in `docs/release.md` draft — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T012 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T013 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
