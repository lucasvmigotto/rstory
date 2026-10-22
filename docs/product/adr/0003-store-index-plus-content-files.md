---
status: proposed
date: 2026-10-21
---
# 0003 — Store: an atomic index file plus write-once content files

## Context and problem statement
The history (20 entries by default, at most 500 configured, each up to
1 MiB) must survive crashes without corruption (brief §11), must be
deletable from disk for real (BR10), must be private (`0600`), and must
be readable by clients while the daemon is down. The store has to be pure
Rust.

## Decision drivers
- Crash safety, with RPO at most the in-flight entry
- Real deletion: nothing should linger in free pages or a WAL
- Simplicity at a few hundred records
- A pure-Rust, static build

## Considered options
1. `index.json` (atomic temp + fsync + rename) plus `content/<id>` files
   written once.
2. An embedded KV database (redb, or sled, which is unmaintained).
3. An append-only log with compaction.
4. SQLite (bundled C), which is rejected by the brief.

## Decision outcome
Chosen: **option 1**. The index is the source of truth. A startup sweep
removes orphan content files and index rows whose content file is
missing. Reads take a shared `flock`, and the daemon holds an exclusive
one.

### Consequences
- Good: unlinking a content file really removes the content, and nothing
  survives in a database page or a WAL.
- Good: it can be inspected with `cat`/`jq`, and needs no extra
  dependency.
- Bad: the whole index is rewritten per change, which is fine up to
  about 5 000 entries (trigger E3).
- Bad: two fsyncs per capture. Bursts are batched (architecture §4).

### Confirmation
proptest crash-injection tests (kill between steps) never leave an
unreadable store. The F9 permission test passes.
