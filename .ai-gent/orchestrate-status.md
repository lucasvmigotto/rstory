# Orchestration ledger

State: **running** — autonomous run approved 2026-10-22; dev environment landed (PR #2); wave 0 started. Update as cards run, so the run survives compaction or a crash.

| Wave | Feature | Card | Executor | Reviewer | Effort | Tasks | State | Worktree | Pending merge | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 001-store | `001-foundation` | builder | thinker | medium | 8 | done (reviewed) | `.worktrees/feat-store` | — | acceptance re-run; F1 proven on injected edge |
| 0 | 001-store | `001-us1-core` | builder | reviewer | high | 7 | done (reviewed) | `.worktrees/feat-store` | — | fresh review round 2: approve with nits; all sync mutations caught, 4096 cases green |
| 0 | 001-store | `001-us2-core` | builder | reviewer | high | 5 | done (reviewed) | `.worktrees/feat-store` | — | re-review: approve with nits; unpinned-floor rule unified (FR-018), nits queued after polish |
| 0 | 001-store | `001-us3-core` | builder | thinker | medium | 3 | done (reviewed) | `.worktrees/feat-store` | — | thinker review: green; read lock blocks daemon start → specs 004/006/007 updated |
| 0 | 001-store | `001-polish` | builder | thinker | medium | 3 | done (reviewed) | `.worktrees/feat-store` | — | benches green; batch sync tuned, crash suites re-run at 2048 |
| 1 | 000-design-system | `000-foundation` | builder | thinker | medium | 5 | planned | `.worktrees/feat-design-system` | — | |
| 1 | 000-design-system | `000-us1-frontend` | builder | thinker | medium | 3 | planned | `.worktrees/feat-design-system` | — | |
| 1 | 000-design-system | `000-us2-frontend` | thinker | reviewer | high | 2 | planned | `.worktrees/feat-design-system` | — | |
| 1 | 000-design-system | `000-polish` | builder | thinker | low | 1 | planned | `.worktrees/feat-design-system` | — | |
| 1 | 002-x11-capture | `002-foundation` | thinker | reviewer | high | 4 | planned | `.worktrees/feat-x11-capture` | — | |
| 1 | 002-x11-capture | `002-us1-core` | builder | reviewer | high | 4 | planned | `.worktrees/feat-x11-capture` | — | |
| 1 | 002-x11-capture | `002-us2-core` | builder | reviewer | high | 4 | planned | `.worktrees/feat-x11-capture` | — | |
| 1 | 002-x11-capture | `002-us3-core` | builder | thinker | medium | 5 | planned | `.worktrees/feat-x11-capture` | — | |
| 1 | 002-x11-capture | `002-polish` | builder | thinker | low | 3 | planned | `.worktrees/feat-x11-capture` | — | |
| 1 | 008-picker-launch | `008-foundation` | builder | thinker | low | 1 | planned | `.worktrees/feat-picker-launch` | — | |
| 1 | 008-picker-launch | `008-us1-core` | builder | thinker | medium | 5 | planned | `.worktrees/feat-picker-launch` | — | |
| 1 | 008-picker-launch | `008-polish` | scout | thinker | low | 1 | planned | `.worktrees/feat-picker-launch` | — | |
| 2 | 003-wayland-capture | `003-foundation` | builder | thinker | medium | 3 | planned | `.worktrees/feat-wayland-capture` | — | |
| 2 | 003-wayland-capture | `003-us1-core` | builder | reviewer | high | 3 | planned | `.worktrees/feat-wayland-capture` | — | |
| 2 | 003-wayland-capture | `003-us2-core` | builder | reviewer | high | 3 | planned | `.worktrees/feat-wayland-capture` | — | |
| 2 | 003-wayland-capture | `003-polish` | builder | thinker | low | 3 | planned | `.worktrees/feat-wayland-capture` | — | |
| 2 | 004-daemon | `004-foundation` | builder | reviewer | high | 5 | planned | `.worktrees/feat-daemon` | — | |
| 2 | 004-daemon | `004-us1-core` | builder | reviewer | high | 6 | planned | `.worktrees/feat-daemon` | — | |
| 2 | 004-daemon | `004-us2-core` | builder | reviewer | high | 3 | planned | `.worktrees/feat-daemon` | — | |
| 2 | 004-daemon | `004-us3-core` | builder | thinker | medium | 4 | planned | `.worktrees/feat-daemon` | — | |
| 2 | 004-daemon | `004-us4-core` | builder | thinker | medium | 4 | planned | `.worktrees/feat-daemon` | — | |
| 2 | 004-daemon | `004-polish` | builder | reviewer | high | 4 | planned | `.worktrees/feat-daemon` | — | |
| 3 | 005-privacy-filters | `005-foundation` | builder | thinker | low | 1 | planned | `.worktrees/feat-privacy-filters` | — | |
| 3 | 005-privacy-filters | `005-us1-core` | builder | thinker | medium | 4 | planned | `.worktrees/feat-privacy-filters` | — | |
| 3 | 005-privacy-filters | `005-us2-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-privacy-filters` | — | |
| 3 | 005-privacy-filters | `005-us3-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-privacy-filters` | — | |
| 3 | 005-privacy-filters | `005-polish` | builder | thinker | low | 3 | planned | `.worktrees/feat-privacy-filters` | — | |
| 3 | 006-cli | `006-foundation` | builder | thinker | medium | 3 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us1-core` | builder | thinker | medium | 5 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us1-frontend` | builder | thinker | medium | 2 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us2-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us2-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us3-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us3-frontend` | builder | thinker | low | 1 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us4-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-us4-frontend` | builder | thinker | low | 1 | planned | `.worktrees/feat-cli` | — | |
| 3 | 006-cli | `006-polish` | builder | thinker | medium | 3 | planned | `.worktrees/feat-cli` | — | |
| 3 | 012-autostart | `012-foundation` | builder | thinker | low | 1 | planned | `.worktrees/feat-autostart` | — | |
| 3 | 012-autostart | `012-us1-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-autostart` | — | |
| 3 | 012-autostart | `012-us1-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-autostart` | — | |
| 3 | 012-autostart | `012-us2-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-autostart` | — | |
| 3 | 012-autostart | `012-us2-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-autostart` | — | |
| 3 | 012-autostart | `012-polish` | builder | thinker | low | 2 | planned | `.worktrees/feat-autostart` | — | |
| 4 | 007-picker | `007-foundation` | thinker | reviewer | high | 3 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-us1-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-us1-frontend` | builder | reviewer | high | 3 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-us2-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-us2-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-us3-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-us3-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-picker` | — | |
| 4 | 007-picker | `007-polish` | builder | reviewer | high | 3 | planned | `.worktrees/feat-picker` | — | |
| 4 | 009-tray | `009-foundation` | builder | thinker | medium | 3 | planned | `.worktrees/feat-tray` | — | |
| 4 | 009-tray | `009-us1-core` | builder | thinker | medium | 4 | planned | `.worktrees/feat-tray` | — | |
| 4 | 009-tray | `009-us1-frontend` | builder | thinker | low | 1 | planned | `.worktrees/feat-tray` | — | |
| 4 | 009-tray | `009-us2-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-tray` | — | |
| 4 | 009-tray | `009-us3-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-tray` | — | |
| 4 | 009-tray | `009-polish` | builder | thinker | low | 2 | planned | `.worktrees/feat-tray` | — | |
| 4 | 010-gnome-extension | `010-foundation` | builder | reviewer | high | 3 | planned | `.worktrees/feat-gnome-extension` | — | |
| 4 | 010-gnome-extension | `010-us1-core` | builder | thinker | medium | 2 | planned | `.worktrees/feat-gnome-extension` | — | |
| 4 | 010-gnome-extension | `010-us1-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-gnome-extension` | — | |
| 4 | 010-gnome-extension | `010-us2-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-gnome-extension` | — | |
| 4 | 010-gnome-extension | `010-us2-frontend` | builder | reviewer | high | 1 | planned | `.worktrees/feat-gnome-extension` | — | |
| 4 | 010-gnome-extension | `010-polish` | builder | thinker | low | 2 | planned | `.worktrees/feat-gnome-extension` | — | |
| 5 | 011-doctor | `011-foundation` | builder | thinker | medium | 1 | planned | `.worktrees/feat-doctor` | — | |
| 5 | 011-doctor | `011-us1-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-doctor` | — | |
| 5 | 011-doctor | `011-us1-frontend` | builder | thinker | medium | 1 | planned | `.worktrees/feat-doctor` | — | |
| 5 | 011-doctor | `011-us2-core` | builder | thinker | medium | 3 | planned | `.worktrees/feat-doctor` | — | |
| 5 | 011-doctor | `011-polish` | builder | thinker | medium | 2 | planned | `.worktrees/feat-doctor` | — | |
| 5 | 013-release | `013-foundation` | thinker | reviewer | high | 1 | planned | `.worktrees/ci-release` | — | |
| 5 | 013-release | `013-us1-core` | thinker | reviewer | high | 3 | planned | `.worktrees/ci-release` | — | |
| 5 | 013-release | `013-us2-core` | thinker | reviewer | high | 3 | planned | `.worktrees/ci-release` | — | |
| 5 | 013-release | `013-us3-core` | builder | thinker | medium | 3 | planned | `.worktrees/ci-release` | — | |
| 5 | 013-release | `013-polish` | scout | thinker | low | 1 | planned | `.worktrees/ci-release` | — | |
| 6 | 000-design-system | `000-qa` | builder | thinker | low | 1 | planned | `.worktrees/feat-design-system` | — | |
| 6 | 001-store | `001-qa` | builder | thinker | low | 2 | planned | `.worktrees/feat-store` | — | |
| 6 | 001-store | `001-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/feat-store` | — | |
| 6 | 002-x11-capture | `002-qa` | builder | thinker | medium | 3 | planned | `.worktrees/feat-x11-capture` | — | |
| 6 | 002-x11-capture | `002-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/feat-x11-capture` | — | |
| 6 | 003-wayland-capture | `003-qa` | builder | thinker | low | 2 | planned | `.worktrees/feat-wayland-capture` | — | |
| 6 | 003-wayland-capture | `003-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/feat-wayland-capture` | — | |
| 6 | 004-daemon | `004-qa` | builder | reviewer | high | 3 | planned | `.worktrees/feat-daemon` | — | |
| 6 | 004-daemon | `004-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/feat-daemon` | — | |
| 6 | 005-privacy-filters | `005-qa` | builder | reviewer | high | 2 | planned | `.worktrees/feat-privacy-filters` | — | |
| 6 | 005-privacy-filters | `005-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/feat-privacy-filters` | — | |
| 6 | 006-cli | `006-qa` | builder | thinker | medium | 2 | planned | `.worktrees/feat-cli` | — | |
| 6 | 007-picker | `007-qa` | builder | reviewer | high | 2 | planned | `.worktrees/feat-picker` | — | |
| 6 | 008-picker-launch | `008-qa` | builder | thinker | low | 2 | planned | `.worktrees/feat-picker-launch` | — | |
| 6 | 009-tray | `009-qa` | builder | thinker | medium | 3 | planned | `.worktrees/feat-tray` | — | |
| 6 | 010-gnome-extension | `010-qa` | builder | thinker | medium | 2 | planned | `.worktrees/feat-gnome-extension` | — | |
| 6 | 010-gnome-extension | `010-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/feat-gnome-extension` | — | |
| 6 | 011-doctor | `011-qa` | builder | thinker | low | 2 | planned | `.worktrees/feat-doctor` | — | |
| 6 | 012-autostart | `012-qa` | builder | thinker | low | 2 | planned | `.worktrees/feat-autostart` | — | |
| 6 | 013-release | `013-qa` | builder | thinker | medium | 3 | planned | `.worktrees/ci-release` | — | |
| 6 | 013-release | `013-qa-charter` | thinker | reviewer | high | 1 | planned | `.worktrees/ci-release` | — | |

## Decisions the user owes

- Approve the orchestration plan (waves, routing) before wave 0 is dispatched.
- Each feature's landing on `dev` (merge type per git:workflow) and every push.
