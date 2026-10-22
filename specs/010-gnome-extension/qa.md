# QA: GNOME Shell extension

Layer added by `qa:strategy` (2026-10-21). See `docs/product/test-strategy.md`.

**Risk**: Medium

## Traceability

| Story / scenario / SC | Layer | Test (planned) |
|---|---|---|
| US1 install | integration | `tests/install_gnome.rs` |
| US2 menu | contract | `extension/tests/client.test.js` |
| SC-001 Shell 48–51 | manual | `docs/release.md GNOME matrix` |

## E2E journeys

None. A tooling or rendering feature: the suites in the traceability table take the journeys' place (pipeline rule 6).

## Load profile

No load targets beyond the budgets in the traceability table.

## Exploratory charter

*Explore* gnome shell extension *with* adversarial inputs *to discover* invariant violations: On each Shell version: daemon restart while the menu is open, disable/enable cycles (leak check in Looking Glass), light/dark panel, pause from the CLI reflected in the icon.

## Exit criteria (Verified)

Per `test-strategy.md` §7: QA tasks done; the journeys or suites above green in CI; perf thresholds met; no open High-severity defect; mono/ASCII snapshots pass; manual matrix rows signed off.
