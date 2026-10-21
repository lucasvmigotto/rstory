# Orchestration cards: 005-privacy-filters

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/privacy-filters` (off `dev`) · Worktree: `.worktrees/feat-privacy-filters` · Wave: 3 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `005-foundation` | T001 | builder | thinker | low | the feature's build stage |
| `005-us1-core` | T002, T003, T004, T012 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `005-us2-core` | T005, T006, T013 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `005-us3-core` | T007, T008, T014 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `005-polish` | T009, T010, T015 | builder | thinker | low | `backend:build`, the feature's build stage |
| `005-qa` | T011, T016 | builder | reviewer (fresh context) | high | `qa:e2e` / `qa:load`, the feature's build stage |
| `005-qa-charter` | T017 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `005-foundation`: T001

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T001: 005-privacy-filters foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/domain/filter.rs; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: Define `FilterDecision` and `SENSITIVE_HINTS` in `src/domain/filter.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `005-us1-core`: T002, T003, T004, T012

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T002, T003, T004, T012: 005-privacy-filters us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T002, T003, T004, T012 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/domain/filter.rs, tests/filter_props.rs; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T002: [P] [US1] proptest: random jobs incl. hinted ones; assert hinted bytes never reach store, logs, or post-decision memory (SC-001, F8) in `tests/filter_props.rs`
2. T003: [P] [US1] unit: hint with text, hint only, hint via bridge targets
3. T004: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T012: [US1] `CopyFilter` pre-read gates (pause, sensitive) using `SENSITIVE_HINTS` in `src/domain/filter.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `005-us2-core`: T005, T006, T013

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T005, T006, T013: 005-privacy-filters us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T005, T006, T013 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T005: [P] [US2] boundary tests cap−1/cap/cap+1 (SC-002)
2. T006: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T013: [US2] Post-read gates (oversize via limit+1, empty) and drop of rejected bytes

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `005-us3-core`: T007, T008, T014

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T007, T008, T014: 005-privacy-filters us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T007, T008, T014 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/daemon/pipeline.rs; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T007: [P] [US3] pause-gate unit tests + order-of-checks table test
2. T008: [US3] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T014: [US3] Wire filter into the daemon pipeline with counters (`skipped_*`) in `src/daemon/pipeline.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `005-polish`: T009, T010, T015

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T009, T010, T015: 005-privacy-filters polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T009, T010, T015 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T009: Write README sections: threat model, sensitive hints, `passc` wrapper (FR-007)
2. T010: Run quickstart.md validation
3. T015: README: threat model, hints, `passc` wrapper

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `005-qa`: T011, T016

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T011, T016: 005-privacy-filters qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T011, T016 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/e2e/E2E-K3.rs; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T011: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T016: `tests/e2e/E2E-K3.rs` (Xvfb, sway, bridge) and `E2E-K3b.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `005-qa-charter`: T017

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T017: 005-privacy-filters qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-privacy-filters (branch feat/privacy-filters, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T017 of specs/005-privacy-filters/tasks.md.

Inputs: specs/005-privacy-filters/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/product/qa/charters/005.md; Cargo.toml only to add the dependencies named in plan.md; specs/005-privacy-filters/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T017: Charter session → `docs/product/qa/charters/005.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
