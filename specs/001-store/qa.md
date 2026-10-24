# QA: Store & history

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: High

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1-AS1 survive restart | property | `tests/store_crash.rs::acknowledged_survive` |
| US1-AS2 crash between writes | property | `tests/store_crash.rs::orphan_swept` |
| US1-AS3 too open | integration | `tests/fitness/perms.rs::refuses_0755` |
| US2-AS1 de-dup | unit | `domain::history::tests::duplicate_refreshes` |
| US2-AS2 count cap | property | `tests/history_props.rs::cap_never_exceeded` |
| US2-AS3 pin cap | unit | `domain::history::tests::eleventh_pin_refused` |
| US2-AS4 expiry | unit | `domain::history::tests::expiry_spares_pinned` |
| US2-AS5 clear | unit | `domain::history::tests::clear_keeps_pinned / clear_all_removes_all` |
| US3 read-only | integration | `tests/store_readonly.rs::*` |
| SC-003 open budget | perf | `benches/store.rs` |

## E2E journeys

None. A tooling or rendering feature: the suites in the traceability table take the journeys' place (pipeline rule 6).

## Load profile

Burst insert: 1 000 inserts in a batch loop; assert caps hold and the batch commit stays ≤ 20 ms + N × 1 ms

## Exploratory charter

*Explore* store & history *with* adversarial inputs *to discover* invariant violations: Crash the daemon (SIGKILL) at random moments during a copy burst; restart; the store opens and the caps hold.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass.
