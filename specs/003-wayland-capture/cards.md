# Orchestration cards: 003-wayland-capture

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/wayland-capture` (off `dev`) · Worktree: `.worktrees/feat-wayland-capture` · Wave: 2 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `003-foundation` | T001, T002, T010 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `003-us1-core` | T003, T004, T011 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `003-us2-core` | T005, T006, T012 | builder | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `003-polish` | T007, T008, T013 | builder | thinker | low | `backend:build`, the feature's build stage |
| `003-qa` | T009, T014 | builder | thinker | low | `qa:e2e` / `qa:load`, the feature's build stage |
| `003-qa-charter` | T015 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `003-foundation`: T001, T002, T010

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T001, T002, T010: 003-wayland-capture foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-wayland-capture (branch feat/wayland-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T010 of specs/003-wayland-capture/tasks.md.

Inputs: specs/003-wayland-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: protocols/, src/backend/wayland/protocols.rs, tests/support/wl_client.rs; Cargo.toml only to add the dependencies named in plan.md; specs/003-wayland-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [P] Add Wayland test client (sets/reads selection via data-control) in `tests/support/wl_client.rs`
2. T002: Verify wayland-protocols exposes `ext-data-control-v1`; else vendor XML under `protocols/` with `wayland-scanner` (research R2)
3. T010: Protocol bindings (wayland-protocols `staging` or vendored XML) in `src/backend/wayland/protocols.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `003-us1-core`: T003, T004, T011

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T003, T004, T011: 003-wayland-capture us1-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-wayland-capture (branch feat/wayland-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T003, T004, T011 of specs/003-wayland-capture/tasks.md.

Inputs: specs/003-wayland-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/backend/wayland/{mod,read}.rs, tests/wayland_capture.rs; Cargo.toml only to add the dependencies named in plan.md; specs/003-wayland-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T003: [P] [US1] sway capture tests (ext path, zwlr fallback via protocol filter, hint present in types, clear) in `tests/wayland_capture.rs` (SC-001)
2. T004: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T011: [US1] Connect + bind + offer tracking + read with limit/timeout in `src/backend/wayland/{mod,read}.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `003-us2-core`: T005, T006, T012

**Route**: Executor `builder` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T005, T006, T012: 003-wayland-capture us2-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-wayland-capture (branch feat/wayland-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T005, T006, T012 of specs/003-wayland-capture/tasks.md.

Inputs: specs/003-wayland-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/backend/wayland/serve.rs, tests/wayland_serve.rs; Cargo.toml only to add the dependencies named in plan.md; specs/003-wayland-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T005: [P] [US2] sway serve tests (MIME set, 1 MiB round trip, cancelled, self_owned, EPIPE tolerance) in `tests/wayland_serve.rs` (SC-002)
2. T006: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T012: [US2] Source/serve with async `send` handling and `cancelled` in `src/backend/wayland/serve.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `003-polish`: T007, T008, T013

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T007, T008, T013: 003-wayland-capture polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-wayland-capture (branch feat/wayland-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T007, T008, T013 of specs/003-wayland-capture/tasks.md.

Inputs: specs/003-wayland-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/003-wayland-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T007: Add KDE Plasma manual check to `docs/release.md` (SC-003)
2. T008: Run quickstart.md validation
3. T013: KDE Plasma manual check entry in `docs/release.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `003-qa`: T009, T014

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T009, T014: 003-wayland-capture qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-wayland-capture (branch feat/wayland-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T009, T014 of specs/003-wayland-capture/tasks.md.

Inputs: specs/003-wayland-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/003-wayland-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T009: QA tasks — see `## QA` (appended by `qa:strategy`)
2. T014: KDE manual matrix row in `docs/release.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `003-qa-charter`: T015

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T015: 003-wayland-capture qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-wayland-capture (branch feat/wayland-capture, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T015 of specs/003-wayland-capture/tasks.md.

Inputs: specs/003-wayland-capture/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/product/qa/charters/003.md; Cargo.toml only to add the dependencies named in plan.md; specs/003-wayland-capture/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T015: Charter session → `docs/product/qa/charters/003.md`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
