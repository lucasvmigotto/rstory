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

- [x] T001 Create the crate skeleton: `Cargo.toml` (edition 2024, `license = "GPL-3.0-or-later"`, `rust-version`), `src/main.rs`, `src/lib.rs` with `#![forbid(unsafe_code)]` and the module tree from plan.md — Executor: builder · Reviewer: thinker · Effort: low
- [x] T002 [P] Add `rust-toolchain.toml` (pinned), `rustfmt.toml`, `clippy.toml`, `deny.toml` (bans F3, licenses F4) — Executor: builder · Reviewer: thinker · Effort: low
- [x] T003 [P] Add `tests/fitness/boundaries.rs` (F1 module-dependency rules from architecture §5.3) — failing until modules exist — Executor: builder · Reviewer: thinker

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T004 [P] Define `Fs` abstraction with real and fault-injecting implementations in `src/store/fs.rs` (used by crash tests) — Executor: builder · Reviewer: thinker
- [x] T005 [P] Implement `config` caps parsing/validation with defaults (20/10/1 MiB/7d) in `src/config/mod.rs` + unit tests — Executor: builder · Reviewer: thinker

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Recorded entries survive restarts (Priority: P1) 🎯 MVP

**Independent Test**: Insert, crash at random points, reopen: every acknowledged entry present

### Tests for User Story 1 ⚠️

- [x] T006 [P] [US1] proptest crash-injection suite in `tests/store_crash.rs` (SC-001) — Executor: builder · Reviewer: thinker
- [x] T007 [P] [US1] permission tests (dir 0700, files 0600, `store_too_open`) in `tests/fitness/perms.rs` (F9) — Executor: builder · Reviewer: thinker
- [x] T008 [P] [US1] orphan-sweep and unknown-version tests in `tests/store_open.rs` — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T009 [US1] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - History stays small and fresh (Priority: P1)

**Independent Test**: Property tests over random op sequences assert caps, de-dup, expiry, pin rules

### Tests for User Story 2 ⚠️

- [x] T010 [P] [US2] proptest invariants (count cap with pins counted, pin cap, de-dup, expiry, clear vs clear_all) in `tests/history_props.rs` (SC-002) — Executor: builder · Reviewer: thinker
- [x] T011 [P] [US2] boundary tests: 21st insert evicts oldest unpinned; 11th pin → `pin_limit` in `src/domain/history.rs` tests — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T012 [US2] Backend implementation — see `## Backend` (appended by `backend:spec`) — Executor: builder · Reviewer: thinker

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

---

## Backend

Appended by `backend:spec`. Built by `backend:build` (ticks only this section). Tests first.


### Backend Setup

- [x] T018 Add deps (serde, serde_json, blake3 `pure`, rustix `fs`, time, rand, thiserror, tracing) and dev-deps (proptest, tempfile) to `Cargo.toml` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: `cargo build` and `cargo test` green; setup pieces covered by unit tests.

### Backend Foundational

- [x] T019 `Redacted<T>` wrapper + tracing init helper in `src/domain/redact.rs`, `src/daemon/log.rs` — Executor: builder · Reviewer: thinker
- [x] T020 `EntryId` (ULID layout), `ShortId`, `EntryRef` value objects with unit tests in `src/domain/ids.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: `cargo build` and `cargo test` green; foundational pieces covered by unit tests.

### Backend US1 — Recorded entries survive restarts

- [x] T021 [US1] `Content`, `ContentHash`, `Preview` value objects in `src/domain/entry.rs` — Executor: builder · Reviewer: thinker
- [x] T022 [US1] Atomic write helpers + `Store::open_exclusive` with perms check and orphan sweep in `src/store/{mod,fs,perms}.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T023 [US1] Index v1 serde types + load/commit in `src/store/index.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US2 — History stays small and fresh

- [x] T024 [US2] `History` aggregate: record (de-dup, evict), touch, pin/unpin (PinLimit, idempotent), remove, clear, clear_all, expire, next_deadline in `src/domain/history.rs` — Executor: builder · Reviewer: reviewer · Effort: high
- [x] T025 [US2] `Store::apply(batch)` mapping mutations to content writes + one index commit + unlinks — Executor: builder · Reviewer: reviewer · Effort: high

**Checkpoint**: the US2 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend US3 — Clients read while the daemon is down

- [ ] T026 [US3] `Store::open_read_only` (LOCK_SH|LOCK_NB, `DaemonRunning`) exposing only read APIs in `src/store/readonly.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US3 tests in this file's story phase pass (`cargo test`), plus the contract tests where they apply.

### Backend Polish

- [ ] T027 Bench `benches/store.rs` and tune (SC-003) — Executor: builder · Reviewer: thinker

**Checkpoint**: the budgets in backend.md are met; `/speckit-converge` (Backend) reports converged.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [ ] T028 Exploratory charter session for the store (SIGKILL during bursts) → notes in `docs/product/qa/charters/001.md` — Executor: thinker · Reviewer: reviewer · Effort: high
- [ ] T029 Nightly property job with 100 000 cases (gate wiring for `devsecops:pipeline`) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.
