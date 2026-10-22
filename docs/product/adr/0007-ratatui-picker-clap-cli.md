---
status: proposed
date: 2026-10-21
---
# 0007 — The picker in ratatui + crossterm, the CLI in clap, fuzzy search with nucleo-matcher

## Context and problem statement
The terminal picker must show its first frame in under 100 ms from a
keybinding, work with the keyboard only, support a plain mode for screen
readers, and replace `fzf`. The CLI needs subcommands, `--json` and
stable exit codes. The code lives in the same Rust crate (it embeds the
project's code, per `terminal-stacks.md`).

## Considered options
- TUI: **ratatui 0.30 + crossterm 0.29** versus cursive versus raw
  crossterm.
- CLI: **clap 4 (derive)** versus argh/lexopt (smaller).
- Matching: **nucleo-matcher 0.3** versus fuzzy-matcher versus a plain
  substring match.

## Decision outcome
Chosen: the options in bold. The picker restores the terminal on exit,
panic and signals (`ratatui::init`/`restore` plus a panic hook), and runs
inline (not on the alternate screen) when `--plain` is given.

### Consequences
- Good: the de-facto Rust TUI stack, with `TestBackend` and insta
  snapshots for tests. clap generates help and shell completions.
- Bad: clap adds about 1 MiB. Acceptable under F5.

### Confirmation
F6 (first frame) passes. Snapshot tests cover every picker state at
80×24 and 40×10.
