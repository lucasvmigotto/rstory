# Orchestration cards: 007-picker

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/picker` (off `dev`) · Worktree: `.worktrees/feat-picker` · Wave: 4 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `007-foundation` | T001, T002, T014 | thinker | reviewer (fresh context) | high | `frontend:tui`, the feature's build stage |
| `007-us1-core` | T003, T004, T005 | builder | thinker | medium | the feature's build stage |
| `007-us1-frontend` | T015, T016, T017 | builder | reviewer (fresh context) | high | `frontend:tui` |
| `007-us2-core` | T006, T007 | builder | thinker | medium | the feature's build stage |
| `007-us2-frontend` | T018 | builder | thinker | medium | `frontend:tui` |
| `007-us3-core` | T008, T009, T010 | builder | thinker | medium | the feature's build stage |
| `007-us3-frontend` | T019 | builder | thinker | medium | `frontend:tui` |
| `007-polish` | T011, T012, T020 | builder | reviewer (fresh context) | high | `frontend:tui`, the feature's build stage |
| `007-qa` | T013, T021 | builder | reviewer (fresh context) | high | `qa:e2e` / `qa:load`, the feature's build stage |

## Card `007-foundation`: T001, T002, T014

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T001, T002, T014: 007-picker foundation`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T014 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/picker/state.rs, src/picker/term.rs; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Add ratatui/crossterm/nucleo-matcher deps and insta snapshot config
2. T002: Define `PickerState`, `Msg`, `Cmd` and pure `update()` skeleton in `src/picker/state.rs`
3. T014: [TUI] terminal lifecycle: init/restore, panic hook, SIGTERM/SIGHUP restore, TTY check (exit 2) in `src/picker/term.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-us1-core`: T003, T004, T005

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T003, T004, T005: 007-picker us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T003, T004, T005 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/picker/tests.rs; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T003: [P] [US1] journey tests on `update()` (filter, rank, select, Enter → Restore, Esc 1, Ctrl-C 130) in `src/picker/tests.rs`
2. T004: [P] [US1] snapshots: normal, filtered, preview on/off at 80×24 and 40×10
3. T005: [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-us1-frontend`: T015, T016, T017

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T015, T016, T017: 007-picker us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T015, T016, T017 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/picker/state.rs, src/picker/view.rs; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T015: [US1] [TUI] `update()` for query editing, filtering (nucleo), ranking, selection-by-id, Enter → Restore in `src/picker/state.rs`
2. T016: [US1] [TUI] layout + render (QueryLine, list of EntryRow, PreviewPane, Footer) with responsive splits in `src/picker/view.rs`
3. T017: [US1] [TUI] data: initial list (socket or read-only), debounced `show` for the preview, `subscribe` reader thread

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-us2-core`: T006, T007

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T006, T007: 007-picker us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US2] journey tests for pin toggle, pin_limit status line, delete confirm y/n
2. T007: [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-us2-frontend`: T018

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T018: 007-picker us2-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T018 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T018: [US2] [TUI] Ctrl-T pin toggle, Ctrl-D confirm mode, pin-limit status line

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-us3-core`: T008, T009, T010

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T008, T009, T010: 007-picker us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T009, T010 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: [P] [US3] snapshots plain + NO_COLOR + read-only banner + empty + too small (SC-003)
2. T009: [P] [US3] terminal-restore-on-panic test
3. T010: [US3] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-us3-frontend`: T019

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T019: 007-picker us3-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T019 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T019: [US3] [TUI] `--plain` inline renderer, NO_COLOR/ASCII, read-only banner and Enter → exit 3, empty and too-small states

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-polish`: T011, T012, T020

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T011, T012, T020: 007-picker polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T011, T012, T020 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: benches/picker_first_frame.rs; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T011: [P] `benches/picker_first_frame.rs` 20 and 500 entries (F6, SC-001/002)
2. T012: Run quickstart.md validation
3. T020: [TUI] first-frame optimization against F6 (load before draw, no alt-screen flicker); F1 keymap footer

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `007-qa`: T013, T021

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T013, T021: 007-picker qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker (branch feat/picker, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T013, T021 of specs/007-picker/tasks.md.

Inputs: specs/007-picker/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/e2e/E2E-K2.rs; Cargo.toml only to add the dependencies named in plan.md; specs/007-picker/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T013: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T021: `tests/e2e/E2E-K2.rs` (keyboard-only recall through a pty, X11 and sway; negative read-only path)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
