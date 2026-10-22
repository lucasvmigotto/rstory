# Feature Specification: Terminal & menu design system

**Feature Branch**: `feat/design-system`

**Created**: 2026-10-21

**Status**: Draft

**Input**: `docs/product/ux-vision.md` (visual direction, tokens, glyphs, copy keys, keymap); Constitution VII.

> Tests are **required** (Constitution II): snapshot tests of each component in every color tier and in ASCII mode.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Consistent, readable surfaces in any terminal (Priority: P1)

Every terminal surface (picker, CLI output, doctor) renders with the same semantic roles, glyphs and spacing, in 16-color, 256, truecolor, monochrome and ASCII modes, and the state stays readable without color.

**Why this priority**: Constitution VII. The picker and CLI depend on it.

**Independent Test**: Render each component in a `TestBackend` at each tier. Snapshots show that state cues (`>`, `📌`/`[p]`, words) are present in monochrome.

**Acceptance Scenarios**:

1. **Given** `NO_COLOR=1`, **When** any component renders, **Then** no ANSI color codes are emitted, and selected, pinned, matched and state are still distinguishable.
2. **Given** a non-UTF-8 locale or `--ascii`, **When** components render, **Then** glyphs use the ASCII fallbacks.
3. **Given** `theme = "rstory"` and `COLORTERM=truecolor`, **When** components render, **Then** the truecolor tier values are used.

---

### User Story 2 - Recognisable icons in every state (Priority: P2)

Tray and panel icons are distinguishable by shape for recording, paused and disconnected, in light and dark panels.

**Why this priority**: Principle 2 ("recording state is never ambiguous").

**Independent Test**: Icon assets exist for every state and size, and a review checklist confirms the silhouettes differ.

**Acceptance Scenarios**:

1. **Given** the icon set, **When** shown at 16 px in monochrome, **Then** recording (three lines), paused (two bars) and disconnected (slash) are distinguishable.

## Requirements *(mandatory)*

- **FR-001**: The semantic color roles (`accent`, `selected`, `muted`, `pinned`, `warning`, `danger`) MUST be defined once, with values per tier as in ux-vision.
- **FR-002**: The tier MUST be detected from `NO_COLOR`, `TERM`, `COLORTERM` and config `theme` (`terminal` default, `rstory` opt-in, `mono`).
- **FR-003**: Glyphs MUST be defined once, with ASCII fallbacks, and selected by locale or `--ascii`.
- **FR-004**: Components: `QueryLine`, `EntryRow`, `PreviewPane`, `Footer` (hints, status, confirm), `Banner`, `StateBadge`, `CheckLine` (doctor), `EntryLine` (CLI `ls`).
- **FR-005**: Copy strings MUST be keyed constants (`src/ui/copy.rs`) matching ux-vision's key-screen copy.
- **FR-006**: Icons: symbolic SVG sources for recording, paused and disconnected; PNG renders at 16/22/24/32/48 px, light and dark, for SNI.

## Success Criteria *(mandatory)*

- **SC-001**: Every component has snapshots in 16-color, mono and ASCII at 80 and 40 columns.
- **SC-002**: The icon review checklist passes at 16 px, monochrome.
