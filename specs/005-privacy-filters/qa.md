# QA: Privacy filters

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: High

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 sensitive | property | `tests/filter_props.rs::hinted_never_persisted` |
| US2 oversize | unit | `domain::filter::tests::boundaries` |
| US3 paused | unit | `domain::filter::tests::paused_no_read` |
| SC-001 canary | fitness | `tests/fitness/canary.rs` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K3 | Sensitive copy | copier | Xvfb, sway, bridge | copier offers text + `x-kde-passwordManagerHint` with a canary; then `rstory ls --json` and grep the store dir and the logs for the canary | not in `ls`; canary absent from the store files and logs; `status.counters.skipped_sensitive == 1` |
| E2E-K3b | Timed pause around pass | owner, copier | Xvfb | `rstory pause --for 3s && copier sets S`; wait for the daemon's pause end (poll `status` until recording, max 5 s); copier sets T | S absent, T present; `skipped_paused == 1`. Negative: daemon down → `pause` exits 3 and copier not run |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

*Explore* privacy filters *with* adversarial inputs *to discover* invariant violations: Hint with odd casing (must NOT match, documented), hint-only offers, hint arriving through the bridge, pause toggled mid-burst.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
