# Research: Release & publishing

Phase 0 for `013-release`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Static musl

- **Decision**: rustup musl targets on native runners; Rust's self-contained musl CRT + rust-lld (pure-Rust deps → no C toolchain needed)
- **Alternatives rejected**: cross/QEMU (slow), zigbuild (extra tool)

## R2. Publishing auth

- **Decision**: crates.io Trusted Publishing via `rust-lang/crates-io-auth-action` (OIDC → short-lived token)
- **Alternatives rejected**: `CARGO_REGISTRY_TOKEN` secret (long-lived)

## R3. Headless sessions in CI

- **Decision**: sway `WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1` with XWayland; Xvfb; bridge mode = sway with data-control protocols disabled via config/env
- **Alternatives rejected**: real GNOME in CI (not feasible headless → manual matrix)
