# QA: Command-line interface

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Medium

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 | integration | `tests/cli_browse.rs::*` |
| US2 | integration | `tests/cli_manage.rs::*` |
| US3 | integration | `tests/cli_readonly.rs::*` |
| US4 | integration | `tests/cli_status.rs::*` |
| SC-001 ls budget | perf | `benches/ls.rs` |
| SC-002 json | contract | `tests/contract/cli_json.rs` |
| SC-003 exit codes | integration | `tests/cli_exit_codes.rs` |

## E2E journeys

| Id | Journey | Personas | Environment | Steps | Expected (incl. negative) |
|---|---|---|---|---|---|
| E2E-K6 | Scripting | script | Xvfb | `ls --json | jq`, `show <shortid>` piped, `clear` without TTY (no --yes → exit 2; --yes → ok) | JSON validates; bytes exact; exit codes per contract |
| E2E-K3b | Timed pause around pass | owner, copier | Xvfb | `rstory pause --for 3s && copier sets S`; wait for the daemon's pause end (poll `status` until recording, max 5 s); copier sets T | S absent, T present; `skipped_paused == 1`. Negative: daemon down → `pause` exits 3 and copier not run |

The journeys live in `tests/e2e/<id>.rs`. They're owned by `qa:e2e` and shared across the features that list them.

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

N/A (risk below High, or covered by the manual matrix).

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
