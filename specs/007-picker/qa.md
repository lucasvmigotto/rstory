# QA: Terminal picker

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Medium

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 | component | `picker::tests::journey_* + snapshots` |
| US2 | component | `picker::tests::pin_* / delete_*` |
| US3 | component | `snapshots plain/mono/readonly/empty/too_small` |
| SC-001/002 F6 | perf | `benches/picker_first_frame.rs` |
| keyboard-only a11y | e2e | `E2E-K2 (pty, keys only)` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K2 | Recall via picker | owner, copier | Xvfb (and sway) | copier sets A then B; spawn `rstory pick` in a pty; type `A`'s prefix; Enter; read CLIPBOARD with a test client | clipboard == A; picker exit 0; `ls` shows A at index 0. Negative: daemon stopped → picker lists read-only, Enter exits 3, clipboard unchanged |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

N/A (risk below High, or covered by the manual matrix).

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
