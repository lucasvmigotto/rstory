# rstory — agent orchestration plan

Status: **Planned, not started** (2026-10-21). Owner: `agents:orchestrate`
(+ `agents:parallel` for the worktrees). Nothing here has been dispatched.
Implementation starts only after the user approves this plan.

## 1. Roles

Roles resolve to models in `.ai-gent/orchestrate.yaml` (defaults from
`agent-roles.md`):

| Role | Model (Claude) | Does here | Never |
|---|---|---|---|
| `thinker` | Opus | approves waves, design-heavy cards (Backend trait, picker state model, icons, release job graph), exploratory charters, merge/no-merge calls, reports to the user | bulk edits, long implementations |
| `builder` | Sonnet | every implementation card: tests first, then code, commits on its feature branch | merges, pushes, spec or contract edits |
| `reviewer` | Opus, **fresh context** | re-runs acceptance and reviews the diff for the 17 high-risk cards (protocol, socket server, store crash safety, X11/Wayland serve, picker terminal lifecycle, JS client) | trusting the builder's report |
| `scout` | Haiku | lookups (e.g. 003 foundation: does wayland-protocols expose `ext-data-control-v1`?), CI watching via `devsecops:watch` | edits, commits |

## 2. Cards

- **Every feature has `specs/NNN-*/cards.md`**, generated from its
  `tasks.md`. Each card is a self-contained prompt (template from
  `agents/orchestrate/references/cards.md`) with task ids, inputs, the
  paths it may change, acceptance commands, the "never" list and the
  report format.
- **Grouping:**
  - `foundation`: Setup + Foundational (layer-neutral, Backend and
    Frontend);
  - `usN-core`: story N's acceptance tests + its `## Backend` tasks;
  - `usN-frontend`: story N's `## Frontend` (`[TUI]`/`[CLI]`) tasks;
  - `polish`;
  - `qa`: `## QA` journeys and suites;
  - `qa-charter`: exploratory sessions.
- **Routing:**
  - The executor is the strongest executor tag among the card's real
    tasks. Pointer tasks are ignored.
  - The reviewer is `reviewer` when any task asks for it, else `thinker`.
  - Effort is `high` when any task is high, and `low` when all are.
- **Totals:** 99 cards. 67 are builder→thinker, 17 builder→reviewer, 13
  thinker→reviewer and 2 scout→thinker.
- **The ledger** `.ai-gent/orchestrate-status.md` lists every card with
  its wave, role, worktree and state. Update it as cards run.

## 3. Waves (dependency order, at most 4 features in parallel)

```
W0  001 store ─────────────────────────────── (crate skeleton; lands first, alone)
W1  000 design-system │ 002 x11-capture │ 008 picker-launch
W2  003 wayland-capture │ 004 daemon                (004 needs 001, 002)
W3  005 privacy-filters │ 006 cli │ 012 autostart  (all need 004)
W4  007 picker │ 009 tray │ 010 gnome-extension     (007 needs 000+006; 009/010 need 004+008)
W5  011 doctor │ 013 release                        (011 reads 002/003/004/008/010/012; 013 via devsecops:pipeline)
W6  all -qa / -qa-charter cards                     (qa:e2e, qa:load: need the full stack)
```

| Wave | Cards | Thinker-led cards | Gate to start the next wave |
|---|---|---|---|
| W0 | 5 | — | `feat/store` landed on `dev` (ff or no-ff by count, asked) |
| W1 | 12 | `000-us2-frontend` (icons), `002-foundation` (Backend trait) | 000, 002 and 008 landed on `dev` |
| W2 | 10 | — | 003 and 004 landed. Capture → list works on Xvfb and sway. |
| W3 | 21 | — | 005, 006 and 012 landed |
| W4 | 20 | `007-foundation` (picker state model) | 007, 009 and 010 landed. **The owner checks GNOME 48 by hand** (extension + bridge). |
| W5 | 10 | `013-foundation`, `013-us1-core`, `013-us2-core` (job graph and workflows, with `devsecops:pipeline`) | 011 and 013 landed. CI is green on `dev` (`devsecops:watch`). |
| W6 | 21 | 7 charters (`001`, `002`, `003`, `004`, `005`, `010`, `013` `-qa-charter`) | every feature's exit criteria → Verified in `specs/README.md` |

Within a feature, cards run **in order** on that feature's worktree
(`foundation` → `us1-core` → `us1-frontend` → `us2-*` → … → `polish`).
Parallelism is only across features in the same wave, because their files
are disjoint (architecture §5.3 module ownership). Shared files
(`Cargo.toml`, `src/lib.rs` module list) are touched only by small
explicit edits. Conflicts get settled on `dev` at landing (`git:workflow`).

## 4. Worktrees and branches

- One worktree per feature: `.worktrees/<branch with / → ->>`, for
  example `.worktrees/feat-daemon` on `feat/daemon`. Each is created by
  the thinker off the current `dev` right before its wave (after the
  previous wave landed).
- Builders work and commit only inside their worktree, by explicit
  paths, with Conventional Commits, no trailers and never `--no-verify`.
- `.worktrees/` must be git-ignored. The first card of W0 adds it to
  `.gitignore` (this is a Setup task to add to `001` T002 at dispatch
  time).

## 5. Review loop

1. The builder reports: the task ids, the files, test counts, and open
   doubts. No file dumps.
2. The reviewer or thinker reads the **diff + report**, re-runs the
   card's acceptance commands, and checks the constitution gates the
   card touches (III no-content logging, IV module boundaries via F1).
3. Small fixes are made by the reviewer. Larger ones go back to the
   **same** builder with only the delta.
4. The ledger is updated, and the feature's `tasks.md` boxes are ticked
   by the owning stage only (`backend:build`, `frontend:tui`, `qa:e2e`).
5. At the feature's last checkpoint: `/speckit-converge` per layer, then
   status Implemented in `specs/README.md`.

## 6. Gates that always stop and ask the user

- Approving this plan (before W0).
- **Every landing on `dev`, every push**, and any promotion to `main`
  (`git:workflow`). There's no autonomous-run permission yet.
- Anything external: the first crates.io publish (a manual runbook, the
  owner's token), configuring the Trusted Publisher, tags and releases.
- The manual GNOME 48–51 / KDE / waybar matrix sign-offs.

## 7. Token economy

- The thinker never reads whole trees: diffs and reports only. Lookups
  go to the scout.
- The role preamble is identical across cards of the same role (cache
  reuse), and card-specific text comes last.
- Mechanical cards (`Effort: low`: skeletons, templates, doc rows) go to
  the builder at low effort.
- A builder stopped by a session limit is resumed ("check what you
  already did, then finish"), never restarted.
- Before each wave: estimate the wave's cost against the remaining budget
  and say proceed, shrink, or stop (pacing). After the run:
  `token_report.py tasks --specs-dir specs` links costs to `tasks.md`.

## 8. How to start (once approved)

1. Approve, optionally adjusting the waves or `max_parallel`.
2. The thinker creates `.worktrees/feat-store` off `dev` and dispatches
   `001-foundation` (the builder, in the background).
3. Then follow the ledger, wave by wave.
