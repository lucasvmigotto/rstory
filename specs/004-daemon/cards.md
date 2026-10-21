# Orchestration cards: 004-daemon

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/daemon` (off `dev`) · Worktree: `.worktrees/feat-daemon` · Wave: 2 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `004-foundation` | T001, T002, T017, T018, T019 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `004-us1-core` | T003, T004, T005, T020, T021, T022 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `004-us2-core` | T006, T007, T023 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `004-us3-core` | T008, T009, T024, T025 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `004-us4-core` | T010, T011, T012, T026 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `004-polish` | T013, T014, T015, T027 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `004-qa` | T016, T028, T029 | builder | reviewer (fresh context) | high | `qa:e2e` / `qa:load`, the feature's build stage |
| `004-qa-charter` | T030 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `004-foundation`: T001, T002, T017, T018, T019

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T001, T002, T017, T018, T019: 004-daemon foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T017, T018, T019 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/ipc/codec.rs, src/ipc/error.rs, src/ipc/peercred.rs, src/ipc/protocol.rs, src/ipc/server.rs, tests/fixtures/protocol/; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Create shared protocol fixtures (one NDJSON file per op/event, valid + invalid) in `tests/fixtures/protocol/`
2. T002: Implement protocol v1 types + codec (serde, line framing, 2 MiB limit) in `src/ipc/protocol.rs`, `src/ipc/codec.rs` with schema-validation tests against `/contracts/protocol.schema.json`
3. T017: Add tokio (`rt`,`net`,`time`,`signal`,`io-util`,`macros`,`sync`,`fs`), tracing-subscriber, rustix `net`; dev: `jsonschema` validator
4. T018: Socket server: runtime dir resolution, 0700 dir, bind/stale handling, `SO_PEERCRED` check, line codec with 2 MiB limit in `src/ipc/server.rs`, `src/ipc/peercred.rs`
5. T019: Error → protocol code mapping and correlation ids in `src/ipc/error.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-us1-core`: T003, T004, T005, T020, T021, T022

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T003, T004, T005, T020, T021, T022: 004-daemon us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T003, T004, T005, T020, T021, T022 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/daemon/handlers.rs, src/daemon/mod.rs, src/daemon/pipeline.rs, tests/contract/protocol.rs, tests/daemon_capture.rs; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T003: [P] [US1] contract tests: every fixture validates; server responses validate (SC-004) in `tests/contract/protocol.rs`
2. T004: [P] [US1] integration: single instance (exit 4), inactive state, capture→list < 100 ms p95 (F7), burst 500/s bounded + `dropped_burst` in `tests/daemon_capture.rs`
3. T005: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T020: [US1] Daemon main: single instance (store LOCK_EX), backend selection, inactive state, signal handling in `src/daemon/mod.rs`
5. T021: [US1] Capture pipeline: bounded queue, filter hook (005), batched `Store::apply`, `history_changed` fan-out in `src/daemon/pipeline.rs`
6. T022: [US1] Handlers: list, show, status, subscribe in `src/daemon/handlers.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-us2-core`: T006, T007, T023

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T006, T007, T023: 004-daemon us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007, T023 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/daemon/restore.rs, tests/daemon_restore.rs; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US2] restore tests (ok after ownership, not_found, inactive, self_owned → top) in `tests/daemon_restore.rs`
2. T007: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T023: [US2] Restore: resolve → touch → backend serve → ack after ownership; self_owned handling in `src/daemon/restore.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-us3-core`: T008, T009, T024, T025

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T008, T009, T024, T025: 004-daemon us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T009, T024, T025 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/daemon/timers.rs, src/domain/pause.rs, tests/daemon_pause.rs; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: [P] [US3] pause tests with tokio paused clock (indefinite, timed, resume early, persisted across restart, past end resumes) in `tests/daemon_pause.rs`
2. T009: [US3] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T024: [US3] RecordingState aggregate (pause, timed pause, persisted, resume on load) in `src/domain/pause.rs` + handlers pause/resume
4. T025: [US3] Deadline timer (`min(next_expiry, paused_until)`) in `src/daemon/timers.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-us4-core`: T010, T011, T012, T026

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T010, T011, T012, T026: 004-daemon us4-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T010, T011, T012, T026 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/daemon/state.rs, tests/daemon_socket.rs, tests/daemon_status.rs; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T010: [P] [US4] status schema + single-timer + idle wake-up (F10) tests in `tests/daemon_status.rs`
2. T011: [P] [US4] peercred rejection, malformed line, oversize line, stale socket, SIGTERM cleanup tests in `tests/daemon_socket.rs`
3. T012: [US4] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T026: [US4] Counters + status read model; reconnect policy for bridge mode in `src/daemon/state.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-polish`: T013, T014, T015, T027

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T013, T014, T015, T027: 004-daemon polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T013, T014, T015, T027 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/daemon_rss.rs, tests/fitness/canary.rs; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T013: RSS check after 10 000 captures (SC-002) in `tests/daemon_rss.rs`
2. T014: [P] F11 canary: capture a canary at RUST_LOG=trace; assert absent from logs in `tests/fitness/canary.rs`
3. T015: Run quickstart.md validation
4. T027: Burst tuning, RSS verification, logging review against F11

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-qa`: T016, T028, T029

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T016, T028, T029: 004-daemon qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T016, T028, T029 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/e2e/E2E-K1.rs, tests/e2e/support.rs, tests/perf/{burst,soak}.rs; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T016: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T028: `tests/e2e/E2E-K1.rs`, `E2E-K4.rs`, `E2E-PEER.rs` (pty/expectrl harness + session launchers in `tests/e2e/support.rs`)
3. T029: Burst and soak harness `tests/perf/{burst,soak}.rs` per the load profile

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `004-qa-charter`: T030

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T030: 004-daemon qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-daemon (branch feat/daemon, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T030 of specs/004-daemon/tasks.md.

Inputs: specs/004-daemon/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/product/qa/charters/004.md; Cargo.toml only to add the dependencies named in plan.md; specs/004-daemon/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T030: Charter session → `docs/product/qa/charters/004.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
