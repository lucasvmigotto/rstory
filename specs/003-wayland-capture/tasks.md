---
description: "Task list for Wayland data-control capture"
---

# Tasks: Wayland data-control capture

**Input**: Design documents from `/specs/003-wayland-capture/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [ ] T001 [P] Add Wayland test client (sets/reads selection via data-control) in `tests/support/wl_client.rs` — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T002 Verify wayland-protocols exposes `ext-data-control-v1`; else vendor XML under `protocols/` with `wayland-scanner` (research R2) — Executor: scout · Reviewer: thinker · Effort: low

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Capture on data-control compositors (Priority: P1) 🎯 MVP

**Independent Test**: Headless sway: client sets selection; one event; text read via pipe

### Tests for User Story 1 ⚠️

- [ ] T003 [P] [US1] sway capture tests (ext path, zwlr fallback via protocol filter, hint present in types, clear) in `tests/wayland_capture.rs` (SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T004 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Restore on data-control compositors (Priority: P1)

**Independent Test**: Backend serves; separate client reads; cancelled stops serving

### Tests for User Story 2 ⚠️

- [ ] T005 [P] [US2] sway serve tests (MIME set, 1 MiB round trip, cancelled, self_owned, EPIPE tolerance) in `tests/wayland_serve.rs` (SC-002) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T006 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: Polish & Cross-Cutting

- [ ] T007 Add KDE Plasma manual check to `docs/release.md` (SC-003) — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T008 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T009 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

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

- [ ] T010 Protocol bindings (wayland-protocols `staging` or vendored XML) in `src/backend/wayland/protocols.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: `cargo build` and `cargo test` green; foundational pieces covered by unit tests.

### Backend US1 — Capture on data-control compositors

- [ ] T011 [US1] Connect + bind + offer tracking + read with limit/timeout in `src/backend/wayland/{mod,read}.rs` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US2 — Restore on data-control compositors

- [ ] T012 [US2] Source/serve with async `send` handling and `cancelled` in `src/backend/wayland/serve.rs` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the US2 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend Polish

- [ ] T013 KDE Plasma manual check entry in `docs/release.md` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the budgets in backend.md are met; `/speckit-converge` (Backend) reports converged.
