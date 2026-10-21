---
description: "Task list for Store & history"
---

# Tasks: Store & history

**Input**: Design documents from `/specs/001-store/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [ ] T001 Create the crate skeleton: `Cargo.toml` (edition 2024, `license = "GPL-3.0-or-later"`, `rust-version`), `src/main.rs`, `src/lib.rs` with `#![forbid(unsafe_code)]` and the module tree from plan.md — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T002 [P] Add `rust-toolchain.toml` (pinned), `rustfmt.toml`, `clippy.toml`, `deny.toml` (bans F3, licenses F4) — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T003 [P] Add `tests/fitness/boundaries.rs` (F1 module-dependency rules from architecture §5.3) — failing until modules exist — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [ ] T004 [P] Define `Fs` abstraction with real and fault-injecting implementations in `src/store/fs.rs` (used by crash tests) — Executor: builder · Reviewer: thinker
- [ ] T005 [P] Implement `config` caps parsing/validation with defaults (20/10/1 MiB/7d) in `src/config/mod.rs` + unit tests — Executor: builder · Reviewer: thinker

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Recorded entries survive restarts (Priority: P1) 🎯 MVP

**Independent Test**: Insert, crash at random points, reopen: every acknowledged entry present

### Tests for User Story 1 ⚠️

- [ ] T006 [P] [US1] proptest crash-injection suite in `tests/store_crash.rs` (SC-001) — Executor: builder · Reviewer: thinker
- [ ] T007 [P] [US1] permission tests (dir 0700, files 0600, `store_too_open`) in `tests/fitness/perms.rs` (F9) — Executor: builder · Reviewer: thinker
- [ ] T008 [P] [US1] orphan-sweep and unknown-version tests in `tests/store_open.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [ ] T009 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - History stays small and fresh (Priority: P1)

**Independent Test**: Property tests over random op sequences assert caps, de-dup, expiry, pin rules

### Tests for User Story 2 ⚠️

- [ ] T010 [P] [US2] proptest invariants (count cap with pins counted, pin cap, de-dup, expiry, clear vs clear_all) in `tests/history_props.rs` (SC-002) — Executor: builder · Reviewer: thinker
- [ ] T011 [P] [US2] boundary tests: 21st insert evicts oldest unpinned; 11th pin → `pin_limit` in `src/domain/history.rs` tests — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [ ] T012 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: User Story 3 - Clients read while the daemon is down (Priority: P2)

**Independent Test**: Read-only open lists and shows; write APIs absent; refused while daemon holds the lock

### Tests for User Story 3 ⚠️

- [ ] T013 [US3] read-only handle tests (shared lock, `daemon_running`) in `tests/store_readonly.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 3

- [ ] T014 [US3] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 3 passes its tests independently.

---

## Phase 6: Polish & Cross-Cutting

- [ ] T015 [P] `benches/store.rs`: open+list 500 < 10 ms, insert ≤ 20 ms (SC-003) — Executor: builder · Reviewer: thinker
- [ ] T016 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [ ] T017 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.
