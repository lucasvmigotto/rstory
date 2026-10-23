---
description: "Task list for SNI tray"
---

# Tasks: SNI tray

**Input**: Design documents from `/specs/009-tray/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [x] T001 [P] Add icon sources `assets/icons/{normal,paused}.svg` + rendered PNGs and `build.rs` ARGB generation — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T002 Define pure `TrayMenu` model per `contracts/menu.md` in `src/tray/menu.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Restore from the top bar (Priority: P1) 🎯 MVP

**Independent Test**: Menu model from 12 entries → 10 items; activate #3 → restore

### Tests for User Story 1 ⚠️

- [x] T003 [P] [US1] menu-model unit tests (labels, truncation, ⏎, pin glyph, order) in `src/tray/tests.rs` — Executor: builder · Reviewer: thinker
- [x] T004 [P] [US1] D-Bus integration with stub watcher: item registers, activate → restore in `tests/tray_dbus.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T005 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Pause, timed pause, open picker, clear (Priority: P1)

**Independent Test**: Model per state; clear goes through confirm

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] model tests for recording/paused/timed/no-terminal/confirming (SC-002) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T007 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - No SNI host (Priority: P3)

**Independent Test**: No watcher → status.tray=no_host; watcher appears → registers

### Tests for User Story 3 ⚠️

- [x] T008 [P] [US3] watcher absent/appears tests in `tests/tray_dbus.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [x] T009 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [x] T010 Manual KDE + waybar check entry in `docs/release.md` (SC-001) — Executor: builder · Reviewer: thinker · Effort: low
- [x] T011 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T012 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

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

- [x] T013 Add ksni (tokio), zbus (tokio, no default features), png (build-dep); `build.rs` icon ARGB generation — Executor: builder · Reviewer: thinker

**Checkpoint**: `cargo build` and `cargo test` green; setup pieces covered by unit tests.

### Backend US1 — Restore from the top bar

- [x] T014 [US1] ksni `Tray` impl: icon pixmaps, tooltip, entry items → restore in `src/tray/mod.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US2 — Pause, timed pause, open picker, clear

- [x] T015 [US2] Pause/Resume/Pause 1 min/Open picker/Clear-confirm items wired to daemon handlers — Executor: builder · Reviewer: thinker

**Checkpoint**: the US2 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US3 — No SNI host

- [x] T016 [US3] Watcher tracking, `status.tray` states, config `tray = false` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US3 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

---

## Frontend

Appended by `frontend:spec`. `[TUI]`/`[CLI]` tasks built by `frontend:tui` (ticks only this section). Depends on `000-design-system`. Snapshot and journey tests first.


### Frontend US1 — Restore from the top bar

- [x] T017 [US1] Icon pixmaps from `assets/icons/png` (000-design-system T010) wired into `build.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the US1 snapshot/journey tests pass; `cargo insta review` is clean.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [ ] T018 `tests/e2e/E2E-K7.rs` (dbus-daemon + stub watcher) — Executor: builder · Reviewer: thinker
- [ ] T019 KDE + waybar manual rows in `docs/release.md` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.
