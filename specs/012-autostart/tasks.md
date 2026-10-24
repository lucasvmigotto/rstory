---
description: "Task list for Autostart"
---

# Tasks: Autostart

**Input**: Design documents from `/specs/012-autostart/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: REQUIRED (Constitution II). Test tasks precede implementation and must fail first.

**Organization**: layer-neutral phases by user story; detailed layer work lives in the `## Backend`, `## Frontend` and `## QA` sections appended by `backend:spec`, `frontend:spec` and `qa:strategy`.

**Roles**: each task carries `Executor · Reviewer · Effort` (see `.ai-gent/orchestrate.yaml` and `docs/product/agent-orchestration.md`).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

No feature-specific setup (crate skeleton comes from 001).

---

## Phase 2: Foundational

**⚠️ CRITICAL**: blocks every story in this feature.

- [x] T001 Templates (desktop entry, systemd unit) as consts in `src/install/autostart.rs` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: foundation ready.

---

## Phase 3: User Story 1 - Start at every login, any init system (Priority: P1) 🎯 MVP

**Independent Test**: Temp XDG_CONFIG_HOME: desktop entry written, idempotent, uninstall

### Tests for User Story 1 ⚠️

- [x] T002 [P] [US1] byte-exact template + idempotency + uninstall + temp-path warning tests in `tests/install_autostart.rs` (SC-002) — Executor: builder · Reviewer: thinker

### Implementation for User Story 1

- [x] T003 [US1] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 1 passes its tests independently.

---

## Phase 4: User Story 2 - systemd user unit (Priority: P2)

**Independent Test**: --systemd writes unit; stub systemctl for --now; no systemd → exit 5

### Tests for User Story 2 ⚠️

- [x] T004 [P] [US2] unit template + stub systemctl tests — Executor: builder · Reviewer: thinker

### Implementation for User Story 2

- [x] T005 [US2] Terminal UI / CLI implementation — see `## Frontend` (appended by `frontend:spec`; `[TUI]`/`[CLI]` tasks built by `frontend:tui`) — Executor: builder · Reviewer: thinker

**Checkpoint**: User Story 2 passes its tests independently.

---

## Phase 5: Polish & Cross-Cutting

- [x] T006 Manual re-login check on GNOME/KDE/sway in `docs/release.md` (SC-001) — Executor: builder · Reviewer: thinker · Effort: low
- [x] T007 Run quickstart.md validation — Executor: scout · Reviewer: thinker · Effort: low
- [x] T008 QA tasks — see `## QA` (appended by `qa:strategy`) — Executor: thinker · Reviewer: reviewer · Effort: high

---

## Dependencies & Execution Order

- Setup → Foundational → stories in priority order; stories with no shared files may run in parallel once Foundational is done.
- Within a story: tests (fail first) → layer implementation → checkpoint.
- Cross-feature dependencies: see `specs/README.md`.

## Implementation Strategy

MVP first: User Story 1 end to end, validate at its checkpoint, then the next story. The implement step belongs to `backend:build` / `frontend:tui` (+ `qa:e2e` for `## QA`), each ticking only its own section; close with `/speckit-converge`.

---

## Frontend

Appended by `frontend:spec`. `[TUI]`/`[CLI]` tasks built by `frontend:tui` (ticks only this section). Depends on `000-design-system`. Snapshot and journey tests first.


### Frontend US1 — Start at every login, any init system

- [x] T009 [US1] [CLI] `install-autostart` XDG path + messages + temp-path warning in `src/install/autostart.rs` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US1 snapshot/journey tests pass; `cargo insta review` is clean.

### Frontend US2 — systemd user unit

- [x] T010 [US2] [CLI] `--systemd`, `--now`, `--uninstall` — Executor: builder · Reviewer: thinker

**Checkpoint**: the US2 snapshot/journey tests pass; `cargo insta review` is clean.

---

## QA

Appended by `qa:strategy`. Executed by `qa:e2e` / `qa:load` after the builds (ticks only this section).

- [x] T011 Re-login manual rows (GNOME, KDE, sway) in `docs/release.md` — Executor: builder · Reviewer: thinker · Effort: low

**Checkpoint**: the journeys and suites in `qa.md` are green in CI; the exit criteria are met → Verified in `specs/README.md`.

---

## Convergence — 2026-10-23

Converged. FR-001–FR-005 map to `src/install/autostart.rs` (templates, atomic write, XDG and systemd flows, `detect` for doctor) and the `install-autostart` arm in `src/main.rs`. Every path and the `systemctl` program are injected through `Env`, and only `run()` reads the process environment. The 15 tests in `tests/install_autostart.rs` use temp dirs and a fake `systemctl`. Readings the spec left open: `--systemd` without `--now` also runs `daemon-reload` and `enable` (otherwise the unit would never start); systemd availability is probed with `systemctl --user show-environment` (exit 5, nothing written); `--uninstall` with `--systemd` or `--now` is a usage error. Unverified until QA: real re-login on GNOME, KDE and sway, and a real `systemctl --user`. The QA tasks (T008, T011) are deferred to before Verified.
