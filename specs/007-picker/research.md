# Research: Terminal picker

Phase 0 for `007-picker`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Fuzzy matcher

- **Decision**: nucleo-matcher (fzf-like scoring, fast, small)
- **Alternatives rejected**: fuzzy-matcher (slower, unmaintained), substring only (worse UX)

## R2. Startup budget

- **Decision**: skip alternate-screen flicker; load index (≤250 KiB) before first draw; no async runtime
- **Alternatives rejected**: tokio runtime in picker

## R3. Plain mode

- **Decision**: inline viewport (ratatui `Viewport::Inline`), ASCII markers
- **Alternatives rejected**: separate renderer
