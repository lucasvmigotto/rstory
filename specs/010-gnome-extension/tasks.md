---
description: "Task list for GNOME Shell extension"
---

# Tasks: GNOME Shell extension

**Input**: Design documents from `/specs/010-gnome-extension/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [ ] T001 [P] Create `extension/` skeleton (`metadata.json` with uuid + shell-version 48–51, `extension.js`, `stylesheet.css`, `icons/`) and eslint config — Executor: builder · Reviewer: thinker · Effort: low

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T002 JS protocol client module (`extension/client.js`): connect, NDJSON read loop, request ids, reconnect backoff 1→30 s — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Install the extension from the binary (Priority: P1) 🎯 MVP

**Independent Test**: Temp XDG_DATA_HOME: files written, metadata correct, messages

### Tests for User Story 1 ⚠️

- [ ] T003 [P] [US1] install/reinstall/uninstall/--enable (stub gnome-extensions) tests in `tests/install_gnome.rs` (SC-003) — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T004 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Panel icon and menu on GNOME (Priority: P1)

**Independent Test**: JS client replays fixtures → expected menu model; manual Shell 48–51

### Tests for User Story 2 ⚠️

- [ ] T005 [P] [US2] gjs fixture test of client + menu model in `extension/tests/client.test.js` (SC-002) — Executor: builder · Reviewer: thinker
- [ ] T006 [US2] Manual matrix procedure for Shell 48–51 in `docs/release.md` (SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T007 [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: Polish & Cross-Cutting

- [ ] T008 eslint + GNOME review rules check (disable() cleanup) in CI — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T009 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T010 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
