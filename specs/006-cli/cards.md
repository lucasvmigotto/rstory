# Orchestration cards: 006-cli

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/cli` (off `dev`) · Worktree: `.worktrees/feat-cli` · Wave: 3 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `006-foundation` | T001, T002, T018 | builder | thinker | medium | `frontend:tui`, the feature's build stage |
| `006-us1-core` | T003, T004, T005, T016, T017 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `006-us1-frontend` | T019, T020 | builder | thinker | medium | `frontend:tui` |
| `006-us2-core` | T006, T007 | builder | thinker | medium | the feature's build stage |
| `006-us2-frontend` | T021 | builder | thinker | medium | `frontend:tui` |
| `006-us3-core` | T008, T009 | builder | thinker | medium | the feature's build stage |
| `006-us3-frontend` | T022 | builder | thinker | low | `frontend:tui` |
| `006-us4-core` | T010, T011 | builder | thinker | medium | the feature's build stage |
| `006-us4-frontend` | T023 | builder | thinker | low | `frontend:tui` |
| `006-polish` | T012, T013, T014 | builder | thinker | medium | the feature's build stage |
| `006-qa` | T015, T024 | builder | thinker | medium | `qa:e2e` / `qa:load`, the feature's build stage |

## Card `006-foundation`: T001, T002, T018

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T001, T002, T018: 006-cli foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T018 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/cli/args.rs, src/cli/mod.rs, tests/support/daemon.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Add `assert_cmd`/`insta` dev-deps and a test harness that starts a daemon with FakeBackend on a temp runtime dir in `tests/support/daemon.rs`
2. T002: Define `ExitCode` enum and `EntryRef` parser per `/contracts/cli.md` in `src/cli/mod.rs` with unit tests
3. T018: [CLI] clap command tree for every subcommand with help text and examples (`src/cli/args.rs`); `rstory completions` via clap_complete

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us1-core`: T003, T004, T005, T016, T017

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T003, T004, T005, T016, T017: 006-cli us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T003, T004, T005, T016, T017 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/cli/source.rs, src/ipc/client.rs, tests/cli_browse.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T003: [P] [US1] assert_cmd tests: ls text (insta), ls --json schema, show raw bytes, copy, unknown/ambiguous ref exit 2 in `tests/cli_browse.rs`
2. T004: [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)
3. T005: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T016: [US1] Blocking protocol client with timeouts and typed errors in `src/ipc/client.rs`
5. T017: [US1] Read-only fallback adapter (`ls`/`show`) in `src/cli/source.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us1-frontend`: T019, T020

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T019, T020: 006-cli us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T019, T020 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/cli/render.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T019: [US1] [CLI] snapshots first, then `ls` renderer (TTY width-aware, piped TSV) and `show` raw output in `src/cli/render.rs` using `EntryLine`
2. T020: [US1] [CLI] `--json` serializers per `/contracts/cli.md` with `schema: 1`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us2-core`: T006, T007

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T006, T007: 006-cli us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/cli_manage.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US2] assert_cmd tests incl. pin_limit exit 6, clear prompt/--yes/no-TTY, clear --all prompt, pause without daemon exit 3 in `tests/cli_manage.rs`
2. T007: [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us2-frontend`: T021

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T021: 006-cli us2-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T021 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/cli/commands.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T021: [US2] [CLI] pin/unpin/rm/clear/clear --all/pause/resume commands with prompts (TTY, /dev/tty, --yes) and copy-keyed messages in `src/cli/commands.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us3-core`: T008, T009

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T008, T009: 006-cli us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T009 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/cli_readonly.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: [P] [US3] read-only fallback tests in `tests/cli_readonly.rs`
2. T009: [US3] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us3-frontend`: T022

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T022: 006-cli us3-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T022 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T022: [US3] [CLI] read-only fallback messaging (`cli.readonly`) and exit 3 for write commands

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us4-core`: T010, T011

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T010, T011: 006-cli us4-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T010, T011 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/cli_status.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T010: [P] [US4] status/config tests in `tests/cli_status.rs`; exit-code coverage matrix (SC-003)
2. T011: [US4] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-us4-frontend`: T023

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T023: 006-cli us4-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T023 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/cli/status.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T023: [US4] [CLI] `status` text renderer + `config [--path]` in `src/cli/status.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-polish`: T012, T013, T014

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T012, T013, T014: 006-cli polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T012, T013, T014 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: benches/ls.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T012: [P] `benches/ls.rs`: < 50 ms at 500 entries (SC-001)
2. T013: Shell completions smoke test
3. T014: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `006-qa`: T015, T024

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T015, T024: 006-cli qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-cli (branch feat/cli, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T015, T024 of specs/006-cli/tasks.md.

Inputs: specs/006-cli/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/e2e/E2E-K6.rs; Cargo.toml only to add the dependencies named in plan.md; specs/006-cli/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T015: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T024: `tests/e2e/E2E-K6.rs` (scripting, non-TTY)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
