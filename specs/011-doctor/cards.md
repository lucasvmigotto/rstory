# Orchestration cards: 011-doctor

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/doctor` (off `dev`) · Worktree: `.worktrees/feat-doctor` · Wave: 5 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `011-foundation` | T001 | builder | thinker | medium | the feature's build stage |
| `011-us1-core` | T002, T003, T004 | builder | thinker | medium | the feature's build stage |
| `011-us1-frontend` | T011 | builder | thinker | medium | `frontend:tui` |
| `011-us2-core` | T005, T006, T010 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `011-polish` | T007, T008 | builder | thinker | medium | the feature's build stage |
| `011-qa` | T009, T012 | builder | thinker | low | `qa:e2e` / `qa:load`, the feature's build stage |

## Card `011-foundation`: T001

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T001: 011-doctor foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-doctor (branch feat/doctor, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001 of specs/011-doctor/tasks.md.

Inputs: specs/011-doctor/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/doctor/mod.rs; Cargo.toml only to add the dependencies named in plan.md; specs/011-doctor/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: Define `Check`, statuses, text/JSON renderers and exit scale in `src/doctor/mod.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `011-us1-core`: T002, T003, T004

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T002, T003, T004: 011-doctor us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-doctor (branch feat/doctor, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T002, T003, T004 of specs/011-doctor/tasks.md.

Inputs: specs/011-doctor/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/doctor/tests.rs; Cargo.toml only to add the dependencies named in plan.md; specs/011-doctor/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T002: [P] [US1] healthy-baseline test + JSON schema test in `src/doctor/tests.rs`
2. T003: [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)
3. T004: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `011-us1-frontend`: T011

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T011: 011-doctor us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-doctor (branch feat/doctor, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T011 of specs/011-doctor/tasks.md.

Inputs: specs/011-doctor/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/doctor/render.rs; Cargo.toml only to add the dependencies named in plan.md; specs/011-doctor/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T011: [US1] [CLI] text renderer (CheckLine, summary line) and `--json` in `src/doctor/render.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `011-us2-core`: T005, T006, T010

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T005, T006, T010: 011-doctor us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-doctor (branch feat/doctor, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T005, T006, T010 of specs/011-doctor/tasks.md.

Inputs: specs/011-doctor/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/doctor/probes.rs; Cargo.toml only to add the dependencies named in plan.md; specs/011-doctor/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T005: [P] [US2] table-driven failure tests (8 rows, status/message/hint; SC-001)
2. T006: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T010: [US2] Probe adapters with timeouts (session, store, daemon status, extension, shell version, launch, autostart) in `src/doctor/probes.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `011-polish`: T007, T008

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T007, T008: 011-doctor polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-doctor (branch feat/doctor, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T007, T008 of specs/011-doctor/tasks.md.

Inputs: specs/011-doctor/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/011-doctor/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T007: Assert doctor never prints content (canary)
2. T008: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `011-qa`: T009, T012

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T009, T012: 011-doctor qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-doctor (branch feat/doctor, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T009, T012 of specs/011-doctor/tasks.md.

Inputs: specs/011-doctor/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/011-doctor/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T009: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T012: Reuse E2E-K1/K4 assertions on doctor output

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
