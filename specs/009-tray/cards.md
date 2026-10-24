# Orchestration cards: 009-tray

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/tray` (off `dev`) · Worktree: `.worktrees/feat-tray` · Wave: 4 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `009-foundation` | T001, T002, T013 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `009-us1-core` | T003, T004, T005, T014 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `009-us1-frontend` | T017 | builder | thinker | low | `frontend:tui` |
| `009-us2-core` | T006, T007, T015 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `009-us3-core` | T008, T009, T016 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `009-polish` | T010, T011 | builder | thinker | low | the feature's build stage |
| `009-qa` | T012, T018, T019 | builder | thinker | medium | `qa:e2e` / `qa:load`, the feature's build stage |

## Card `009-foundation`: T001, T002, T013

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T001, T002, T013: 009-tray foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T013 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: assets/icons/{normal,paused}.svg, contracts/menu.md, src/tray/menu.rs; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Add icon sources `assets/icons/{normal,paused}.svg` + rendered PNGs and `build.rs` ARGB generation
2. T002: Define pure `TrayMenu` model per `contracts/menu.md` in `src/tray/menu.rs`
3. T013: Add ksni (tokio), zbus (tokio, no default features), png (build-dep); `build.rs` icon ARGB generation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `009-us1-core`: T003, T004, T005, T014

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T003, T004, T005, T014: 009-tray us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T003, T004, T005, T014 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/tray/mod.rs, src/tray/tests.rs, tests/tray_dbus.rs; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T003: [P] [US1] menu-model unit tests (labels, truncation, ⏎, pin glyph, order) in `src/tray/tests.rs`
2. T004: [P] [US1] D-Bus integration with stub watcher: item registers, activate → restore in `tests/tray_dbus.rs`
3. T005: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T014: [US1] ksni `Tray` impl: icon pixmaps, tooltip, entry items → restore in `src/tray/mod.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `009-us1-frontend`: T017

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T017: 009-tray us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T017 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: assets/icons/png; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T017: [US1] Icon pixmaps from `assets/icons/png` (000-design-system T010) wired into `build.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `009-us2-core`: T006, T007, T015

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T006, T007, T015: 009-tray us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007, T015 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US2] model tests for recording/paused/timed/no-terminal/confirming (SC-002)
2. T007: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T015: [US2] Pause/Resume/Pause 1 min/Open picker/Clear-confirm items wired to daemon handlers

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `009-us3-core`: T008, T009, T016

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T008, T009, T016: 009-tray us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T009, T016 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: tests/tray_dbus.rs; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: [P] [US3] watcher absent/appears tests in `tests/tray_dbus.rs`
2. T009: [US3] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T016: [US3] Watcher tracking, `status.tray` states, config `tray = false`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `009-polish`: T010, T011

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T010, T011: 009-tray polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T010, T011 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T010: Manual KDE + waybar check entry in `docs/release.md` (SC-001)
2. T011: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `009-qa`: T012, T018, T019

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T012, T018, T019: 009-tray qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-tray (branch feat/tray, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T012, T018, T019 of specs/009-tray/tasks.md.

Inputs: specs/009-tray/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md, tests/e2e/E2E-K7.rs; Cargo.toml only to add the dependencies named in plan.md; specs/009-tray/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T012: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T018: `tests/e2e/E2E-K7.rs` (dbus-daemon + stub watcher)
3. T019: KDE + waybar manual rows in `docs/release.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
