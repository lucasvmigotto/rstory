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

- [x] T001 [P] Add ratatui/crossterm/nucleo-matcher deps and insta snapshot config — Executor: builder · Reviewer: thinker · Effort: low

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T002 Define `PickerState`, `Msg`, `Cmd` and pure `update()` skeleton in `src/picker/state.rs` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Find and restore a recent copy (Priority: P1) 🎯 MVP

**Independent Test**: Keystrokes `reb` + Enter → restore(A), exit 0

### Tests for User Story 1 ⚠️

- [x] T003 [P] [US1] journey tests on `update()` (filter, rank, select, Enter → Restore, Esc 1, Ctrl-C 130) in `src/picker/tests.rs` — Executor: builder · Reviewer: thinker
- [x] T004 [P] [US1] snapshots: normal, filtered, preview on/off at 80×24 and 40×10 — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T005 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Manage entries from the picker (Priority: P2)

**Independent Test**: Ctrl-T pin/unpin, Ctrl-D confirm delete

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] journey tests for pin toggle, pin_limit status line, delete confirm y/n — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T007 [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - No color, no daemon, screen readers (Priority: P2)

**Independent Test**: Plain/NO_COLOR snapshots; daemon-down Enter exits 3

### Tests for User Story 3 ⚠️

- [x] T008 [P] [US3] snapshots plain + NO_COLOR + read-only banner + empty + too small (SC-003) — Executor: builder · Reviewer: thinker
- [x] T009 [P] [US3] terminal-restore-on-panic test — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [x] T010 [US3] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [x] T011 [P] `benches/picker_first_frame.rs` 20 and 500 entries (F6, SC-001/002) — Executor: builder · Reviewer: thinker
- [ ] T012 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [x] T013 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.

---

## Frontend

Appended by `frontend:spec`. `[TUI]`/`[CLI]` tasks built by `frontend:tui` (ticks only this section). Depends on `000-design-system`. Snapshot and journey tests first.


### Frontend Foundational

- [x] T014 [TUI] terminal lifecycle: init/restore, panic hook, SIGTERM/SIGHUP restore, TTY check (exit 2) in `src/picker/term.rs` — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: unit tests green.

### Frontend US1 — Find and restore a recent copy

- [x] T015 [US1] [TUI] `update()` for query editing, filtering (nucleo), ranking, selection-by-id, Enter → Restore in `src/picker/state.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T016 [US1] [TUI] layout + render (QueryLine, list of EntryRow, PreviewPane, Footer) with responsive splits in `src/picker/view.rs` — Executor: builder · Reviewer: thinker
- [x] T017 [US1] [TUI] data: initial list (socket or read-only), debounced `show` for the preview, `subscribe` reader thread — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US2 — Manage entries from the picker

- [x] T018 [US2] [TUI] Ctrl-T pin toggle, Ctrl-D confirm mode, pin-limit status line — Executor: builder · Reviewer: thinker

**Checkpoint**: the US2 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US3 — No color, no daemon, screen readers

- [x] T019 [US3] [TUI] `--plain` inline renderer, NO_COLOR/ASCII, read-only banner and Enter → exit 3, empty and too-small states — Executor: builder · Reviewer: thinker

**Checkpoint**: the US3 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend Polish

- [x] T020 [TUI] first-frame optimization against F6 (load before draw, no alt-screen flicker); F1 keymap footer — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: budgets met; `/speckit-converge` (Frontend) reports converged.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [x] T021 `tests/e2e/E2E-K2.rs` (keyboard-only recall through a pty, X11 and sway; negative read-only path) — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.

---

## Convergence (Frontend) — 2026-10-23

Converged. `rstory pick` maps to `src/picker/{state,app,data,term,mod}.rs`: a pure reducer `update(&mut PickerState, Msg) -> Vec<Cmd>`, the daemon source with a read-only fallback and subscribe, and terminal lifecycle with restore on panic and on SIGHUP, SIGTERM and SIGINT. The picker never depends on `cli`, `backend` or `daemon` (F1). 57 unit tests (29 insta snapshots), `tests/picker_{data,cli,restore}.rs`. `fmt-check`, `lint` and `test` are green. `picker_first_frame` p95: 6.7 ms read-only and 10.2 ms over the socket at 500 entries; keystroke to redraw 3.9 ms against a 16 ms budget. Review fix: the pin-limit status line now shows the daemon's limit (copy key `picker.pin.limit` takes `{max}`, ux-vision updated). Readings the spec left open: plain mode marks pins `[pinned]` (US3 and ux-vision) where FR-005 says `[p]`; fuzzy matching runs on the ≤256-character preview; the theme stays `terminal` until the config is read. Unverified until QA: a real TTY run (alternate screen, `--plain` inline viewport, resizing, key reading). T012 (quickstart on a real terminal) and the QA tasks (T013, T021) remain.
