# Feature Specification: Privacy filters

**Feature Branch**: `feat/privacy-filters`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief J5, BR3, BR5, BR6, BR11, BR12, spike S2; Constitution III.

> Tests are **required** for every story (Constitution II). The "never persisted" invariants use property tests (fitness F8, F11).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Copies marked sensitive are never stored (Priority: P1)

When a password manager offers `x-kde-passwordManagerHint` with a copy, rstory skips it without reading the content.

**Why this priority**: J5 and G4. A leak here defeats the product's safety promise.

**Independent Test**: Feed the filter a change whose types include `x-kde-passwordManagerHint` (any value). Assert the decision is `SkipSensitive`, that `read_text` was never called, and that the counter went up.

**Acceptance Scenarios**:

1. **Given** a change offering `text/plain;charset=utf-8` and `x-kde-passwordManagerHint`, **When** it's filtered, **Then** it's skipped, its content is never read, and `skipped_sensitive` increments.
2. **Given** a change offering only `x-kde-passwordManagerHint` (S1 observation), **When** it's filtered, **Then** it's skipped the same way.
3. **Given** a change through the XWayland bridge with the hint among the X11 targets, **When** it's filtered, **Then** it's skipped.

---

### User Story 2 - Oversize copies are dropped, never truncated (Priority: P1)

A copy larger than the size cap is discarded whole and counted.

**Why this priority**: Owner decision (1 MiB), and BR6.

**Independent Test**: Fake backend returning 1 MiB + 1 byte. The decision is `SkipOversize`, nothing is stored, and the counter goes up.

**Acceptance Scenarios**:

1. **Given** the cap is 1 MiB, **When** a copy of exactly 1 MiB arrives, **Then** it's recorded.
2. **Given** the cap is 1 MiB, **When** a copy of 1 MiB + 1 byte arrives, **Then** the read stops at the limit, nothing is stored, and `skipped_oversize` increments.

---

### User Story 3 - Nothing is recorded while paused (Priority: P1)

While paused (indefinitely or timed), changes are counted and dropped before any read.

**Why this priority**: J6 and J6b.

**Independent Test**: Filter with the state paused: the decision is `SkipPaused`, there's no read, and the counter goes up.

**Acceptance Scenarios**:

1. **Given** the state is `paused`, **When** a change arrives, **Then** it's not read and `skipped_paused` increments.

### Edge Cases

- A change with no text type (an image only) is ignored with no counter (images are Next).
- An empty text is ignored.
- Hint matching is case-sensitive and exact on the type name. Unknown hint-like types aren't honored until they're added to the list (decision log).
- Check order: paused › sensitive › no-text › oversize › empty › self-owned › de-dup.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The filter MUST run, in order: pause gate, sensitive-hint gate, text-type gate, read with the size limit, empty check, then hand over to the store (self-owned and de-dup are handled in specs 004 and 001).
- **FR-002**: The sensitive-hint list MUST contain `x-kde-passwordManagerHint` and be defined in one place (a constant plus a test). Extending it is a spec change.
- **FR-003**: The pause and sensitive gates MUST decide on the offered types alone, without reading content.
- **FR-004**: Content read for a change MUST be dropped from memory as soon as it's rejected.
- **FR-005**: Counters MUST be incremented for skipped sensitive, oversize and paused changes.
- **FR-006**: No filter decision MUST log content. Decision, size and type names only.
- **FR-007**: The README MUST document the threat model and the `passc` wrapper (`rstory pause --for 50s && pass -c "$@"`), because `pass` sets no hint.

### Key Entities

- **Filter decision**: `Record(text) | SkipPaused | SkipSensitive | SkipOversize | IgnoreNoText | IgnoreEmpty`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 0 hinted contents reach disk, memory past the decision, or logs, across 10 000 property-test runs and the canary log test (F11).
- **SC-002**: 100% of size-boundary tests (cap − 1, cap, cap + 1) behave as specified.

## Assumptions

- Further hints found by the remaining S2 tests (KeePassXC, Bitwarden, 1Password, Chromium, Firefox) are added through a spec amendment.
