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

- [x] T001 [P] Add test fixtures: Xvfb launcher helper and a minimal X11 test client in `tests/support/x11.rs` — Executor: builder · Reviewer: thinker · Effort: low
- [x] T002 [P] Add a headless-mutter launcher (`mutter --headless --wayland --virtual-monitor`, private D-Bus, Xauthority from `$XDG_RUNTIME_DIR/.mutter-Xwaylandauth.*`) in `tests/support/mutter.rs` — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T003 Define `Backend` trait, `BackendEvent`, errors and a `FakeBackend` in `src/backend/mod.rs`, `src/backend/fake.rs` per `contracts/backend.md` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Capture on X11 (Priority: P1) 🎯 MVP

**Independent Test**: Under Xvfb an X11 client sets CLIPBOARD; one event with types and correct text

### Tests for User Story 1 ⚠️

- [x] T004 [P] [US1] Xvfb capture tests (UTF8_STRING, STRING→UTF-8, INCR 900 KiB, owner exit → cleared) in `tests/x11_capture.rs` (SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T005 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Restore on X11 (Priority: P1)

**Independent Test**: Backend serves text; another client reads it; TARGETS correct; self_owned flagged

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] Xvfb serve tests (targets, 1 MiB INCR round trip, ownership loss, self_owned) in `tests/x11_serve.rs` (SC-002) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T007 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Automatic backend selection incl. XWayland bridge (Priority: P1)

**Independent Test**: Table-driven selection + bridge-mode selection and X11-side capture under headless mutter

### Tests for User Story 3 ⚠️

- [x] T008 [P] [US3] selection table tests (≥ 8 rows incl. override, no_session, no_data_control_no_xwayland) in `src/session/tests.rs` (SC-003) — Executor: builder · Reviewer: thinker
- [x] T009 [P] [US3] bridge-mode test on headless mutter: selection = x11/xwayland_bridge, and capture/restore between X11 clients on Mutter's XWayland, in `tests/bridge_capture.rs` (cross-protocol copies: manual GNOME matrix) — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [x] T010 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [x] T011 Record GNOME 48–51 manual check steps (GUI-app copies, brief Q7) in `docs/release.md` draft — Executor: builder · Reviewer: thinker · Effort: low
- [x] T012 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [x] T013 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

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


### Backend Foundational

- [x] T014 Backend trait + `FakeBackend` (scriptable events, recorded serves) in `src/backend/{mod,fake}.rs` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: `cargo build` and `cargo test` green; foundational pieces covered by unit tests.

### Backend US1 — Capture on X11

- [x] T015 [US1] X11 connection, XFIXES subscription, TARGETS-first read with limit and timeout in `src/backend/x11/{mod,read}.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T016 [US1] INCR receive in `src/backend/x11/incr.rs` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US2 — Restore on X11

- [x] T017 [US2] Owner/serve state machine with SelectionRequest handling and INCR send in `src/backend/x11/serve.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T018 [US2] `self_owned` detection by owner window id — Executor: builder · Reviewer: thinker

**Checkpoint**: the US2 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US3 — Automatic backend selection incl. XWayland bridge

- [x] T019 [US3] `session::select` + Wayland registry probe (data-control globals) in `src/session/mod.rs` — Executor: builder · Reviewer: thinker
- [x] T020 [US3] Bridge mode: X11 backend with `mode = xwayland_bridge`, connect retry 3 s — Executor: builder · Reviewer: thinker

**Checkpoint**: the US3 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend Polish

- [ ] T021 Document GNOME 48–51 manual check in `docs/release.md` (draft) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the budgets in backend.md are met; `/speckit-converge` (Backend) reports converged.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [x] T022 Bridge-mode CI environment (headless mutter) in `tests/support/mutter.rs` validated by `qa:e2e` — Executor: builder · Reviewer: thinker
- [x] T023 GNOME 48–51 manual matrix rows in `docs/release.md` (GUI-app copies, restore into Wayland apps) — Executor: builder · Reviewer: thinker
- [x] T024 Charter session → `docs/product/qa/charters/002.md` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.

---

## Convergence (Backend) — 2026-10-22

Converged. FR-001–FR-010 map to `src/backend/{mod,fake}.rs`, `src/backend/x11/*` and `src/session/mod.rs`, with session suites on Xvfb, headless sway and headless mutter (434 tests green after the rebase onto dev). Review: a focused thinker review (re-run of every suite, unsafe = 0, no content in logs). The fresh-context review was deferred because of the owner's session budget and should be done before Verified. Open: T013 (QA), the GNOME 48–51 manual matrix (`docs/release.md`), and F2/F3 (`cargo deny` is not in the tools image).

