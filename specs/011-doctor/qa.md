# QA: Doctor

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Low

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 | unit | `doctor::tests::healthy` |
| US2 | unit | `doctor::tests::failure_rows (8)` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K1 | First run | owner | fresh XDG dirs, Xvfb | `install-autostart` → file written; start `rstory daemon`; `rstory doctor --json` → session/backend/store/daemon ok | doctor exit 0; desktop entry matches template |
| E2E-K4 | No capture path | owner | headless mutter with `--no-x11` (no data-control, no XWayland) | start daemon; `status --json`; `copy 0` on a prepared store | state inactive with reason `no_data_control_no_xwayland`; `copy` exits 7 |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

N/A (risk below High, or covered by the manual matrix).

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
