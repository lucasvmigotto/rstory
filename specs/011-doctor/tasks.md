---
description: "Task list for Doctor"
---

# Tasks: Doctor

**Input**: Design documents from `/specs/011-doctor/`

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

- [ ] T001 Define `Check`, statuses, text/JSON renderers and exit scale in `src/doctor/mod.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - See what works and what's missing (Priority: P1) 🎯 MVP

**Independent Test**: Faked healthy env → all ✓, exit 0

### Tests for User Story 1 ⚠️

- [ ] T002 [P] [US1] healthy-baseline test + JSON schema test in `src/doctor/tests.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T003 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker
- [ ] T004 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Every detectable failure is explained (Priority: P1)

**Independent Test**: 8-row failure table with faked envs

### Tests for User Story 2 ⚠️

- [ ] T005 [P] [US2] table-driven failure tests (8 rows, status/message/hint; SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T006 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: Polish & Cross-Cutting

- [ ] T007 Assert doctor never prints content (canary) — Executor: builder · Reviewer: thinker
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


### Backend US2 — Every detectable failure is explained

- [ ] T010 [US2] Probe adapters with timeouts (session, store, daemon status, extension, shell version, launch, autostart) in `src/doctor/probes.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US2 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

---

## Frontend

Appended by `frontend:spec`. `[TUI]`/`[CLI]` tasks built by `frontend:tui` (ticks only this section). Depends on `000-design-system`. Snapshot and journey tests first.


### Frontend US1 — See what works and what's missing

- [ ] T011 [US1] [CLI] text renderer (CheckLine, summary line) and `--json` in `src/doctor/render.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 snapshot/journey tests pass; `cargo insta review` is clean.
