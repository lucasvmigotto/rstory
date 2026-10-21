---
description: "Task list for the terminal & menu design system"
---

# Tasks: Terminal & menu design system

**Tests**: REQUIRED (Constitution II).

## Frontend

### Phase 1: Setup

- [ ] T001 [TUI] Add ratatui, crossterm, unicode-width, insta; create `src/ui/mod.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: `cargo build` green.

### Phase 2: Foundational

- [ ] T002 [P] [TUI] Snapshot test harness rendering a component at (tier, width, ascii) combinations in `src/ui/tests.rs` — Executor: builder · Reviewer: thinker
- [ ] T003 [TUI] Theme detection (NO_COLOR, TERM=dumb, COLORTERM, config `theme`) + roles × tiers in `src/ui/theme.rs` with unit tests — Executor: builder · Reviewer: thinker
- [ ] T004 [P] [TUI] Glyphs with ASCII fallback + locale detection in `src/ui/glyphs.rs` — Executor: builder · Reviewer: thinker · Effort: low
- [ ] T005 [P] [TUI] Copy constants keyed per ux-vision in `src/ui/copy.rs` (a test asserts every ux-vision key exists) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: theme, glyph and copy tests pass.

### Phase 3: User Story 1 - Consistent surfaces (P1) 🎯 MVP

- [ ] T006 [P] [US1] [TUI] Snapshots first: QueryLine, EntryRow, PreviewPane, Footer, Banner, StateBadge in every tier and ASCII at 80/40 cols — Executor: builder · Reviewer: thinker
- [ ] T007 [US1] [TUI] Implement widgets in `src/ui/widgets/*.rs` until the snapshots pass — Executor: builder · Reviewer: thinker
- [ ] T008 [P] [US1] [CLI] Snapshots first, then `EntryLine` and `CheckLine` renderers in `src/ui/widgets/{entry_line,check_line}.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: SC-001. All component snapshots reviewed (`cargo insta review`).

### Phase 4: User Story 2 - Icons (P2)

- [ ] T009 [US2] Draw the symbolic SVGs (recording: clipboard + 3 lines; paused: clipboard + 2 bars; disconnected: clipboard + slash) in `assets/icons/` — Executor: thinker · Reviewer: reviewer · Effort: high
- [ ] T010 [US2] Render PNGs 16/22/24/32/48 light/dark into `assets/icons/png/`; review checklist at 16 px mono (SC-002) — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the icon review checklist is ticked in the PR description.

### Phase 5: Polish

- [ ] T011 [TUI] Optional `docs/demo.tape` (vhs) for the README picker demo — Executor: builder · Reviewer: thinker · Effort: low
