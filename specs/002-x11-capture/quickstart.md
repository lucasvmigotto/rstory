# Quickstart: Session detection & X11 capture (incl. XWayland bridge)

How to exercise this feature once built. Everything runs in the `tools` container through the
make recipes (pass `TARGET_VOLUME=rstory-tools-target-<name>` from a parallel worktree).

```bash
make test                                               # unit tests: selection table, target ranking, Latin-1, FakeBackend
make test-integration TEST_ARGS="--test x11_capture"    # US1, each test on its own Xvfb
make test-integration TEST_ARGS="--test x11_serve"      # US2, incl. the 1 MiB INCR round trip
make test-integration TEST_ARGS="--test bridge_capture" # US3: headless mutter, no data-control, XWayland on demand
make test-integration TEST_ARGS="--test wayland_probe"  # registry probe against headless sway and mutter
make test-integration                                   # all of the above, offline
```

On GNOME (manual): run the daemon and copy from a GUI app (see `docs/release.md`, "GNOME 48–51 manual check").

```bash
RUST_LOG=rstory=debug cargo run -- daemon   # logs: backend=x11 mode=xwayland_bridge
```
