---
status: proposed
date: 2026-10-21
---
# 0009 — Release: static musl builds on native runners; crates.io Trusted Publishing

## Context and problem statement
Releases need static binaries for x86_64 and aarch64 musl with checksums,
plus a crates.io publish with no long-lived token (brief §11, J10). The
first publish must be manual (crates.io has no pending publisher).

## Considered options
- Builds: **native `ubuntu-24.04` and `ubuntu-24.04-arm` runners with
  `rustup target add <arch>-unknown-linux-musl`** (pure-Rust deps, linked
  with the self-contained musl CRT) versus `cross` (Docker/QEMU) versus
  cargo-zigbuild.
- Publish: **`rust-lang/crates-io-auth-action` (OIDC → short-lived
  token) in a protected `release` environment** versus a stored
  `CARGO_REGISTRY_TOKEN`.

## Decision outcome
Chosen: the options in bold. On a tag `X.Y.Z` (plain SemVer): build and
test both targets, then `cargo publish` through Trusted Publishing, then
create the GitHub release with the tarballs and `SHA256SUMS`. If
publishing fails, no GitHub release is created. The first release
follows a manual runbook (`cargo publish --dry-run`, then a publish with
the owner's token). The token is revoked after the first CI release
succeeds. The workflow details belong to `devsecops:pipeline`.

### Consequences
- Good: no emulation, fast builds, and no stored publishing secret.
- Bad: depends on GitHub's arm64 runners (AA4). The fallback is `cross`.

### Confirmation
F2 on both artifacts. The release job has no `secrets.*` publishing
token. crates.io shows the version published by the trusted publisher.
