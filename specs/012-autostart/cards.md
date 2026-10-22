# Orchestration cards: 012-autostart

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/autostart` (off `dev`) · Worktree: `.worktrees/feat-autostart` · Wave: 3 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `012-foundation` | T001 | builder | thinker | low | the feature's build stage |
| `012-us1-core` | T002, T003 | builder | thinker | medium | the feature's build stage |
| `012-us1-frontend` | T009 | builder | thinker | medium | `frontend:tui` |
| `012-us2-core` | T004, T005 | builder | thinker | medium | the feature's build stage |
| `012-us2-frontend` | T010 | builder | thinker | medium | `frontend:tui` |
| `012-polish` | T006, T007 | builder | thinker | low | the feature's build stage |
| `012-qa` | T008, T011 | builder | thinker | low | `qa:e2e` / `qa:load`, the feature's build stage |

## Card `012-foundation`: T001

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T001: 012-autostart foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/install/autostart.rs; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: Templates (desktop entry, systemd unit) as consts in `src/install/autostart.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `012-us1-core`: T002, T003

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T002, T003: 012-autostart us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T002, T003 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/install_autostart.rs; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T002: [P] [US1] byte-exact template + idempotency + uninstall + temp-path warning tests in `tests/install_autostart.rs` (SC-002)
2. T003: [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `012-us1-frontend`: T009

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T009: 012-autostart us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T009 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/install/autostart.rs; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T009: [US1] [CLI] `install-autostart` XDG path + messages + temp-path warning in `src/install/autostart.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `012-us2-core`: T004, T005

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T004, T005: 012-autostart us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T004, T005 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T004: [P] [US2] unit template + stub systemctl tests
2. T005: [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `012-us2-frontend`: T010

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T010: 012-autostart us2-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T010 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T010: [US2] [CLI] `--systemd`, `--now`, `--uninstall`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `012-polish`: T006, T007

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T006, T007: 012-autostart polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: Manual re-login check on GNOME/KDE/sway in `docs/release.md` (SC-001)
2. T007: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `012-qa`: T008, T011

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T008, T011: 012-autostart qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-autostart (branch feat/autostart, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T011 of specs/012-autostart/tasks.md.

Inputs: specs/012-autostart/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/012-autostart/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T011: Re-login manual rows (GNOME, KDE, sway) in `docs/release.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
