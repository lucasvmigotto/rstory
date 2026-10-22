# Orchestration cards: 010-gnome-extension

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/gnome-extension` (off `dev`) · Worktree: `.worktrees/feat-gnome-extension` · Wave: 4 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `010-foundation` | T001, T002, T011 | builder | reviewer (fresh context) | high | `frontend:tui`, the feature's build stage |
| `010-us1-core` | T003, T004 | builder | thinker | medium | the feature's build stage |
| `010-us1-frontend` | T012 | builder | thinker | medium | `frontend:tui` |
| `010-us2-core` | T005, T006, T007 | builder | thinker | medium | the feature's build stage |
| `010-us2-frontend` | T013 | builder | reviewer (fresh context) | high | `frontend:tui` |
| `010-polish` | T008, T009 | builder | thinker | low | the feature's build stage |
| `010-qa` | T010, T014 | builder | thinker | medium | `qa:e2e` / `qa:load`, the feature's build stage |
| `010-qa-charter` | T015 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `010-foundation`: T001, T002, T011

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T001, T002, T011: 010-gnome-extension foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T011 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: extension/, extension/client.js, extension/menu.js, icons/, specs/009-tray/contracts/menu.md; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Create `extension/` skeleton (`metadata.json` with uuid + shell-version 48–51, `extension.js`, `stylesheet.css`, `icons/`) and eslint config
2. T002: JS protocol client module (`extension/client.js`): connect, NDJSON read loop, request ids, reconnect backoff 1→30 s
3. T011: [TUI] JS menu model module (`extension/menu.js`) per `specs/009-tray/contracts/menu.md` + Disconnected state

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-us1-core`: T003, T004

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T003, T004: 010-gnome-extension us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T003, T004 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/install_gnome.rs; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T003: [P] [US1] install/reinstall/uninstall/--enable (stub gnome-extensions) tests in `tests/install_gnome.rs` (SC-003)
2. T004: [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-us1-frontend`: T012

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T012: 010-gnome-extension us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T012 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/install/gnome.rs; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T012: [US1] [CLI] `install-gnome-extension` (embed via include_dir, write, `--enable`, `--uninstall`, version compare) in `src/install/gnome.rs` with tui.md messages

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-us2-core`: T005, T006, T007

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T005, T006, T007: 010-gnome-extension us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T005, T006, T007 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md, extension/tests/client.test.js; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T005: [P] [US2] gjs fixture test of client + menu model in `extension/tests/client.test.js` (SC-002)
2. T006: [US2] Manual matrix procedure for Shell 48–51 in `docs/release.md` (SC-001)
3. T007: [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-us2-frontend`: T013

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T013: 010-gnome-extension us2-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T013 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T013: [US2] [TUI] `extension.js`: PanelMenu.Button, symbolic icons per state, menu from the model, actions → client requests, full `disable()` cleanup

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-polish`: T008, T009

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T008, T009: 010-gnome-extension polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T009 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: eslint + GNOME review rules check (disable() cleanup) in CI
2. T009: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-qa`: T010, T014

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T010, T014: 010-gnome-extension qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T010, T014 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T010: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T014: GNOME Shell 48–51 manual matrix + leak-check procedure in `docs/release.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `010-qa-charter`: T015

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T015: 010-gnome-extension qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-gnome-extension (branch feat/gnome-extension, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T015 of specs/010-gnome-extension/tasks.md.

Inputs: specs/010-gnome-extension/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/product/qa/charters/010.md; Cargo.toml only to add the dependencies named in plan.md; specs/010-gnome-extension/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T015: Charter session per Shell version → `docs/product/qa/charters/010.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
