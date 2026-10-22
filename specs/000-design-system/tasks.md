---
description: "Task list for the terminal & menu design system"
---

# Tasks: Terminal & menu design system

**Tests**: REQUIRED (Constitution II).

## Frontend

### Phase 1: Setup

- [x] T001 [TUI] Add ratatui, crossterm, unicode-width, insta; create `src/ui/mod.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: `cargo build` green.

### Phase 2: Foundational

- [x] T002 [P] [TUI] Snapshot test harness rendering a component at (tier, width, ascii) combinations in `src/ui/tests.rs` — Executor: builder · Reviewer: thinker
- [x] T003 [TUI] Theme detection (NO_COLOR, TERM=dumb, COLORTERM, config `theme`) + roles × tiers in `src/ui/theme.rs` with unit tests — Executor: builder · Reviewer: thinker
- [x] T004 [P] [TUI] Glyphs with ASCII fallback + locale detection in `src/ui/glyphs.rs` — Executor: builder · Reviewer: thinker · Effort: low
- [x] T005 [P] [TUI] Copy constants keyed per ux-vision in `src/ui/copy.rs` (a test asserts every ux-vision key exists) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: theme, glyph and copy tests pass.

### Phase 3: User Story 1 - Consistent surfaces (P1) 🎯 MVP

- [x] T006 [P] [US1] [TUI] Snapshots first: QueryLine, EntryRow, PreviewPane, Footer, Banner, StateBadge in every tier and ASCII at 80/40 cols — Executor: builder · Reviewer: thinker
- [x] T007 [US1] [TUI] Implement widgets in `src/ui/widgets/*.rs` until the snapshots pass — Executor: builder · Reviewer: thinker
- [x] T008 [P] [US1] [CLI] Snapshots first, then `EntryLine` and `CheckLine` renderers in `src/ui/widgets/{entry_line,check_line}.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: SC-001. All component snapshots reviewed (`cargo insta review`).

### Phase 4: User Story 2 - Icons (P2)

- [x] T009 [US2] Draw the symbolic SVGs (recording: clipboard + 3 lines; paused: clipboard + 2 bars; disconnected: clipboard + slash) in `assets/icons/` — Executor: thinker · Reviewer: reviewer · Effort: high
- [x] T010 [US2] Render PNGs 16/22/24/32/48 light/dark into `assets/icons/png/`; review checklist at 16 px mono (SC-002) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the icon review checklist is ticked in the PR description.

### Phase 5: Polish

- [ ] T011 [TUI] Optional `docs/demo.tape` (vhs) for the README picker demo — Executor: builder · Reviewer: thinker · Effort: low

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [ ] T012 Icon review checklist (16 px mono) recorded in the PR template — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.
