# Feature Specification: Store & history

**Feature Branch**: `feat/store`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Brief §6 (Privacy & storage), §8 (entry lifecycle), BR1, BR6–BR10, BR14, BR15; ADR-0003.

> Tests are **required** for every story (Constitution II). Store invariants use property tests.

## Clarifications

### Session 2026-10-21

- Q: Do pinned entries count toward the 20-entry cap? → A: Yes, 20 in total. Pins count, but the cap only ever evicts unpinned entries.
- Q: Can pinned entries be wiped in bulk? → A: Yes. `clear --all` removes pinned entries too, after a confirmation. Plain `clear` keeps them.

### Session 2026-10-22

- Q: What happens when `max_pinned` or `max_entries` is lowered below the pins already stored? → A: Existing pins are kept: they're never evicted, expired or unpinned because caps shrank. New pins are refused while pinned ≥ `max_pinned`. The count cap evicts only unpinned entries, so the history may exceed `max_entries` only by pinned entries. `doctor` and `status` report the over-cap counts. Caps are re-applied to unpinned entries when the store opens.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Recorded entries survive restarts (Priority: P1)

The daemon records text entries. After a restart, or a crash in the middle of a write, the history is intact and in the same order.

**Why this priority**: Without a durable history there's nothing to recall (J1).

**Independent Test**: Insert entries through the store API, kill the process at random points during writes, reopen, and assert that the history is readable and that every completed entry is present.

**Acceptance Scenarios**:

1. **Given** an empty store, **When** entries A then B are inserted, **Then** the history lists B (index 0), then A (index 1), and both survive reopening.
2. **Given** a crash after the content file is written but before the index is renamed, **When** the store reopens, **Then** the history doesn't contain the partial entry and the orphan content file is removed.
3. **Given** a store directory with mode `0755`, **When** the daemon opens it for writing, **Then** opening fails with a "store too open" error and nothing is written.

---

### User Story 2 - The history stays small and fresh (Priority: P1)

The history applies de-duplication, the count cap, the pin cap and expiry, so it stays a short-term assistant.

**Why this priority**: These are owner decisions (20 entries, 10 pins, 7 days, 1 MiB) and the privacy guarantees of G4.

**Independent Test**: Property tests over random sequences of insert, pin, unpin, rm, clear and clock advances, asserting the invariants after every step.

**Acceptance Scenarios**:

1. **Given** A, B in the history, **When** A is inserted again, **Then** there are still 2 entries and A is at index 0 with a refreshed `last_copied`.
2. **Given** 20 entries (none pinned), **When** a 21st is inserted, **Then** the oldest is deleted from disk and 20 remain.
3. **Given** 10 pinned entries, **When** an 11th pin is requested, **Then** it's refused with `pin_limit` and nothing changes.
4. **Given** an unpinned entry last copied 7 days and 1 second ago, **When** housekeeping runs, **Then** the entry and its content file are deleted. A pinned entry of the same age stays.
5. **Given** pinned and unpinned entries, **When** `clear` runs, **Then** all unpinned entries are deleted from disk and the pinned ones remain.

---

### User Story 3 - Clients read while the daemon is down (Priority: P2)

`ls`, `show` and the picker can read the history without the daemon, and can never write it.

**Why this priority**: Decision 2026-10-21 (BR14).

**Independent Test**: Open the store read-only while no writer holds the lock. List and show work, and every write API is unavailable on the read-only handle.

**Acceptance Scenarios**:

1. **Given** no daemon, **When** a client opens the store read-only, **Then** it gets the history with previews and can load any entry's content.
2. **Given** the daemon holds the exclusive lock, **When** a client tries a read-only open, **Then** it's told to use the daemon (`daemon_running`). Clients prefer the socket in that case.

### Edge Cases

- An index with an unknown future `version` refuses to open, with a clear error, and never rewrites it.
- An index row whose content file is missing is dropped on open (logged by id only).
- A full disk during a write: the write fails, the previous index stays valid, and the daemon reports an error without crashing.
- Two entries with the same preview but different content are distinct (the hash decides).
- Clock moving backwards: expiry uses `last_copied` and never deletes entries whose timestamp is in the future (they're treated as now).
- An empty string copy is not recorded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The store MUST persist entries under `$XDG_STATE_HOME/rstory` (default `~/.local/state/rstory`) as `index.json` plus `content/<id>`, creating the directory as `0700` and files as `0600`.
- **FR-002**: Every mutation MUST be crash-safe: content written via temp + fsync + rename, then the index via temp + fsync + rename + directory fsync.
- **FR-003**: Opening MUST sweep orphans in both directions (content files without rows, rows without content).
- **FR-004**: The store MUST refuse to open for writing if the directory or any file is accessible by group or others, and report `store_too_open`.
- **FR-005**: De-duplication MUST use a BLAKE3 hash of the content. A duplicate insert moves the existing entry to index 0 and refreshes `last_copied`.
- **FR-006**: The count cap (default 20) MUST bound the total number of entries. When it's exceeded, the oldest unpinned entry is deleted. Pinned entries count toward the cap but are never evicted by it (with 10 pins, 10 unpinned slots remain).
- **FR-007**: The pin cap (default 10) MUST refuse further pins with `pin_limit`. The pin cap MUST be lower than the count cap. When stored pins already exceed a lowered cap, the existing pins MUST be kept and only new pins refused (owner decision 2026-10-22).
- **FR-018**: Opening the store MUST re-apply expiry and the count cap to unpinned entries durably, before any other operation. Recording and opening apply the same rule: unpinned entries ≤ max(`max_entries` − pinned, 1), so the newest unpinned entry (the current copy) is always kept, even when pins fill the cap. [DECIDED: 2026-10-22 by the orchestrating thinker, within the owner's pin policy; it keeps the latest copy restorable across restarts.]
- **FR-008**: Unpinned entries MUST expire 7 days (default) after `last_copied`. The store MUST expose the next expiry instant, so the daemon can arm a single timer.
- **FR-009**: `clear` MUST delete every unpinned entry from disk. `clear_all` MUST delete every entry, pinned included (the CLI asks for confirmation, spec 006).
- **FR-010**: `rm <id>` MUST delete one entry (pinned or not) from disk.
- **FR-011**: Content larger than the size cap (default 1 MiB) MUST be rejected by the store API (`too_large`). The filter normally catches it earlier (spec 005).
- **FR-012**: Each entry MUST hold: `id` (128-bit, time-ordered), `hash`, `size` (bytes), `created`, `last_copied`, `pinned`, `preview` (the first 256 chars, control characters escaped). It MUST NOT hold any source-application data.
- **FR-013**: The store MUST compute `short id`s as the shortest unique prefix of at least 4 characters.
- **FR-014**: A read-only open MUST take a shared lock and expose only read APIs. The daemon holds the exclusive lock for its lifetime.
- **FR-015**: Caps MUST be configurable (`max_entries` 2–500, `max_pinned` 0 to `max_entries`−1, `max_entry_bytes` 1 KiB–1 MiB, `ttl` 1 h–90 d). Out-of-range values are rejected at config load.
- **FR-016**: Batched mutations MUST be supported (several inserts, one index write), for capture bursts (architecture §4).
- **FR-017**: No API MUST log content. Logs carry ids, sizes and counts only.

### Key Entities

- **Entry**: one recorded text value with the attributes in FR-012.
- **History**: the ordered list of entries (by `last_copied` descending) plus the caps in force.
- **Store**: the on-disk representation (index + content files + lock).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 10 000 property-test runs with injected crashes, the store always reopens and every acknowledged mutation is present.
- **SC-002**: After any sequence of operations, the history never exceeds the count cap or the pin cap, and contains no unpinned entry older than the TTL after housekeeping.
- **SC-003**: Opening a full store (500 entries) and listing it takes under 10 ms on the reference laptop.
- **SC-004**: All store files are `0600` and the directory is `0700` after every operation (100% of permission tests).

## Assumptions

- Expiry counts from `last_copied` (brief A9).
- A single writer is guaranteed by the lock and by the daemon's single-instance check (spec 004).
- Timestamps are stored in UTC (RFC 3339).
