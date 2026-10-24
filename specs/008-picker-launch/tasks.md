---
description: "Task list for Picker launch"
---

# Tasks: Picker launch

**Input**: Design documents from `/specs/008-picker-launch/`

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

- [x] T001 Define `TERMINALS` table and `LaunchPlan` in `src/launch/mod.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Open the picker from a menu (Priority: P1) 🎯 MVP

**Independent Test**: Fake PATH with kitty → `kitty rstory pick` spawned; none → no_terminal

### Tests for User Story 1 ⚠️

- [x] T002 [P] [US1] resolution-order tests (config › $TERMINAL › xdg-terminal-exec › list) and one argv test per table row (SC-001) in `src/launch/tests.rs` — Executor: builder · Reviewer: thinker
- [x] T003 [P] [US1] protocol test: open_picker ok / no_terminal; status.picker_available — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T004 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: Polish & Cross-Cutting

- [x] T005 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [x] T006 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

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


### Backend US1 — Open the picker from a menu

- [x] T007 [US1] `launch::plan` resolution + `TERMINALS` table in `src/launch/mod.rs` — Executor: builder · Reviewer: thinker
- [x] T008 [US1] `launch::spawn` detached (setsid, null stdio) + `open_picker` handler + `picker_available` in status — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [x] T009 Verify the launch table against the terminals installed in the tools container (smoke) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.

---

## Convergence — 2026-10-23

Converged. FR-001–FR-003 map to `src/launch/mod.rs` (resolution order and launch table, PR #57), `src/config/mod.rs` (`picker_command_from_toml`) and the daemon's `open_picker` handler and `status.picker_available` (`src/daemon/handlers.rs`, landed with 004). Tests cover the configured command, `$TERMINAL`, `xdg-terminal-exec` and the fixed list, `no_terminal`, and `picker_command_failed` naming only the program. No test starts a terminal from the host: spawns use `/bin/true`, a fake script or a missing path. The QA tasks (T006, T009) are deferred to before Verified.
