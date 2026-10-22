# Orchestration cards: 002-x11-capture

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/x11-capture` (off `dev`) · Worktree: `.worktrees/feat-x11-capture` · Wave: 1 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `002-foundation` | T001, T002, T003, T014 | thinker | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `002-us1-core` | T004, T005, T015, T016 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `002-us2-core` | T006, T007, T017, T018 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `002-us3-core` | T008, T009, T010, T019, T020 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `002-polish` | T011, T012, T021 | builder | thinker | low | `backend:build`, the feature's build stage |
| `002-qa` | T013, T022, T023 | builder | thinker | medium | `qa:e2e` / `qa:load`, the feature's build stage |
| `002-qa-charter` | T024 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `002-foundation`: T001, T002, T003, T014

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T001, T002, T003, T014: 002-x11-capture foundation`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T003, T014 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: contracts/backend.md, src/backend/fake.rs, src/backend/mod.rs, src/backend/{mod,fake}.rs, tests/support/mutter.rs, tests/support/sway.rs, tests/support/x11.rs; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Add test fixtures: Xvfb launcher helper and a minimal X11 test client in `tests/support/x11.rs`
2. T002: [P] Add a headless-mutter launcher (`mutter --headless --wayland --virtual-monitor`, private D-Bus, Xauthority from `$XDG_RUNTIME_DIR/.mutter-Xwaylandauth.*`) in `tests/support/mutter.rs`
3. T003: Define `Backend` trait, `BackendEvent`, errors and a `FakeBackend` in `src/backend/mod.rs`, `src/backend/fake.rs` per `contracts/backend.md`
4. T014: Backend trait + `FakeBackend` (scriptable events, recorded serves) in `src/backend/{mod,fake}.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `002-us1-core`: T004, T005, T015, T016

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T004, T005, T015, T016: 002-x11-capture us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T004, T005, T015, T016 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/backend/x11/incr.rs, src/backend/x11/{mod,read}.rs, tests/x11_capture.rs; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T004: [P] [US1] Xvfb capture tests (UTF8_STRING, STRING→UTF-8, INCR 900 KiB, owner exit → cleared) in `tests/x11_capture.rs` (SC-001)
2. T005: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T015: [US1] X11 connection, XFIXES subscription, TARGETS-first read with limit and timeout in `src/backend/x11/{mod,read}.rs`
4. T016: [US1] INCR receive in `src/backend/x11/incr.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `002-us2-core`: T006, T007, T017, T018

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T006, T007, T017, T018: 002-x11-capture us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007, T017, T018 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/backend/x11/serve.rs, tests/x11_serve.rs; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US2] Xvfb serve tests (targets, 1 MiB INCR round trip, ownership loss, self_owned) in `tests/x11_serve.rs` (SC-002)
2. T007: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T017: [US2] Owner/serve state machine with SelectionRequest handling and INCR send in `src/backend/x11/serve.rs`
4. T018: [US2] `self_owned` detection by owner window id

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `002-us3-core`: T008, T009, T010, T019, T020

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T008, T009, T010, T019, T020: 002-x11-capture us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T008, T009, T010, T019, T020 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/session/mod.rs, src/session/tests.rs, tests/bridge_capture.rs; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T008: [P] [US3] selection table tests (≥ 8 rows incl. override, no_session, no_data_control_no_xwayland) in `src/session/tests.rs` (SC-003)
2. T009: [P] [US3] bridge-mode test on headless mutter: selection = x11/xwayland_bridge, and capture/restore between X11 clients on Mutter's XWayland, in `tests/bridge_capture.rs` (cross-protocol copies: manual GNOME matrix)
3. T010: [US3] Backend implementation — see `## Backend` (appended by `backend:spec`)
4. T019: [US3] `session::select` + Wayland registry probe (data-control globals) in `src/session/mod.rs`
5. T020: [US3] Bridge mode: X11 backend with `mode = xwayland_bridge`, connect retry 3 s

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `002-polish`: T011, T012, T021

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T011, T012, T021: 002-x11-capture polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T011, T012, T021 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T011: Record GNOME 48–51 manual check steps (GUI-app copies, brief Q7) in `docs/release.md` draft
2. T012: Run quickstart.md validation
3. T021: Document GNOME 48–51 manual check in `docs/release.md` (draft)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `002-qa`: T013, T022, T023

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T013, T022, T023: 002-x11-capture qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T013, T022, T023 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md, tests/support/sway.rs; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T013: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T022: Bridge-mode CI environment (headless mutter) in `tests/support/mutter.rs` validated by `qa:e2e`
3. T023: GNOME 48–51 manual matrix rows in `docs/release.md` (GUI-app copies, restore into Wayland apps)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `002-qa-charter`: T024

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T024: 002-x11-capture qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-x11-capture (branch feat/x11-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T024 of specs/002-x11-capture/tasks.md.

Inputs: specs/002-x11-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/product/qa/charters/002.md; Cargo.toml only to add the dependencies named in plan.md; specs/002-x11-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T024: Charter session → `docs/product/qa/charters/002.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
