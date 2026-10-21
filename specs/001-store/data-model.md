# Data model: Store & history

## Entry

| Field | Type | Rules |
|---|---|---|
| `id` | EntryId (26-char ULID-like) | unique, time-ordered, immutable |
| `hash` | 32-byte BLAKE3, hex in JSON | unique across the history (de-dup key) |
| `size` | u32 bytes | 1 ≤ size ≤ `max_entry_bytes` |
| `created` | UTC timestamp | first time this content was recorded |
| `last_copied` | UTC timestamp | refreshed on re-copy and on restore (BR1, BR2); ordering key |
| `pinned` | bool | at most `max_pinned` true |
| `preview` | string ≤ 256 chars | first chars, control characters escaped (`\n`, `\t`, `\xNN`) |

Never stored: source application, window, process (BR11).

## History (aggregate)

- Ordered by `last_copied` descending; `index` is the position (0 = newest).
- Invariants: `len ≤ max_entries`; `count(pinned) ≤ max_pinned < max_entries`; hashes unique; after housekeeping no unpinned entry has `now − last_copied > ttl`.
- Operations: `insert(content, now) → Inserted | Refreshed(id)`, `touch(id, now)` (restore), `pin(id) → Ok | PinLimit`, `unpin(id)`, `remove(id)`, `clear() → n` (unpinned), `clear_all() → n`, `expire(now) → [id]`, `next_expiry() → Option<instant>`.
- Eviction on insert: while `len > max_entries`, remove the oldest unpinned.

## PauseState (persisted with the index, used by 004)

`Recording | Paused { until: Option<UTC> }`

## ShortId

Shortest prefix of `id` (≥ 4 chars) unique within the current history; recomputed per listing.

## On-disk index (v1)

```json
{"version":1,"entries":[{"id":"01JB…","hash":"…","size":42,"created":"…","last_copied":"…","pinned":false,"preview":"git rebase -i HEAD~3"}],
 "state":{"pause":{"kind":"recording"}}}
```

## Caps (config, validated)

| Key | Default | Range |
|---|---|---|
| `max_entries` | 20 | 2–500 |
| `max_pinned` | 10 | 0 – `max_entries`−1 |
| `max_entry_bytes` | 1048576 | 1024–1048576 |
| `ttl` | "7d" | 1h–90d |
