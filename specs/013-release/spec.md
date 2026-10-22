# Feature Specification: Release & publishing

**Feature Branch**: `ci/release`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief J10, K9, §11 Publishing, §14; ADR-0009; Constitution I, IX.

> Tests are **required** (Constitution II). For tooling, the integration suites below take the place of e2e journeys and run in CI (pipeline rule 6).

## Clarifications

### Session 2026-10-21

- Q: GPL-3.0-only or -or-later? → A: `GPL-3.0-or-later`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Every change is checked on Tier-1 sessions (Priority: P1)

Every PR runs format, lint, unit and property tests, fitness functions, and integration suites on headless sway and Xvfb, for x86_64 and aarch64.

**Why this priority**: Constitution II, and fitness functions F1–F11.

**Independent Test**: Open a PR that breaks a module boundary. CI fails at F1.

**Acceptance Scenarios**:

1. **Given** a PR, **When** CI runs, **Then** `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo deny check`, the integration suites (sway headless, Xvfb, headless mutter for bridge mode) and the musl static build checks (F2, F5) run on both architectures.
2. **Given** any job fails, **When** the PR is viewed, **Then** it can't be merged (branch protection, set up by `devsecops:pipeline`).

---

### User Story 2 - A tag publishes everything without a stored token (Priority: P1)

Pushing tag `X.Y.Z` builds and tests both musl targets, publishes the crate through Trusted Publishing, and creates the GitHub release with the tarballs and `SHA256SUMS`.

**Why this priority**: J10, and brief §14 ("no token stored").

**Independent Test**: A dry run of the workflow (`workflow_dispatch` with `dry_run: true`) produces the artifacts and runs `cargo publish --dry-run`.

**Acceptance Scenarios**:

1. **Given** tag `0.1.1` on `main`, **When** the release workflow runs in the protected `release` environment, **Then** it exchanges OIDC for a short-lived crates.io token (`rust-lang/crates-io-auth-action`), publishes, then creates the GitHub release.
2. **Given** publishing fails, **When** the workflow continues, **Then** no GitHub release is created.
3. **Given** the workflow file, **When** it's inspected, **Then** it references no `secrets.*` publishing token.

---

### User Story 3 - The first release by hand, then token retirement (Priority: P2)

A runbook guides the first manual publish and the switch to Trusted Publishing.

**Why this priority**: crates.io has no pending publisher for new crates (brief §11).

**Independent Test**: The runbook steps are executable in order. `cargo publish --dry-run` passes on the release commit.

**Acceptance Scenarios**:

1. **Given** the runbook `docs/release.md`, **When** the maintainer follows it, **Then** they run `cargo publish --dry-run`, publish `0.1.0` with their token (read from their password store at the moment of use, never written to the repository or the shell history), configure the trusted publisher (repository, workflow file and environment `release`), push `0.1.1`, and after it succeeds revoke the token and delete the password-store entry.

### Edge Cases

- The arm64 runner is unavailable: fall back to `cross` for aarch64 (ADR-0009).
- A tag not on `main`: the release workflow refuses.
- The crate version doesn't match the tag: the workflow fails before building.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: CI MUST run on PRs into `dev` and `main`, and on pushes to `dev`/`main`, with jobs for x86_64 and aarch64.
- **FR-002**: The integration jobs MUST start headless sway (`WLR_BACKENDS=headless`, with XWayland), Xvfb, and headless mutter (no data-control, XWayland on demand: bridge mode; `--no-x11`: the no-capture-path case).
- **FR-003**: The release workflow MUST run on tags matching `[0-9]+.[0-9]+.[0-9]+`, in the `release` environment, with `id-token: write`, and publish before creating the GitHub release.
- **FR-004**: Artifacts MUST be `rstory-X.Y.Z-<arch>-unknown-linux-musl.tar.gz` (binary, LICENSE, README) plus `SHA256SUMS`.
- **FR-005**: `Cargo.toml` MUST declare `license = "GPL-3.0-or-later"`, and the README MUST state it.
- **FR-006**: The release checklist (`docs/release.md`) MUST include the manual Tier-1 matrix: GNOME 48–51 (extension, bridge capture with GUI apps, brief Q7), KDE, sway with waybar.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The second and later releases are published with no publishing token stored anywhere in the repository or its settings.
- **SC-002**: Both artifacts pass F2 (static) and F5 (≤ 15 MiB).
- **SC-003**: A PR CI run completes in under 15 minutes.

## Assumptions

- The workflow details (job graph, caching, pinning) are designed by `devsecops:pipeline` and hardened by `devsecops:supply-chain`.
