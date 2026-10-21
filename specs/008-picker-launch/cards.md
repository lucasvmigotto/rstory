# Orchestration cards: 008-picker-launch

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/picker-launch` (off `dev`) · Worktree: `.worktrees/feat-picker-launch` · Wave: 1 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `008-foundation` | T001 | builder | thinker | low | the feature's build stage |
| `008-us1-core` | T002, T003, T004, T007, T008 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `008-polish` | T005 | scout | thinker | low | the feature's build stage |
| `008-qa` | T006, T009 | builder | thinker | low | `qa:e2e` / `qa:load`, the feature's build stage |

## Card `008-foundation`: T001

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T001: 008-picker-launch foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker-launch (branch feat/picker-launch, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001 of specs/008-picker-launch/tasks.md.

Inputs: specs/008-picker-launch/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/launch/mod.rs; Cargo.toml only to add the dependencies named in plan.md; specs/008-picker-launch/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: Define `TERMINALS` table and `LaunchPlan` in `src/launch/mod.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `008-us1-core`: T002, T003, T004, T007, T008

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T002, T003, T004, T007, T008: 008-picker-launch us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker-launch (branch feat/picker-launch, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T002, T003, T004, T007, T008 of specs/008-picker-launch/tasks.md.

Inputs: specs/008-picker-launch/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/launch/mod.rs, src/launch/tests.rs; Cargo.toml only to add the dependencies named in plan.md; specs/008-picker-launch/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T002: [P] [US1] resolution-order tests (config › $TERMINAL › xdg-terminal-exec › list) and one argv test per table row (SC-001) in `src/launch/tests.rs`
2. T003: [P] [US1] protocol test: open_picker ok / no_terminal; status.picker_available
3. T004: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T007: [US1] `launch::plan` resolution + `TERMINALS` table in `src/launch/mod.rs`
5. T008: [US1] `launch::spawn` detached (setsid, null stdio) + `open_picker` handler + `picker_available` in status

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `008-polish`: T005

**Route**: Executor `scout` · Reviewer `thinker` · Effort `low` · Dispatch description: `T005: 008-picker-launch polish`

```markdown
You are the scout for rstory. Find and report facts only: no edits, no commits.

Work ONLY in .worktrees/feat-picker-launch (branch feat/picker-launch, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T005 of specs/008-picker-launch/tasks.md.

Inputs: specs/008-picker-launch/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/008-picker-launch/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T005: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `008-qa`: T006, T009

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T006, T009: 008-picker-launch qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-picker-launch (branch feat/picker-launch, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T009 of specs/008-picker-launch/tasks.md.

Inputs: specs/008-picker-launch/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/008-picker-launch/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T009: Verify the launch table against the terminals installed in the tools container (smoke)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
