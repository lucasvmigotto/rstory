# QA: Wayland data-control capture

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: High

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 capture | integration | `tests/wayland_capture.rs::*` |
| US2 restore | integration | `tests/wayland_serve.rs::*` |
| SC-003 KDE | manual | `docs/release.md KDE row` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K2 | Recall via picker | owner, copier | Xvfb (and sway) | copier sets A then B; spawn `rstory pick` in a pty; type `A`'s prefix; Enter; read CLIPBOARD with a test client | clipboard == A; picker exit 0; `ls` shows A at index 0. Negative: daemon stopped → picker lists read-only, Enter exits 3, clipboard unchanged |
| E2E-K3 | Sensitive copy | copier | Xvfb, sway, bridge | copier offers text + `x-kde-passwordManagerHint` with a canary; then `rstory ls --json` and grep the store dir and the logs for the canary | not in `ls`; canary absent from the store files and logs; `status.counters.skipped_sensitive == 1` |
| E2E-K5 | Switch session | owner, copier | Xvfb then sway, same XDG dirs | record A under X11; stop; start under sway; `ls`; record B | both A and B listed; backend changed in status |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

*Explore* wayland data-control capture *with* adversarial inputs *to discover* invariant violations: Readers that close early, slow sources (> 2 s), compositor exit during serve.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass; manual matrix rows signed off.
