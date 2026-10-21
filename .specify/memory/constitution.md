<!--
Sync Impact Report
- Version: (template) → 1.0.0 (initial ratification)
- Principles added: I–IX (all new)
- Default principles cut: "Contract-First API" (replaced by IV, since there is no
  HTTP API, the seam is the control-socket protocol); "Security Baseline (ASVS)"
  (no web surface; its local equivalent is folded into III); "Observability"
  as a separate principle (folded into III, no content in logs, and VIII, doctor and
  status counters); RED metrics and readiness probes don't apply to a local tool.
- Templates: plan-template Constitution Check derives from this file; no template edits needed.
-->

# rstory Constitution

## Core Principles

### I. Zero Runtime Dependencies

rstory MUST ship as one statically linked binary per target
(`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`) that needs
nothing at runtime beyond the kernel, the session's display server and the
D-Bus session bus. Dependencies MUST be pure Rust: no crate that links
`libwayland`, `libxcb`, `libdbus`, OpenSSL, GTK, Qt or a bundled C database.
A crate compiled with `cc` needs an explicit allow-list entry with a reason.
Source: brief G1, ADR-0001, ADR-0006.

**Checked by**: architecture fitness F2 (static binary) and F3
(`cargo deny check bans`) in CI. The plan lists every new dependency with
its pure-Rust status.

### II. Tests Required per Story (Test-First)

Every user story ships with automated tests that are written to fail before
the code that satisfies them. A story is done only when its tests pass in
CI. This overrides the Spec Kit tasks template, which calls test tasks
optional. Store invariants use property tests. UI states use snapshot
tests. Capture and restore use integration tests on headless sessions.

**Checked by**: test tasks present in every story phase of `tasks.md`,
preceding implementation tasks. `cargo test` plus the integration suites
are green in CI.

### III. Privacy by Default

- Content offered with a sensitive hint MUST never be read, stored or logged.
- Entry content MUST never appear in logs, errors, `doctor` output or panics.
- The store MUST be `0700` (directory) and `0600` (files), and the control
  socket MUST accept only the daemon's own uid.
- rstory MUST make no network connection and send no telemetry.
- Deletion MUST remove content from disk, not only hide it.
- The source application MUST NOT be recorded.

Source: brief G4, BR5, BR10–BR12, BR17.

**Checked by**: fitness F8 (proptest: hinted content never persisted), F9
(permissions), F11 (canary string absent from trace logs), F3 (network
crates banned). Each feature's plan has a privacy section.

### IV. Single Writer, Versioned Protocol

The daemon is the only process that writes the store or owns the
clipboard. Every client (CLI, picker, tray, GNOME extension) MUST go
through the control-socket protocol (`contracts/protocol.md` +
`contracts/protocol.schema.json`). Clients MAY only open the store
read-only, and only while the daemon is down. Changing the protocol is a
spec change made through `backend:spec`. A breaking change bumps the
protocol version `v`. Source: ADR-0002, ADR-0003, brief BR14.

**Checked by**: protocol contract tests (every message validated against
the JSON Schema, with fixtures shared by the Rust client and the GNOME
extension), and fitness F1 (no client module imports store write APIs or
backends).

### V. Budgets Stated Up Front

Every operation with an NFR records its budget in the plan and has a
check:
- capture-to-listed p95 < 100 ms;
- picker first frame p95 < 100 ms;
- `ls` < 50 ms;
- daemon RSS ≤ 20 MiB;
- binary ≤ 15 MiB;
- idle with no polling (timers only for expiry or timed pause).

Source: brief §11.

**Checked by**: fitness F5, F6, F7 and F10 in CI or on the release
checklist. The "Performance Goals" in each plan cite the budget.

### VI. CLI Parity and Scriptability

Every action available in the picker, the tray or the GNOME extension MUST
be available as a CLI command. Listing commands MUST offer `--json` with a
documented, versioned shape. Exit codes MUST be documented and stable
within a major version. Machine output goes to stdout and diagnostics to
stderr. Source: brief G5.

**Checked by**: the parity table in spec 006 (CLI), tests asserting the
`--json` shape against the schema, and exit-code tests.

### VII. Accessible Interfaces

The picker MUST be fully keyboard-operable. State MUST never be conveyed by
color alone (glyphs or labels for selected, pinned and paused). A plain
mode without box drawing MUST exist for screen readers. Tray and panel
menu items MUST carry text labels. These are WCAG 2.2 AA principles
applied to terminal and menu surfaces. Source: brief §11.

**Checked by**: picker snapshot tests in color and `NO_COLOR`/plain mode
(each state distinguishable without color), keyboard-only journey tests,
and a menu-label review in specs 009 and 010.

### VIII. Diagnosable, Honest Failures

rstory MUST NOT pretend to work. When capture is impossible, the daemon
reports *inactive*. Every failure rstory can detect has a plain-words
`doctor` message with a next step. `status` exposes counters (skipped
sensitive, oversize, paused, dropped burst). Logs go to stderr or the
journal through `tracing`, with no content (III). Source: brief J9,
journeys K4 and K8.

**Checked by**: the doctor failure matrix in spec 011, with one test per
detectable failure, and the `status` counter tests in spec 004.

### IX. Versioning and Review

Conventional Commits, a branch per context, work entering `dev` and moving
up one level at a time (`git:workflow`). Releases use plain SemVer tags
(`X.Y.Z`) and are published only from CI through Trusted Publishing after
the first manual release (ADR-0009). Architecture or protocol changes need
an ADR or a spec change before merge.

**Checked by**: `git:workflow`'s guard hook, the release workflow having no
stored publishing token (`devsecops:pipeline`), and PR review of
`docs/product/adr/` and `contracts/` changes.

## Constraints

- **Platforms:** Linux only, x86_64 and aarch64, glibc or musl, any init
  system. Release artifacts are static musl binaries.
- **Tier-1 environments (release-blocking):** GNOME Shell 48–51 on Wayland
  (XWayland bridge plus the extension), KDE Plasma Wayland, a wlroots
  compositor (sway), X11.
- **Privacy regime:** none applies (local processing of the user's own
  data, no transfer). The threat model is documented in the README.
- **Licence:** GPL-3.0 (see LICENSE). Every dependency must be compatible.
- **Interface language:** English only for v1.
- **No root:** install, autostart and extension setup run as the user.
- **No network:** at runtime, ever.

## Governance

This constitution outranks habit, README prose and a plan's convenience. A
plan that violates a principle MUST fill the template's Complexity
Tracking table with the violation, why it is needed, and the simpler
alternative it rejected. An unjustified violation fails the plan's
Constitution Check.

Amendments bump the version: MAJOR when a principle is removed or
redefined, MINOR when one is added or materially expanded, PATCH for
clarifications. Each amendment records what changed in the Sync Impact
Report. Every plan's Constitution Check and every `/speckit-converge`
finding cites a principle by number ("Constitution III").

**Version**: 1.0.0 | **Ratified**: 2026-10-21 | **Last Amended**: 2026-10-21
