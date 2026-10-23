---
description: "Task list for Command-line interface"
---

# Tasks: Command-line interface

**Input**: Design documents from `/specs/006-cli/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [x] T001 [P] Add `assert_cmd`/`insta` dev-deps and a test harness that starts a daemon with FakeBackend on a temp runtime dir in `tests/support/daemon.rs` — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T002 Define `ExitCode` enum and `EntryRef` parser per `/contracts/cli.md` in `src/cli/mod.rs` with unit tests — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Browse and restore from the shell (Priority: P1) 🎯 MVP

**Independent Test**: ls/show/copy against a test daemon

### Tests for User Story 1 ⚠️

- [x] T003 [P] [US1] assert_cmd tests: ls text (insta), ls --json schema, show raw bytes, copy, unknown/ambiguous ref exit 2 in `tests/cli_browse.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T004 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker
- [x] T005 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - Manage the history (Priority: P1)

**Independent Test**: pin/unpin/rm/clear/clear --all/pause --for/resume with exit codes

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] assert_cmd tests incl. pin_limit exit 6, clear prompt/--yes/no-TTY, clear --all prompt, pause without daemon exit 3 in `tests/cli_manage.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T007 [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Read while the daemon is down (Priority: P2)

**Independent Test**: No daemon: ls/show work, copy exit 3

### Tests for User Story 3 ⚠️

- [x] T008 [P] [US3] read-only fallback tests in `tests/cli_readonly.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [x] T009 [US3] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: User Story 4 - Status and configuration (Priority: P2)

**Independent Test**: status --json schema; config defaults/--path

### Tests for User Story 4 ⚠️

- [ ] T010 [P] [US4] status/config tests in `tests/cli_status.rs`; exit-code coverage matrix (SC-003) — Executor: builder · Reviewer: thinker

### Implementation for User Story 4

- [ ] T011 [US4] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 4 passes its tests independently.

---

## Phase 7: Polish & Cross-Cutting

- [ ] T012 [P] `benches/ls.rs`: < 50 ms at 500 entries (SC-001) — Executor: builder · Reviewer: thinker
- [ ] T013 Shell completions smoke test — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T014 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T015 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

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


### Backend US1 — Browse and restore from the shell

- [x] T016 [US1] Blocking protocol client with timeouts and typed errors in `src/ipc/client.rs` — Executor: builder · Reviewer: thinker
- [x] T017 [US1] Read-only fallback adapter (`ls`/`show`) in `src/cli/source.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

---

## Frontend

Appended by `frontend:spec`. `[TUI]`/`[CLI]` tasks built by `frontend:tui` (ticks only this section). Depends on `000-design-system`. Snapshot and journey tests first.


### Frontend Foundational

- [x] T018 [CLI] clap command tree for every subcommand with help text and examples (`src/cli/args.rs`); `rstory completions` via clap_complete — Executor: builder · Reviewer: thinker

**Checkpoint**: unit tests green.

### Frontend US1 — Browse and restore from the shell

- [x] T019 [US1] [CLI] snapshots first, then `ls` renderer (TTY width-aware, piped TSV) and `show` raw output in `src/cli/render.rs` using `EntryLine` — Executor: builder · Reviewer: thinker
- [x] T020 [US1] [CLI] `--json` serializers per `/contracts/cli.md` with `schema: 1` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US2 — Manage the history

- [x] T021 [US2] [CLI] pin/unpin/rm/clear/clear --all/pause/resume commands with prompts (TTY, /dev/tty, --yes) and copy-keyed messages in `src/cli/commands.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US2 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US3 — Read while the daemon is down

- [x] T022 [US3] [CLI] read-only fallback messaging (`cli.readonly`) and exit 3 for write commands — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the US3 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US4 — Status and configuration

- [ ] T023 [US4] [CLI] `status` text renderer + `config [--path]` in `src/cli/status.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the US4 snapshot/journey tests pass; `cargo insta review` is clean.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [ ] T024 `tests/e2e/E2E-K6.rs` (scripting, non-TTY) — Executor: builder · Reviewer: thinker

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.
