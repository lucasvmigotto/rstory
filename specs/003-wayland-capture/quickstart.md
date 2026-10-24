# Quickstart: Wayland data-control capture

How to exercise this feature once built. Everything runs in the `tools` container through the
make recipes (pass `TARGET_VOLUME=rstory-tools-target-<name>` from a parallel worktree). Each
test starts its own headless sway.

```bash
make test                                                # unit tests: MIME ranking, decode, served set
make test-integration TEST_ARGS="--test wayland_capture" # US1: ext (skipped if absent) and zwlr paths, hint, clear, limit, timeout
make test-integration TEST_ARGS="--test wayland_serve"   # US2: serve, 1 MiB round trip, cancelled, EPIPE
make test-integration                                    # everything, offline
```

On KDE (manual): see `docs/release.md`, "KDE Plasma Wayland manual check".

```bash
RUST_LOG=rstory=debug cargo run -- daemon   # logs: backend=wayland mode=native
```

Note: the sway in the tools image (1.10, wlroots 0.18) offers only `zwlr_data_control_manager_v1`,
so the `ext` test reports SKIPPED there.
