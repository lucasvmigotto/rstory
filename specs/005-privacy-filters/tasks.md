---
description: "Task list for Privacy filters"
---

# Tasks: Privacy filters

**Input**: Design documents from `/specs/005-privacy-filters/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

No feature-specific setup (crate skeleton comes from 001).

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T001 Define `FilterDecision` and `SENSITIVE_HINTS` in `src/domain/filter.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Copies marked sensitive are never stored (Priority: P1) 🎯 MVP

**Independent Test**: Hinted change → SkipSensitive, read never called

### Tests for User Story 1 ⚠️

- [ ] T002 [P] [US1] proptest: random jobs incl. hinted ones; assert hinted bytes never reach store, logs, or post-decision memory (SC-001, F8) in `tests/filter_props.rs` — Executor: builder · Reviewer: thinker
- [ ] T003 [P] [US1] unit: hint with text, hint only, hint via bridge targets — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T004 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Oversize copies are dropped, never truncated (Priority: P1)

**Independent Test**: 1 MiB+1 → SkipOversize; 1 MiB recorded

### Tests for User Story 2 ⚠️

- [ ] T005 [P] [US2] boundary tests cap−1/cap/cap+1 (SC-002) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T006 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Nothing recorded while paused (Priority: P1)

**Independent Test**: Paused → SkipPaused, no read

### Tests for User Story 3 ⚠️

- [ ] T007 [P] [US3] pause-gate unit tests + order-of-checks table test — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [ ] T008 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [ ] T009 Write README sections: threat model, sensitive hints, `passc` wrapper (FR-007) — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T010 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T011 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
