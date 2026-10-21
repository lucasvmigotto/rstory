---
description: "Task list for Release & publishing"
---

# Tasks: Release & publishing

**Input**: Design documents from `/specs/013-release/`

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

- [ ] T001 Decide job graph with `devsecops:pipeline` (CI + release), environments (`release` protected), runners (x86_64, arm64) — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Every change is checked on Tier-1 sessions (Priority: P1) 🎯 MVP

**Independent Test**: A PR breaking a module boundary fails F1

### Tests for User Story 1 ⚠️

- [ ] T002 [US1] CI workflow validation: actionlint + a deliberate-failure PR checklist in `docs/release.md` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T003 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker
- [ ] T004 [US1] QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - A tag publishes everything without a stored token (Priority: P1)

**Independent Test**: workflow_dispatch dry_run builds artifacts + publish --dry-run

### Tests for User Story 2 ⚠️

- [ ] T005 [US2] release dry-run job + assertion step: no `secrets.*` publishing token referenced (SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T006 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - First release by hand, then token retirement (Priority: P2)

**Independent Test**: Runbook steps executable in order

### Tests for User Story 3 ⚠️

- [ ] T007 [US3] `docs/release.md` runbook review against brief K9 and ADR-0009 — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [ ] T008 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [ ] T009 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T010 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
