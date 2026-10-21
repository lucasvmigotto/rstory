# QA: Daemon & control socket

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: High

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1-AS1..5 | integration | `tests/daemon_capture.rs::*` |
| US2 | integration | `tests/daemon_restore.rs::*` |
| US3 | integration | `tests/daemon_pause.rs::* (paused clock)` |
| US4 | integration | `tests/daemon_status.rs::*, tests/daemon_socket.rs::*` |
| SC-001 F7 | perf | `tests/perf/capture_latency.rs` |
| SC-002 RSS | perf | `tests/daemon_rss.rs` |
| SC-003 F10 | perf | `tests/perf/idle.rs` |
| SC-004 contract | contract | `tests/contract/protocol.rs` |
| authz other uid | e2e | `E2E-PEER` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K1 | First run | owner | fresh XDG dirs, Xvfb | `install-autostart` → file written; start `rstory daemon`; `rstory doctor --json` → session/backend/store/daemon ok | doctor exit 0; desktop entry matches template |
| E2E-K4 | No capture path | owner | sway with data-control disabled and no XWayland | start daemon; `status --json`; `copy 0` on a prepared store | state inactive with reason `no_data_control_no_xwayland`; `copy` exits 7 |
| E2E-PEER | Intruder is rejected | intruder | daemon as owner | as `intruder`, connect to the socket path (with permissions loosened for the test) and send `status` | connection closed with no bytes received; the daemon logs `peer_rejected{uid}` |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

Burst profile: 500 copies in 1 s from a copier (arrival rate constant), with 5 concurrent `list` clients at 10 req/s. Thresholds: daemon responsive (list p95 < 50 ms during the burst), queue ≤ 256, `dropped_burst` reported, RSS ≤ 20 MiB, no crash. Soak (nightly): 30 min idle + 1 copy/min, RSS flat ± 1 MiB, wake-ups only at timers.

## Exploratory charter

*Explore* daemon & control socket *with* adversarial inputs *to discover* invariant violations: Malformed and oversize lines, 100 concurrent connections, stale socket after SIGKILL, clock jumps, pause across restart.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
