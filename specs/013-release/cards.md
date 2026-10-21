# Orchestration cards: 013-release

Plan only. Generated 2026-10-21 by `agents:orchestrate` from `tasks.md`. **Nothing has been dispatched.**

- Branch: `ci/release` (off `dev`) · Worktree: `.worktrees/ci-release` · Wave: 5 for build cards, 6 for `-qa*` cards (see `docs/product/agent-orchestration.md`)
- Cards in one feature run **in order** on that worktree. Parallelism happens across features in the same wave.
- The pointer tasks (`… see ## Backend/Frontend/QA`) are closed by the card that builds that layer for the story.

| Card | Tasks | Executor | Reviewer | Effort | Stage that owns the ticks |
|---|---|---|---|---|---|
| `013-foundation` | T001 | thinker | reviewer (fresh context) | high | the feature's build stage |
| `013-us1-core` | T002, T003, T011 | thinker | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `013-us2-core` | T005, T006, T012 | thinker | reviewer (fresh context) | high | `backend:build`, the feature's build stage |
| `013-us3-core` | T007, T008, T013 | builder | thinker | medium | `backend:build`, the feature's build stage |
| `013-polish` | T009 | scout | thinker | low | the feature's build stage |
| `013-qa` | T004, T010, T015 | builder | thinker | medium | `qa:e2e` / `qa:load`, the feature's build stage |
| `013-qa-charter` | T014 | thinker | reviewer (fresh context) | high | `qa:e2e` / `qa:load` |

## Card `013-foundation`: T001

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T001: 013-release foundation`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T001 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T001: Decide job graph with `devsecops:pipeline` (CI + release), environments (`release` protected), runners (x86_64, arm64)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo build && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `013-us1-core`: T002, T003, T011

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T002, T003, T011: 013-release us1-core`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T002, T003, T011 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: .github/workflows/ci.yml, docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T002: [US1] CI workflow validation: actionlint + a deliberate-failure PR checklist in `docs/release.md`
2. T003: [US1] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T011: [US1] `.github/workflows/ci.yml` per spec FR-001/002 — via `devsecops:pipeline`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `013-us2-core`: T005, T006, T012

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T005, T006, T012: 013-release us2-core`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T005, T006, T012 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: .github/workflows/release.yml; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T005: [US2] release dry-run job + assertion step: no `secrets.*` publishing token referenced (SC-001)
2. T006: [US2] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T012: [US2] `.github/workflows/release.yml` per FR-003/004 — via `devsecops:pipeline`

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `013-us3-core`: T007, T008, T013

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T007, T008, T013: 013-release us3-core`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T007, T008, T013 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: docs/release.md; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T007: [US3] `docs/release.md` runbook review against brief K9 and ADR-0009
2. T008: [US3] Backend implementation — see `## Backend` (appended by `backend:spec`)
3. T013: [US3] `docs/release.md` runbook + Tier-1 manual matrix

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `013-polish`: T009

**Route**: Executor `scout` · Reviewer `thinker` · Effort `low` · Dispatch description: `T009: 013-release polish`

```markdown
You are the scout for rstory. Find and report facts only: no edits, no commits.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T009 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T009: Run quickstart.md validation

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test && cargo bench --no-run

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `013-qa`: T004, T010, T015

**Route**: Executor `builder` · Reviewer `thinker` · Effort `medium` · Dispatch description: `T004, T010, T015: 013-release qa`

```markdown
You are the builder for rstory (a Rust clipboard history, single crate). Follow the card exactly. Tests first: write the tests named in the card, show them failing, then implement until they pass. Honour `.specify/memory/constitution.md` (I–IX), in particular III: never log clipboard content.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T004, T010, T015 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T004: [US1] QA tasks — see `## QA` (appended by `qa:strategy`)
2. T010: QA tasks — see `## QA` (appended by `qa:strategy`)
3. T015: Flaky-test policy wiring (nextest config, quarantine label)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```

## Card `013-qa-charter`: T014

**Route**: Executor `thinker` · Reviewer `reviewer` (fresh context) · Effort `high` · Dispatch description: `T014: 013-release qa-charter`

```markdown
You are the thinker for rstory. This card is design-heavy: produce the interface or design first, as code with doc comments, keep it minimal, then the tests and implementation. Cite the ADRs and constitution principles you rely on.

Work ONLY in .worktrees/ci-release (branch ci/release, off dev; do not switch branches). Conventional Commits on this branch; no commit trailers; never --no-verify; do not merge or push.

Goal: complete T014 of specs/013-release/tasks.md.

Inputs: specs/013-release/spec.md, plan.md, data-model.md, contracts/, backend.md / tui.md / ui.md / qa.md where present; /contracts/protocol.md; docs/product/architecture.md §5.3 (module boundaries); .specify/memory/constitution.md.
May change: the paths named in the tasks below; Cargo.toml only to add the dependencies named in plan.md; specs/013-release/tasks.md (tick only these task ids).
Must not touch: other features' source files beyond their public API, /contracts/* (a spec change goes through backend:spec), docs/product/*.

Do:
1. T014: Gate wiring spec for `devsecops:pipeline` from `test-strategy.md` §6 (PR, dev, main, tag, nightly)

Never: network calls at runtime, real clipboard contents in fixtures, logging content, editing other branches, publishing, secrets.

Acceptance: cargo test --test 'e2e_*' (inside the tools container with Xvfb/sway available)

Final report (concise): what changed per task id, files, test results (counts), anything you could not verify and why. No file contents.
```
