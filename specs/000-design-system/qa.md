# QA: Terminal & menu design system

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Low

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 consistent surfaces | snapshot | `src/ui/tests.rs::* (tiers × widths × ascii)` |
| US2 icons | manual checklist | `icon review at 16 px mono` |

## E2E journeys

None. A tooling or rendering feature: the suites in the traceability table take the journeys' place (pipeline rule 6).

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

N/A (risk below High, or covered by the manual matrix).

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
