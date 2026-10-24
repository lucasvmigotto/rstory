# rstory — test strategy

Status: Draft
Last updated: 2026-10-21
Owner stage: `qa:strategy`

## 1. Scope and principles

rstory is a local daemon plus a CLI, a TUI and two menu surfaces. There
is no browser, server or staging environment. "End to end" here means the
**real binary in a real (headless) graphical session**: a copying client,
the daemon, a client surface, and a clipboard read-back. Constitution II
makes tests mandatory per story. Constitution III–V make privacy and
budgets testable gates (fitness functions F1–F11, architecture §10).

Assumptions (no questions needed, all derived from the inputs):
- `[ASSUMPTION]` Environments are local (tools container) and CI only.
  GNOME is tested manually before tags.
- `[ASSUMPTION]` UAT is the owner using v0.1 daily on GNOME 48 (brief §14:
  30 days).
- `[ASSUMPTION]` Release cadence is ad hoc (tags when ready). PR pipeline
  budget is < 15 min (spec 013 SC-003).

## 2. Risk per feature

Impact: privacy, data loss, the v0.1 gate, reputation. Likelihood:
novelty, protocol complexity, external churn.

| Feature | Impact | Likelihood | Risk | Why |
|---|---|---|---|---|
| 000 design-system | Low | Low | **Low** | rendering only |
| 001 store | High (data loss, lingering secrets) | Medium (crash safety) | **High** | invariants H1–H10, real deletion |
| 002 x11-capture | High (GNOME gate) | High (INCR, bridge, novelty) | **High** | protocol edge cases, GNOME path |
| 003 wayland-capture | High (KDE/sway Tier 1) | Medium | **High** | pure-Rust bindings maturity (S4) |
| 004 daemon | High (everything depends on it) | Medium | **High** | concurrency of capture vs commands, timers, socket hardening |
| 005 privacy-filters | High (secrets on disk) | Low (pure logic) | **High** | impact dominates |
| 006 cli | Medium | Low | **Medium** | parity and exit codes for scripts |
| 007 picker | Medium (primary journey UX) | Medium | **Medium** | terminal lifecycle, budgets |
| 008 picker-launch | Low | Medium (terminal zoo) | **Low** | table-driven |
| 009 tray | Low | Medium (host variance) | **Low** | convenience |
| 010 gnome-extension | Medium (v0.1 gate on GNOME) | High (Shell churn) | **Medium** | manual matrix |
| 011 doctor | Medium (operator trust) | Low | **Low** | table-driven |
| 012 autostart | Medium | Low | **Low** | templates |
| 013 release | High (supply chain, publishing) | Medium | **Medium** | OIDC and pinning |

High risk gets negative, concurrency, failure-mode and crash tests at
every layer, plus an exploratory charter.

## 3. Layers, owners, tools

| Layer | Covers | Owner | Tools | Location |
|---|---|---|---|---|
| Unit | pure domain (History H1–H10, RecordingState, filter, selection table, picker `update`, menu model, launch table) | build stages | `cargo test` | `src/**/tests` |
| Property | store and filter invariants, crash injection | build stages (001, 005) | `proptest` | `tests/*_props.rs`, `tests/store_crash.rs` |
| Component / snapshot | design-system widgets, picker states, CLI output, doctor output, menus | `frontend:tui` | `insta` + ratatui `TestBackend` | `src/ui`, `src/picker`, `tests/cli_*` |
| Integration | backends against real display servers, daemon with the fake backend, D-Bus tray | `backend:build` | Xvfb, headless sway (`WLR_BACKENDS=headless`) + XWayland, headless mutter (bridge mode; `--no-x11` for no capture path), `dbus-daemon --session` | `tests/{x11,wayland,bridge,daemon,tray}_*.rs` |
| Contract | protocol v1 messages, CLI `--json` shapes, shared fixtures with the JS client | `backend:build`, `frontend:tui`, 010 | a JSON Schema validator (dev-dep), gjs for the JS side | `tests/contract/`, `tests/fixtures/protocol/`, `extension/tests/` |
| **E2E journeys** | brief journeys K1–K7 with the real binary in headless sessions, driven through a pty | **`qa:e2e`** | `expectrl` (pty, dev-dep) + the session launchers from `tests/support/` | `tests/e2e/` |
| Accessibility | no color-only state (mono snapshots), `--plain` mode, keyboard-only journeys, menu text labels | `frontend:tui` (snapshots) + `qa:e2e` (keyboard journey) | insta mono/ASCII snapshots | as above |
| Performance | F5 binary size, F6 picker first frame, F7 capture-to-listed, F10 idle wake-ups, RSS, burst | `qa:load` (burst/RSS/idle), build stages (benches) | `cargo bench` (criterion-free std bench harness or `divan`), custom harness | `benches/`, `tests/perf/` |
| Security | F3 bans, F4 licences, `cargo audit`, secret scanning, peercred negative test, F11 canary | `devsecops:supply-chain` + build stages | cargo-deny, cargo-audit, gitleaks | CI |
| Fitness | F1 boundaries, F2 static, F9 perms, F11 canary | build stages | custom tests | `tests/fitness/` |
| Exploratory | charters for the High-risk features | owner / `qa:e2e` | session notes | `docs/product/qa/charters/` |
| Manual release matrix | GNOME 48–51 (extension, bridge with GUI apps), KDE Plasma, sway + waybar | maintainer | checklist | `docs/release.md` |
| UAT | daily use by the owner on GNOME 48 | owner | — | — |

## 4. Test data and personas

- **Data:** only synthetic. Builders: `EntryBuilder` (content, age,
  pinned), `HistoryBuilder` (n entries, k pinned, ages), `StoreFixture`
  (temp `XDG_STATE_HOME`, prebuilt `index.json` v1), and protocol fixture
  files. Canary strings (`RSTORY-CANARY-<uuid>`) for the no-content-in-logs
  tests. Never real clipboard contents.
- **Reset:** every test gets a fresh temp `XDG_RUNTIME_DIR`,
  `XDG_STATE_HOME`, `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, plus its own
  Xvfb display or sway socket.
- **Personas** (from the authz model in `domain-model.md` §8):

| Persona | Identity | Used for | How it "logs in" |
|---|---|---|---|
| `owner` | the uid running the daemon | every allowed operation | same uid; socket path from its runtime dir |
| `intruder` | a second uid (`nobody`, or a user created in the tools container) | negative peercred test: the connection is closed before any reply | `setpriv --reuid` / `runuser` in CI (container root) |
| `copier` | a test client process (X11 or Wayland) that sets the clipboard, optionally with `x-kde-passwordManagerHint` | capture journeys, sensitive journeys | connects to the test display |
| `script` | non-TTY invocations of the CLI | `--json`, `--yes`, no-TTY refusals | stdin and stdout pipes |

## 5. Environments

| Environment | What runs | Limits |
|---|---|---|
| Local tools container (`devcontainer:setup`, `make` recipes) | everything except GNOME/KDE GUI checks and cross-protocol copies on Mutter (headless mutter has no keyboard seat) | perf numbers relative to the host |
| CI x86_64 + arm64 (GitHub, ubuntu-24.04[-arm]) | unit, property, snapshots, contract, integration (Xvfb, sway, bridge), e2e, fitness, perf smoke, security | shared runners: perf thresholds have 2× headroom in CI; exact budgets on the release checklist |
| Owner's GNOME 48 machine + GNOME OS VMs (49–51) | manual matrix, UAT | manual |
| KDE Plasma VM, sway + waybar | manual matrix | manual |

## 6. Gates per pipeline stage (for `devsecops:pipeline`)

| Stage | Must pass | Thresholds |
|---|---|---|
| PR into `dev` | fmt, clippy (`-D warnings`), unit, property (1 000 cases), snapshots, contract, integration (Xvfb, sway, bridge), e2e smoke (K2, K3, K6), fitness F1/F2/F3/F4/F9/F11, cargo-audit, secret scan | 0 failures. Flake rate < 1% over 30 days. Duration < 15 min. |
| Push to `dev` | the PR set + full e2e (K1–K7 headless) + perf smoke (F5, F6 ≤ 200 ms in CI, F7 ≤ 200 ms in CI, F10) | same |
| PR `dev` → `main` | the push-to-dev set on both architectures + property with 10 000 cases + burst and RSS tests | same |
| Tag (release) | the main set + release dry-run checks + manual matrix signed off in the release PR | the manual checklist is complete |
| Scheduled (nightly) | property with 100 000 cases, cargo-audit, idle soak (daemon for 30 min: no wake-ups beyond timers, RSS flat) | report. A failure opens an issue (`repo:issues`). |

## 7. Exit criteria for Verified (per feature)

A feature is **Verified** when:
1. its `## QA` tasks are done;
2. its e2e journeys (or, for tooling features without journeys, the
   integration suites named in its `qa.md`) pass in CI;
3. its perf thresholds are met where it has them;
4. no open High-severity defect remains;
5. its accessibility snapshots (mono/ASCII/plain) pass;
6. for 002, 010 and 013, the manual matrix rows are signed off on the
   release checklist.

## 8. Flaky-test policy

- CI reports reruns (`cargo nextest` with `retries = 0` on PRs; on `dev`
  a single rerun is allowed, and **reported**, never silently green).
- A test that fails then passes on rerun twice in 14 days is
  **quarantined**: moved behind `#[ignore = "quarantine: <issue>"]`, with
  an owner and a 14-day deadline. Quarantined tests run nightly.
- Session-dependent tests (Xvfb, sway) use readiness probes (wait for the
  socket or display), never sleeps.

## 9. Metrics

- Escaped defects per release (found after the tag).
- Flake rate (reruns per 100 runs) per suite.
- Suite durations (PR < 15 min).
- Mutation score (`cargo mutants`) on `domain::history`, `domain::filter`
  and `store` ≥ 80%, checked monthly. Coverage is tracked but isn't a
  target.

## 10. What build stages must include

- `backend:build`: the property and crash suites (001), backend
  integration suites (002, 003), daemon integration and contract (004),
  filter proptest + canary (005), launch table (008), tray D-Bus (009),
  doctor matrix (011).
- `frontend:tui`: snapshot suites for 000, 006, 007, 010 (CLI part), 011,
  012, plus picker keystroke journeys.
- Every stage: test-first, per the tasks' story phases.
