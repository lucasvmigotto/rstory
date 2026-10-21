# Data model: Daemon & control socket

## DaemonState

`Recording | Paused{until: Option<UTC>} | Inactive{reason}` — `Paused` persisted (index `state.pause`); `Inactive` derived from backend selection/connection.

## Counters (since start, not persisted)

`skipped_sensitive, skipped_oversize, skipped_paused, dropped_burst: u64`

## Connection

`{peer_uid, subscribed: Set<EventKind>, next_request_id}`

## Capture job

`{types, self_owned, received_at}` → filter (005) → `Record(bytes) | Skip*` → store batch

State transitions:

```
start → (select backend) → Recording | Inactive
Recording --pause--> Paused{None} --resume--> Recording
Recording --pause{for}--> Paused{t} --(t reached | resume)--> Recording
Inactive --(backend reconnects)--> Recording | Paused (restored from index)
```
