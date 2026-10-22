# Quickstart: Store & history

How to exercise this feature. Every command runs in the `tools` container (README, Development); no Rust toolchain is needed on the host.

```bash
make test        # unit, property (history, crash, power loss), fitness (boundaries, permissions) and store tests
make bench       # open/list/insert budgets (SC-003); fails if a p95 is over 2x its budget
```

Narrower runs, inside `make shell` (the `fault-injection` feature is test-only and is what the crash suites need):

```bash
cargo test --features fault-injection --lib store::                  # store unit tests
cargo test --features fault-injection --lib domain::history          # History boundaries (T011)
cargo test --features fault-injection --test fitness perms           # permissions, F9
cargo test --features fault-injection --test store_crash             # SC-001 crash and power-loss suite
PROPTEST_CASES=4096 cargo test --features fault-injection --test store_crash   # deeper run
cargo test --features fault-injection --test history_props           # SC-002 invariants
cargo test --features fault-injection --test store_readonly          # read-only handle (US3)
```

`BENCH_DIR=/path make bench` is not forwarded into the container; to benchmark a specific disk run `make shell` and `BENCH_DIR=/some/dir cargo bench --features fault-injection --bench store`. The bench prints the filesystem it ran on, because inserts are fsync-bound.

Inspecting a real store (once the daemon of spec 004 writes one):

```bash
jq . ~/.local/state/rstory/index.json
ls -l ~/.local/state/rstory/content
```
