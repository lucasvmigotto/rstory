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

- [x] T001 [P] Create `extension/` skeleton (`metadata.json` with uuid + shell-version 48–51, `extension.js`, `stylesheet.css`, `icons/`), eslint config, and the type-check setup: `extension/package.json` (dev-only `typescript` 7.0.2, `@girs/gnome-shell` 50.0.4, lockfile), `extension/jsconfig.json` (`checkJs`, `noEmit`, strict), `// @ts-check` in every module, Node + npm in the tools image, `make ext-deps` (networked `npm ci` into a cache volume) and `make ext-typecheck` (offline) — Executor: builder · Reviewer: thinker · Effort: low

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T002 JS protocol client module (`extension/client.js`): connect, NDJSON read loop, request ids, reconnect backoff 1→30 s — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Install the extension from the binary (Priority: P1) 🎯 MVP

**Independent Test**: Temp XDG_DATA_HOME: files written, metadata correct, messages

### Tests for User Story 1 ⚠️

- [x] T003 [P] [US1] install/reinstall/uninstall/--enable (stub gnome-extensions) tests in `tests/install_gnome.rs` (SC-003) — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T004 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Panel icon and menu on GNOME (Priority: P1)

**Independent Test**: JS client replays fixtures → expected menu model; manual Shell 48–51

### Tests for User Story 2 ⚠️

- [x] T005 [P] [US2] gjs fixture test of client + menu model in `extension/tests/client.test.js` (SC-002) — Executor: builder · Reviewer: thinker
- [ ] T006 [US2] Manual matrix procedure for Shell 48–51 in `docs/release.md` (SC-001) — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T007 [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: Polish & Cross-Cutting

- [ ] T008 eslint + GNOME review rules check (disable() cleanup) and `make ext-typecheck` in CI — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T009 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T010 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

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

- [x] T011 [TUI] JS menu model module (`extension/menu.js`) per `specs/009-tray/contracts/menu.md` + Disconnected state — Executor: builder · Reviewer: thinker

**Checkpoint**: unit tests green.

### Frontend US1 — Install the extension from the binary

- [x] T012 [US1] [CLI] `install-gnome-extension` (embed via include_dir, write, `--enable`, `--uninstall`, version compare) in `src/install/gnome.rs` with tui.md messages — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US2 — Panel icon and menu on GNOME

- [x] T013 [US2] [TUI] `extension.js`: PanelMenu.Button, symbolic icons per state, menu from the model, actions → client requests, full `disable()` cleanup — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the US2 snapshot/journey tests pass; `cargo insta review` is clean.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [ ] T014 GNOME Shell 48–51 manual matrix + leak-check procedure in `docs/release.md` — Executor: builder · Reviewer: thinker
- [ ] T015 Charter session per Shell version → `docs/product/qa/charters/010.md` — Executor: thinker · Reviewer: reviewer · Effort: high

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.
