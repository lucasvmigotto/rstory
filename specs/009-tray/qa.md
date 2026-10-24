# QA: SNI tray

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Low

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 | unit+integration | `tray::tests::*, tests/tray_dbus.rs` |
| US2 | unit | `tray::tests::states` |
| US3 | integration | `tests/tray_dbus.rs::no_host / appears` |
| SC-001 | manual | `docs/release.md KDE + waybar` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K7 | Tray | owner | Xvfb + dbus-daemon + stub SNI watcher | daemon registers item; fetch the menu layout over D-Bus; activate entry 2; activate Pause; read the icon pixmap name/state | clipboard == entry 2; status paused; item label `Resume recording`; icon variant paused |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

N/A (risk below High, or covered by the manual matrix).

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass; manual matrix rows signed off.
