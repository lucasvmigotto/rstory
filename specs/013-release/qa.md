# QA: Release & publishing

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Medium

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 | pipeline | `deliberate-failure PR (F1 break) checklist` |
| US2 | pipeline | `release dry-run + no-token assertion` |
| US3 | review | `runbook walkthrough` |

## E2E journeys

None. A tooling or rendering feature: the suites in the traceability table take the journeys' place (pipeline rule 6).

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

*Explore* release & publishing *with* adversarial inputs *to discover* invariant violations: Tag on a non-main commit, version mismatch, arm64 runner unavailable.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass; manual matrix rows signed off.
