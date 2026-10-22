# Research: Store & history

Phase 0 for `001-store`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Atomic file replacement on Linux

- **Decision**: temp file in the same dir → `fsync(file)` → `rename` → `fsync(dir)`
- **Alternatives rejected**: `O_TMPFILE` + `linkat` (not portable to all filesystems), writing in place (not crash-safe)

## R2. Locking

- **Decision**: `flock` via rustix on `store.lock`: LOCK_EX for the daemon (held for life), LOCK_SH|LOCK_NB for readers
- **Alternatives rejected**: fcntl record locks (released on any fd close — footgun), lock files with pid (stale handling)

## R3. Id format

- **Decision**: 26-char Crockford base32 of 48-bit ms timestamp + 80 random bits (ULID layout), implemented in ~40 lines
- **Alternatives rejected**: the `ulid` crate (extra dependency for trivial code), UUIDv7 (36 chars, worse short ids)

## R4. Hash

- **Decision**: BLAKE3 with the `pure` feature: no C/asm build, ample speed for ≤ 1 MiB
- **Alternatives rejected**: SHA-256 (slower, no benefit), xxhash (non-cryptographic: collisions craftable, de-dup could hide an entry)

## R5. Index encoding

- **Decision**: JSON (`serde_json`), human-inspectable, ≤ 250 KiB at 500 entries
- **Alternatives rejected**: postcard/bincode (opaque, marginal speed gain irrelevant at this size)

## R6. Crash-safety testing

- **Decision**: a `Fs` trait with a real impl and a fault-injecting impl (fail or abort after the N-th syscall); proptest over N
- **Alternatives rejected**: killing child processes (slow, flaky)
