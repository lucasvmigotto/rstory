# Research: Daemon & control socket

Phase 0 for `004-daemon`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Event loop

- **Decision**: tokio current_thread + `AsyncFd` for display fds, `UnixListener`, `time::sleep_until` one-shot timers (ADR-0004)
- **Alternatives rejected**: multi-thread runtime (RSS, no need)

## R2. Single instance

- **Decision**: store LOCK_EX (authoritative) then bind; stale socket unlinked only when lock acquired
- **Alternatives rejected**: pidfile

## R3. Burst handling

- **Decision**: bounded mpsc(256); writer drains all pending and commits one batch
- **Alternatives rejected**: unbounded queue (memory), drop-newest (loses the latest = what the user wants)

## R4. Peer auth

- **Decision**: `SO_PEERCRED` uid == geteuid()
- **Alternatives rejected**: filesystem perms only (defense in depth wants both)

## R5. Timers

- **Decision**: single `next_deadline = min(next_expiry, paused_until)` re-armed after each mutation
- **Alternatives rejected**: periodic tick (violates G6)

## R6. Logging

- **Decision**: `tracing` with fields; a `Redacted<T>` wrapper type for anything content-derived prints `<n bytes>`
- **Alternatives rejected**: ad-hoc eprintln
