# Backend: Release & publishing

Layer added by `backend:spec` (2026-10-21). Canonical contract: `/contracts/protocol.md` + `/contracts/protocol.schema.json`.

## Endpoints / interfaces

N/A. This feature's "backend" layer is the CI/CD configuration, implemented by `devsecops:pipeline` (workflows) and `devsecops:supply-chain` (pinning, scanning).

## Authorization matrix

Release environment: the maintainer as the required reviewer.

## Persistence

N/A.

## Transactions & concurrency

Release workflow `concurrency: release` group (one at a time).

## Async work

N/A.

## Integrations

crates.io (Trusted Publishing), GitHub Releases. If publishing fails → no release.

## Observability

Workflow summaries; `devsecops:watch` follows runs.

## Security

No stored publishing token; `id-token: write` only in the publish job; protected `release` environment; actions pinned by SHA (`devsecops:supply-chain`).

## Performance

PR CI < 15 min.

## Test plan

actionlint, dry-run release, the deliberate-failure PR checklist.
