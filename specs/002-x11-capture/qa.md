# QA: Session detection & X11 capture (incl. XWayland bridge)

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: High

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 capture | integration | `tests/x11_capture.rs::*` |
| US2 restore | integration | `tests/x11_serve.rs::*` |
| US3 selection | unit | `session::tests::table` |
| US3-AS3 bridge | integration | `tests/bridge_capture.rs` |
| SC-004 GNOME 48–51 | manual | `docs/release.md GNOME matrix` |

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

*Explore* session detection & x11 capture (incl. xwayland bridge) *with* adversarial inputs *to discover* invariant violations: Copy rapidly from several X11 clients while restoring; kill XWayland mid-transfer; copy 1 MiB + 1 B; owner refuses conversion.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass; manual matrix rows signed off.
