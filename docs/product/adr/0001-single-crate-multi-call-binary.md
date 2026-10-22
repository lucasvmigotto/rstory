---
status: proposed
date: 2026-10-21
---
# 0001 — One crate, one multi-call binary (modular monolith)

## Context and problem statement
rstory needs a daemon, a CLI, a TUI picker, a tray and installers, and has
to ship as a single static artifact (brief G1) that people also get with
`cargo install rstory`. How should the code be packaged?

## Decision drivers
- Portability and a single artifact (driver 1)
- `cargo install rstory` must work, so every crate in the dependency
  chain has to be published on crates.io
- One maintainer, so publishing and versioning overhead should stay small

## Considered options
1. A single crate `rstory` with internal modules and one binary whose
   subcommands select the role (`rstory daemon`, `rstory pick`, …).
2. A Cargo workspace with internal crates (`rstory-core`, `rstory-backend`,
   …), all published.
3. Several binaries (`rstoryd`, `rstory`, `rstory-pick`).

## Decision outcome
Chosen: **option 1**. Module boundaries are enforced by visibility and the
F1 fitness test (architecture §5.3, §10).

### Consequences
- Good: one artifact, one crates.io publish, one version, and the
  smallest total binary size, since shared code is linked once.
- Good: the picker starts in-process with no extra exec.
- Bad: boundaries rely on a test instead of crate edges.
- Bad: clean compile time grows with the whole tree. Trigger E4 splits
  the crate when CI clean builds take more than 10 minutes.

### Confirmation
The F1 test stays green. `cargo install rstory` installs a working
binary. Release artifacts contain exactly one executable.
