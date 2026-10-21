---
description: "Task list for Terminal picker"
---

# Tasks: Terminal picker

**Input**: Design documents from `/specs/007-picker/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [ ] T001 [P] Add ratatui/crossterm/nucleo-matcher deps and insta snapshot config — Executor: builder · Reviewer: thinker · Effort: low

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T002 Define `PickerState`, `Msg`, `Cmd` and pure `update()` skeleton in `src/picker/state.rs` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Find and restore a recent copy (Priority: P1) 🎯 MVP

**Independent Test**: Keystrokes `reb` + Enter → restore(A), exit 0

### Tests for User Story 1 ⚠️

- [ ] T003 [P] [US1] journey tests on `update()` (filter, rank, select, Enter → Restore, Esc 1, Ctrl-C 130) in `src/picker/tests.rs` — Executor: builder · Reviewer: thinker
- [ ] T004 [P] [US1] snapshots: normal, filtered, preview on/off at 80×24 and 40×10 — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T005 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Manage entries from the picker (Priority: P2)

**Independent Test**: Ctrl-T pin/unpin, Ctrl-D confirm delete

### Tests for User Story 2 ⚠️

- [ ] T006 [P] [US2] journey tests for pin toggle, pin_limit status line, delete confirm y/n — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T007 [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - No color, no daemon, screen readers (Priority: P2)

**Independent Test**: Plain/NO_COLOR snapshots; daemon-down Enter exits 3

### Tests for User Story 3 ⚠️

- [ ] T008 [P] [US3] snapshots plain + NO_COLOR + read-only banner + empty + too small (SC-003) — Executor: builder · Reviewer: thinker
- [ ] T009 [P] [US3] terminal-restore-on-panic test — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [ ] T010 [US3] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [ ] T011 [P] `benches/picker_first_frame.rs` 20 and 500 entries (F6, SC-001/002) — Executor: builder · Reviewer: thinker
- [ ] T012 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T013 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
