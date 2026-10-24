# Orchestration cards: 001-store

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/store` (off `dev`) · Worktree: `.worktrees/feat-store` · Wave: 0 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `001-foundation` | T001, T002, T003, T004, T005, T018, T019, T020 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `001-us1-core` | T006, T007, T008, T009, T021, T022, T023 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `001-us2-core` | T010, T011, T012, T024, T025 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `001-us3-core` | T013, T014, T026 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `001-polish` | T015, T016, T027 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `001-qa` | T017, T029 | builder | thinker | low | `qa:e2e` / `qa:load`, the feature's build stage |
| `001-qa-charter` | T028 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `001-foundation`: T001, T002, T003, T004, T005, T018, T019, T020

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T001, T002, T003, T004, T005, T018, T019, T020: 001-store foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T003, T004, T005, T018, T019, T020 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/config/mod.rs, src/daemon/log.rs, src/domain/ids.rs, src/domain/redact.rs, src/lib.rs, src/main.rs, src/store/fs.rs, tests/fitness/boundaries.rs; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: Create the crate skeleton: `Cargo.toml` (edition 2024, `license = "GPL-3.0-or-later"`, `rust-version`), `src/main.rs`, `src/lib.rs` with `#![forbid(unsafe_code)]` and the module tree from plan.md
2. T002: [P] Add `rust-toolchain.toml` (pinned), `rustfmt.toml`, `clippy.toml`, `deny.toml` (bans F3, licenses F4)
3. T003: [P] Add `tests/fitness/boundaries.rs` (F1 module-dependency rules from architecture §5.3) — failing until modules exist
4. T004: [P] Define `Fs` abstraction with real and fault-injecting implementations in `src/store/fs.rs` (used by crash tests)
5. T005: [P] Implement `config` caps parsing/validation with defaults (20/10/1 MiB/7d) in `src/config/mod.rs` + unit tests
6. T018: Add deps (serde, serde_json, blake3 `pure`, rustix `fs`, time, rand, thiserror, tracing) and dev-deps (proptest, tempfile) to `Cargo.toml`
7. T019: `Redacted<T>` wrapper + tracing init helper in `src/domain/redact.rs`, `src/daemon/log.rs`
8. T020: `EntryId` (ULID layout), `ShortId`, `EntryRef` value objects with unit tests in `src/domain/ids.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `001-us1-core`: T006, T007, T008, T009, T021, T022, T023

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T006, T007, T008, T009, T021, T022, T023: 001-store us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007, T008, T009, T021, T022, T023 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/domain/entry.rs, src/store/index.rs, src/store/{mod,fs,perms}.rs, tests/fitness/perms.rs, tests/store_crash.rs, tests/store_open.rs; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US1] proptest crash-injection suite in `tests/store_crash.rs` (SC-001)
2. T007: [P] [US1] permission tests (dir 0700, files 0600, `store_too_open`) in `tests/fitness/perms.rs` (F9)
3. T008: [P] [US1] orphan-sweep and unknown-version tests in `tests/store_open.rs`
4. T009: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
5. T021: [US1] `Content`, `ContentHash`, `Preview` value objects in `src/domain/entry.rs`
6. T022: [US1] Atomic write helpers + `Store::open_exclusive` with perms check and orphan sweep in `src/store/{mod,fs,perms}.rs`
7. T023: [US1] Index v1 serde types + load/commit in `src/store/index.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `001-us2-core`: T010, T011, T012, T024, T025

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T010, T011, T012, T024, T025: 001-store us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T010, T011, T012, T024, T025 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/domain/history.rs, tests/history_props.rs; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T010: [P] [US2] proptest invariants (count cap with pins counted, pin cap, de-dup, expiry, clear vs clear_all) in `tests/history_props.rs` (SC-002)
2. T011: [P] [US2] boundary tests: 21st insert evicts oldest unpinned; 11th pin → `pin_limit` in `src/domain/history.rs` tests
3. T012: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T024: [US2] `History` aggregate: record (de-dup, evict), touch, pin/unpin (PinLimit, idempotent), remove, clear, clear_all, expire, next_deadline in `src/domain/history.rs`
5. T025: [US2] `Store::apply(batch)` mapping mutations to content writes + one index commit + unlinks

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `001-us3-core`: T013, T014, T026

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T013, T014, T026: 001-store us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T013, T014, T026 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/store/readonly.rs, tests/store_readonly.rs; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T013: [US3] read-only handle tests (shared lock, `daemon_running`) in `tests/store_readonly.rs`
2. T014: [US3] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T026: [US3] `Store::open_read_only` (LOCK_SH|LOCK_NB, `DaemonRunning`) exposing only read APIs in `src/store/readonly.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `001-polish`: T015, T016, T027

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T015, T016, T027: 001-store polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T015, T016, T027 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: benches/store.rs; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T015: [P] `benches/store.rs`: open+list 500 < 10 ms, insert ≤ 20 ms (SC-003)
2. T016: Run quickstart.md validation
3. T027: Bench `benches/store.rs` and tune (SC-003)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `001-qa`: T017, T029

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T017, T029: 001-store qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T017, T029 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T017: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T029: Nightly property job with 100 000 cases (gate wiring for `devsecops:pipeline`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `001-qa-charter`: T028

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T028: 001-store qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-store (branch feat/store, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T028 of specs/001-store/tasks.md.

Inputs: specs/001-store/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/product/qa/charters/001.md; Cargo.toml only to add the dependencies named in plan.md; specs/001-store/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T028: Exploratory charter session for the store (SIGKILL during bursts) → notes in `docs/product/qa/charters/001.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
