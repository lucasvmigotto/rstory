# Implementation Plan: Terminal & menu design system

**Branch**: `feat/design-system` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

## Summary

A small `src/ui/` module shared by the picker (007), CLI (006) and doctor (011): theme (roles × tiers), glyphs with ASCII fallback, keyed copy constants, and ratatui widgets/components. Plus icon assets in `assets/icons/` (SVG sources, PNG renders) used by the tray (009) and the GNOME extension (010).

## Technical Context

**Language/Version**: Rust (per architecture) · **Primary Dependencies**: ratatui 0.30 (styles, widgets), crossterm 0.29 (detection), unicode-width 0.2 · **Testing**: insta + ratatui `TestBackend` · **Target Platform**: any terminal (16-color minimum) · **Performance Goals**: component render < 1 ms for 40 rows · **Constraints**: Constitution VII (no color-only state)

## Technical frontend decisions

| Concern | Choice | Rationale |
|---|---|---|
| State management (picker) | Elm-style pure `update(state, msg) -> (state, cmds)` | trivially testable keystroke journeys |
| Data fetching | blocking protocol client before the first frame + a `subscribe` reader thread posting messages | no async runtime in the picker (startup budget) |
| Width handling | `unicode-width` for display width; truncation with `…` at grapheme boundaries | wide CJK and emoji previews |
| Testing | insta snapshots of `TestBackend` buffers; vhs tape for the README demo (optional, not CI) | per terminal-design |
| Icons | hand-drawn symbolic SVG; PNGs rendered once and committed (no build-time SVG renderer) | keeps the build pure Rust |

## Constitution Check

PASS: I (pure-Rust deps), II (snapshots first), III (previews escaped through `EntryRow`), VI (CLI renderers share components), VII (core), the rest N/A.

## Project Structure

```text
src/ui/{mod.rs, theme.rs, glyphs.rs, copy.rs, widgets/{query_line,entry_row,preview,footer,banner,state_badge,check_line}.rs}
assets/icons/{recording,paused,disconnected}{,-symbolic}.svg, png/{16,22,24,32,48}/{light,dark}/*.png
```

## Complexity Tracking

No violations.
