# Orchestration cards: 000-design-system

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `feat/design-system` (off `dev`) · Worktree: `.worktrees/feat-design-system` · Wave: 1 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `000-foundation` | T001, T002, T003, T004, T005 | builder | thinker | medium | `frontend:tui` |
| `000-us1-frontend` | T006, T007, T008 | builder | thinker | medium | `frontend:tui` |
| `000-us2-frontend` | T009, T010 | thinker | reviewer (fresh context) | high | `frontend:tui` |
| `000-polish` | T011 | builder | thinker | low | `frontend:tui` |
| `000-qa` | T012 | builder | thinker | low | `qa:e2e` / `qa:load` |

## Card `000-foundation`: T001, T002, T003, T004, T005

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T001, T002, T003, T004, T005: 000-design-system foundation`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-design-system (branch feat/design-system, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001, T002, T003, T004, T005 of specs/000-design-system/tasks.md.

Inputs: specs/000-design-system/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/ui/copy.rs, src/ui/glyphs.rs, src/ui/mod.rs, src/ui/tests.rs, src/ui/theme.rs; Cargo.toml only to add the dependencies named in plan.md; specs/000-design-system/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: [TUI] Add ratatui, crossterm, unicode-width, insta; create `src/ui/mod.rs`
2. T002: [P] [TUI] Snapshot test harness rendering a component at (tier, width, ascii) combinations in `src/ui/tests.rs`
3. T003: [TUI] Theme detection (NO_COLOR, TERM=dumb, COLORTERM, config `theme`) + roles × tiers in `src/ui/theme.rs` with unit tests
4. T004: [P] [TUI] Glyphs with ASCII fallback + locale detection in `src/ui/glyphs.rs`
5. T005: [P] [TUI] Copy constants keyed per ux-vision in `src/ui/copy.rs` (a test asserts every ux-vision key exists)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `000-us1-frontend`: T006, T007, T008

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T006, T007, T008: 000-design-system us1-frontend`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-design-system (branch feat/design-system, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T006, T007, T008 of specs/000-design-system/tasks.md.

Inputs: specs/000-design-system/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: src/ui/widgets/*.rs, src/ui/widgets/{entry_line,check_line}.rs; Cargo.toml only to add the dependencies named in plan.md; specs/000-design-system/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T006: [P] [US1] [TUI] Snapshots first: QueryLine, EntryRow, PreviewPane, Footer, Banner, StateBadge in every tier and ASCII at 80/40 cols
2. T007: [US1] [TUI] Implement widgets in `src/ui/widgets/*.rs` until the snapshots pass
3. T008: [P] [US1] [CLI] Snapshots first, then `EntryLine` and `CheckLine` renderers in `src/ui/widgets/{entry_line,check_line}.rs`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `000-us2-frontend`: T009, T010

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T009, T010: 000-design-system us2-frontend`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/feat-design-system (branch feat/design-system, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T009, T010 of specs/000-design-system/tasks.md.

Inputs: specs/000-design-system/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: assets/icons/, assets/icons/png/; Cargo.toml only to add the dependencies named in plan.md; specs/000-design-system/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T009: [US2] Draw the symbolic SVGs (recording: clipboard + 3 lines; paused: clipboard + 2 bars; disconnected: clipboard + slash) in `assets/icons/`
2. T010: [US2] Render PNGs 16/22/24/32/48 light/dark into `assets/icons/png/`; review checklist at 16 px mono (SC-002)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo insta test --check

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `000-polish`: T011

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T011: 000-design-system polish`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-design-system (branch feat/design-system, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T011 of specs/000-design-system/tasks.md.

Inputs: specs/000-design-system/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/demo.tape; Cargo.toml only to add the dependencies named in plan.md; specs/000-design-system/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T011: [TUI] Optional `docs/demo.tape` (vhs) for the README picker demo

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `000-qa`: T012

**Route**: Executor `builder` · Reviewer `thinker` · Effort `low` · Dispatch description: `T012: 000-design-system qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/feat-design-system (branch feat/design-system, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T012 of specs/000-design-system/tasks.md.

Inputs: specs/000-design-system/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/000-design-system/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T012: Icon review checklist (16 px mono) recorded in the PR template

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
