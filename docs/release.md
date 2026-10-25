# Release runbook

How rstory is checked and released: the pipelines and their gates, the one-time owner setup, the
first release by hand (brief K9), every later release from a version bump, and the Tier-1 manual matrix
that signs a release off. Spec 013 owns this page up to "Tier-1 manual matrix: sign-off"; the
manual check sections after it belong to the features named in them (002, 003, 009, 010, 012).

## Pipelines

Four workflows on GitHub Actions (ADR-0009), all running the project's `make` recipes in the
`tools` image, exactly as a developer does (README "Development"). A green `make` run locally
means the same as a green job in CI.

| Workflow | Runs on | Jobs |
|---|---|---|
| `.github/workflows/ci.yml` | PRs into `dev` and `main`; pushes to `dev` and `main`; called by `release.yml` | `plan`, `checks`, `lint`, `test`, `integration`, `e2e`, `static`, `perf`, `ci-ok`, `report-failure` |
| `.github/workflows/tag.yml` | pushes to `main`; manual `workflow_dispatch` from `main` (retry) | `tag`: tags the `Cargo.toml` version if it has no tag, then sends `repository_dispatch` `release` |
| `.github/workflows/release.yml` | tags `X.Y.Z` (plain SemVer, no `v`) pushed by hand; `repository_dispatch` `release` from `tag.yml`; manual `workflow_dispatch` (dry run only) | `verify`, `ci` (the main set), `package`, `SHA256SUMS`, `cargo publish --dry-run`, `publish`, `GitHub release`, `report-failure` |
| `.github/workflows/nightly.yml` | every day 03:17 UTC; manual | `property`, `soak`, `audit`, `quarantine`, `report-failure` |

Job graph of `ci.yml` (the `lint`, `test`, `integration`, `e2e`, `static` and `perf` jobs run once per
architecture: x86_64 on `ubuntu-24.04`, aarch64 on `ubuntu-24.04-arm`):

```text
plan ──┬─ test (x86_64, aarch64) ─────────┐
       ├─ integration (x86_64, aarch64) ──┤
       ├─ e2e (x86_64, aarch64) ──────────┤
       └─ perf (x86_64, aarch64)* ────────┤
checks (x86_64) ──────────────────────────┼─ ci-ok ── report-failure (branch pushes only, on failure)
lint (x86_64, aarch64) ───────────────────┤
static (x86_64, aarch64) ─────────────────┘
* perf runs on pushes to dev and for the main set; skipped on PRs into dev.
```

| Job | Recipes |
|---|---|
| `checks` | the SC-001 token assertion, `fmt-check`, `deny` (F3, F4), `audit` (RustSec), `secret-scan` (gitleaks), `ext-deps`, `ext-lint`, `ext-typecheck`, `ext-test` |
| `lint` | `lint` (clippy `-D warnings`, session tests compiled too) |
| `test` | `test` (unit, property, snapshot, contract, fitness F1, F9, F10, F11, RSS and burst tests; doctests) |
| `integration` | `test-integration` (Xvfb, headless sway with XWayland, headless mutter bridge mode) |
| `e2e` | `test-e2e-smoke` (K2, K3, K6) on PRs into `dev`; `test-e2e` (K1 to K7, PEER, launch table) on pushes to `dev` and for the main set |
| `static` | `build-static` (musl static build, F2 static, F5 at most 15 MiB) |
| `perf` | `bench` (store benchmarks and F6 picker first frame, p95 within 2x budget on shared runners), `test-perf` (real daemon on Xvfb: F7, burst, RSS, F10) |
| `ci-ok` | none: fails unless every gate above succeeded (`perf` may be skipped). **The one required status check.** |

Job graph of `release.yml`:

```text
verify ──┬─ ci (calls ci.yml, main set) ──────────────────────┐
         ├─ package (x86_64, aarch64) ── SHA256SUMS ──────────┼─ publish (env: release) ── GitHub release
         └─ cargo publish --dry-run ──────────────────────────┘
any failure on a tag ── report-failure
```

- `tag.yml` reads `version` from `Cargo.toml` on the pushed `main` commit. No tag `X.Y.Z`: it
  creates the annotated tag on that commit, pushes it with the built-in token (`contents: write`,
  that job only) and sends `repository_dispatch` `release` with `{tag, sha}`. (A tag pushed with
  the built-in token starts no workflow; a dispatch sent with it does.) The tag already on that
  commit: nothing happens. The tag on an older commit (the version is already released): nothing happens either, until the next bump. It reads no secret and
  publishes nothing; one run at a time, never cancelled.
- `release.yml` takes the tag from `github.ref_name` (a tag pushed by hand) or from the dispatch
  payload, which reaches the shell only through `env` and is checked against `^[0-9]+\.[0-9]+\.[0-9]+$`
  (the sha against 40 hex digits). A dispatch runs on the default branch, so every job checks out
  `refs/tags/X.Y.Z` explicitly (`ci.yml` takes a `ref` input for this). The run's title is
  `release X.Y.Z <sha>`; `docs.yml` reads the tag and commit from it.
- `verify` refuses a tag that is not `X.Y.Z`, not on `main`, not equal to the `Cargo.toml`
  version, or not resolving to the announced commit, before anything is built. It also runs the SC-001 assertion
  (`.github/scripts/no-publish-token.sh`), which rejects any use of the `secrets` context
  (including `toJSON(secrets)` and `secrets: inherit`), credential-like `vars.*`, and any
  `CARGO_REGISTRY_TOKEN` or `CARGO_REGISTRIES_*_TOKEN` not set from the OIDC exchange;
  `make sc001-test` proves each pattern is caught.
- The published crate ships only what the build needs (`include` in `Cargo.toml`: `src/`,
  `build.rs`, `assets/`, `extension/` without its dev-only files, `scripts/ext-pack.py`, the
  licence and README), not `specs/`, `docs/` or `tests/`. Check it with `cargo package --list`.
- `publish` is the only job with `id-token: write`, and the only one in the `release` environment.
  It exchanges the job's OIDC identity for a short-lived crates.io token
  (`rust-lang/crates-io-auth-action`, revoked when the job ends) and runs `make publish`, which
  is `cargo publish --locked --no-verify`. It must not verify: verification builds the packaged
  crate, so every dependency's `build.rs` and proc macro would run with the token in its
  environment and the network on. The `cargo publish --dry-run` job (`make publish-dry-run`)
  does verify the same commit, and `publish` needs it. When the
  version is already on crates.io (a re-run after the GitHub release step failed), it skips the
  publish.
- `GitHub release` runs only after `publish` succeeded, so a failed publish never creates a
  release. It is the only job with `contents: write`.
- A manual run is always a dry run: everything except `publish` and `GitHub release`. The
  tarballs and `SHA256SUMS` are kept as the `release-assets` artifact for 14 days.
- `concurrency: release` with `cancel-in-progress: false`: one release at a time, never killed
  halfway.

Artifacts (FR-004): `rstory-X.Y.Z-x86_64-unknown-linux-musl.tar.gz` and
`rstory-X.Y.Z-aarch64-unknown-linux-musl.tar.gz`, each with `rstory`, `LICENSE` and `README.md`
in a `rstory-X.Y.Z-<arch>-unknown-linux-musl/` directory, plus `SHA256SUMS`. The tarballs are
reproducible: sorted entries, owner 0, mtime of the release commit, no gzip name or time.

Hardening, in every workflow: `permissions: {}` at the top and least privilege per job; every
action pinned by full commit SHA with its version in a comment (Dependabot moves the pins, with a
14-day cooldown); `persist-credentials: false` on every checkout; no `pull_request_target`; event
data reaches `run:` only through `env:`. No workflow reads the `secrets` context at all; the
issue reporter uses the job's own token. Every job has a timeout.

CI runner notes:

- Docker, not Podman, on the runners (`CONTAINER_ENGINE=docker`, set by
  `.github/actions/tools`): it is rootful there, so the recipes' `--cpus`/`--memory` limits work
  without cgroup delegation. The runners have 4 vCPUs and 16 GiB, so CI sets `MEMORY=12g`.
- `TARGET_VOLUME` and `NPM_VOLUME` point at directories under `$RUNNER_TEMP` (bind mounts), so
  later steps read the JUnit reports and the release binaries.
- The tools image is rebuilt per job. The PR target is under 15 minutes (SC-003); record the
  first measured durations here.

| Run | Measured duration |
|---|---|
| PR into `dev` | not measured yet |
| Push to `dev` | not measured yet |
| Release dry run | not measured yet |

### aarch64 runner unavailable

ADR-0009's fallback is `cross`. When `ubuntu-24.04-arm` jobs stay queued, a release can wait, or
the owner builds the aarch64 tarball with `cross build --release --locked --target
aarch64-unknown-linux-musl` on an x86_64 machine, checks F2 and F5 by hand (`file` says statically
linked, at most 15 MiB), packages it as above and attaches it to the release with its line in
`SHA256SUMS`. The CI path for this is not automated.

## Gates per stage

From `docs/product/test-strategy.md` §6. `plan` in `ci.yml` picks the set.

| Stage | Runs | Gate set in CI |
|---|---|---|
| PR into `dev` | `ci.yml`, both architectures | fmt, clippy, unit, property (`PROPTEST_CASES=1000`), snapshots, contract, integration (Xvfb, sway, bridge), fitness F1, F2, F3, F4, F5, F9, F10, F11, RSS and burst, advisories, secret scan, extension lint, type-check and tests. Nextest profile `ci`: no retries. |
| Push to `dev` | `ci.yml` | the PR set + perf smoke (`make bench`, `make test-perf`). Nextest profile `ci-dev`: one rerun, reported. |
| PR `dev` → `main`, push to `main` | `ci.yml` | the PR set + perf smoke, property with `PROPTEST_CASES=10000`. Nextest profile `ci`. |
| Version bump on `main` | `tag.yml`, then `release.yml` | `tag.yml` tags and dispatches; `release.yml` runs the gates below on the tag. |
| Tag `X.Y.Z` | `release.yml` | the main set (through `ci.yml`) + tag, version and SC-001 checks, both tarballs with F2/F5, `cargo publish --dry-run`. The manual matrix is signed off in the release PR (below). |
| Nightly | `nightly.yml` | property with `PROPTEST_CASES=100000` (`store_crash`, `history_props`, `filter_props`), advisories, quarantined tests on both architectures. A failure opens or updates the issue `CI failing: nightly on <branch>`. |

Thresholds: zero failures; flake rate under 1% over 30 days (the JUnit reports are kept as
artifacts for 30 days); PR CI under 15 minutes.

Gaps against §6, not wired because the suites do not exist yet:

- E2E-PEER runs the real daemon against a second uid only where the job can switch uid
  (`setpriv --reuid`); the unprivileged tools container cannot, so it checks the daemon's accept
  loop in-process there (`tests/e2e/E2E-PEER.rs`).
- Perf smoke: F6 (picker first frame) fails at 2x its 100 ms budget, which is the 200 ms CI
  budget (`make bench`). F7 (`tests/perf/capture_latency.rs`) fails at p95 100 ms locally and 200 ms
  when `CI` is set. Burst, SC-002 RSS and F10 run the real daemon binary on Xvfb and read `/proc/<pid>`
  (`make test-perf`; the binary is a debug build, so RSS figures are an upper bound). Wake-ups are
  voluntary context switches over the daemon's threads, not `schedstat`.
- Nightly soak (`make test-soak`, 30 minutes, 1 copy a minute; `SOAK_SECS=n` shortens it).
- FR-002's headless mutter `--no-x11` case (no capture path) is covered by E2E-K4 in the `e2e`
  job (`Mutter::start_no_x11`).
- `store_crash`, `filter_props` (default 10 000) and tests on proptest's default config follow
  `PROPTEST_CASES`.

## Flaky-test policy

From test-strategy §8, wired through `cargo-nextest` (`.config/nextest.toml`):

- `make test` and `make test-integration` run nextest with `NEXTEST_PROFILE` (default `default`).
- PRs and the release gate use profile `ci`: `retries = 0`. A flaky test fails the PR.
- Pushes to `dev` and the nightly run use `ci-dev`: one rerun. A test that passes on the rerun is
  marked FLAKY in the log, and `.github/scripts/flaky-report.py` turns it into a warning
  annotation and a job-summary table: reported, never silently green.
- A test flaky twice in 14 days is quarantined: mark it
  `#[ignore = "quarantine: #<issue>"]`, open the issue with an owner and a deadline 14 days out,
  and label it `quarantine`. `make test-quarantine` (nightly, both architectures) runs only the
  ignored tests. The fix PR removes the `#[ignore]` and closes the issue.
- Session tests wait on readiness probes (socket, display), never on sleeps.

## Deliberate-failure checklist (CI validation)

Run once after the workflows land, and again when the job graph changes. Each row is a throwaway
branch off `dev` with one commit and a draft PR into `dev`; close the PR and delete the branch
after. Expect `ci-ok` red, and the named job red, each time. Use only synthetic text.

| # | Break | Expected red job and step |
|---|---|---|
| 1 | In a file under `src/picker/`, add a reference to `crate::daemon` (the picker must not depend on the daemon: a module boundary, F1) | `test`: `boundaries::module_boundaries_hold` in `tests/fitness/`, both architectures (US1 independent test) |
| 2 | Misformat one line in `src/` | `checks`: `make fmt-check` |
| 3 | Add `let _x = 1;` with an unused variable in `src/` | `lint`: clippy `-D warnings`, both architectures |
| 4 | Make one unit test `assert!(false)` | `test`, both architectures |
| 5 | Add `openssl = "0.10"` to `[dependencies]` (and `make fetch` locally) | `checks`: `make deny` (F3 bans) |
| 6 | Add a text file holding a made-up token in a known provider format (for example `ghp_` followed by 36 random letters and digits; gitleaks ignores the documented example keys) | `checks`: `make secret-scan` |
| 7 | Add `${{ secrets.ANY }}` to an `env:` in a workflow | `checks`: the SC-001 step |
| 8 | Break `extension/extension.js` syntax | `checks`: `make ext-lint` |
| 9 | Make a session test wait for a socket that never appears | `integration`, both architectures |
| 10 | Set the size check in `build-static` to 1 MiB | `static`, both architectures (F5) |
| 11 | Add a test that fails on its first attempt only (it creates a marker file under `/tmp` and passes when the marker exists) | the PR (profile `ci`): `test` red. Locally `make test NEXTEST_PROFILE=ci-dev TEST_ARGS='--test <that test>'`: green, the test marked FLAKY |
| 12 | Run `gh workflow run release.yml -f dry_run=false` | `verify`: "manual runs are dry runs only" |
| 13 | Run `gh workflow run release.yml -f dry_run=true` on `dev` | green; `publish` and `GitHub release` skipped; artifact `release-assets` has both tarballs and `SHA256SUMS` |

Also lint the workflows locally before pushing a change to them (Podman, images pinned by
digest):

```bash
podman run --rm --network=none -v "$PWD":/repo:ro,Z -w /repo \
  docker.io/rhysd/actionlint@sha256:9d36088643581e728c969f35141f88139fec77280b2be23c1f66f8e40e1025e7  # 1.7.12
podman run --rm --network=none -v "$PWD":/repo:ro,Z -w /repo \
  ghcr.io/zizmorcore/zizmor@sha256:f8451d8719580a4489501768c4ea64b161dee94b049e206ba7cbe940d7f7df3d --offline .  # 1.30.1
.github/scripts/no-publish-token.sh .github
```

zizmor reports only `self-repository` hints (low): the local action and the reusable workflow
are referenced as `./…`, which actionlint and every runner understand.

## Owner setup before the first tag

The workflows cannot configure these; the owner does, once. Nothing here is a stored secret.

1. **Land the workflows.** PR `ci/release` into `dev`, then promote `dev` to `main` with a PR.
2. **Actions defaults** (Settings > Actions > General): workflow permissions "Read repository
   contents and packages permissions"; leave "Allow GitHub Actions to create and approve pull
   requests" off; require approval for workflows from first-time outside contributors.
3. **Environment `release`** (Settings > Environments > New environment):
   - required reviewer: the owner;
   - deployment branches and tags: "Selected branches and tags", add the tag rule `*.*.*` and the
     branch rule `main`. A run started by `tag.yml`'s dispatch runs on the default branch, so
     without the `main` rule the `publish` job is rejected; the tag rule serves tags pushed by
     hand. Approval by the reviewer still gates every publish;
   - no environment secrets and no variables.
4. **Rulesets for `dev` and `main`**: require a pull request; require the status check `ci-ok`
   (it appears after the first CI run); block force pushes and deletions.
   **Also required: a tag ruleset** that lets only the owner create tags matching `*.*.*`.
   Without it any collaborator with write access can push a release tag, and the environment's
   reviewer would be the only gate before publish. **Add "GitHub Actions" to that ruleset's bypass
   list** (Bypass list > Add bypass > GitHub Actions), or `tag.yml` cannot create the tag and its
   run fails with a rejected push.
5. **Labels**: create `quarantine` (flaky-test policy). The failure reporter needs no label; it
   finds its issue by title.
6. **Security settings**: secret scanning with push protection, Dependabot alerts.
7. **Check the arm64 runners**: the first CI run must start `ubuntu-24.04-arm` jobs (free for
   public repositories).
8. **Exercise the pipeline**: the deliberate-failure checklist above, then a release dry run
   (`gh workflow run release.yml --ref dev -f dry_run=true`). Record the durations above.
9. **Confirm the licence exception**: `deny.toml` allows the Unlicense for `ksni` only (the FSF
   lists it as GPL-compatible). Keep it or replace the tray crate.
10. **Publish 0.1.0 by hand and set up Trusted Publishing**: next section.

## First release by hand (0.1.0)

crates.io has no pending publisher for a crate that does not exist yet (brief §11), so the first
version is published with the owner's personal token (K9). The token lives in the owner's
password store; it is read at the moment of use and never written to the repository, a file or
the shell history.

1. On `main` at the release commit, with the manual matrix signed off: `make fetch`, then
   `make publish-dry-run`. It must end with "aborting upload due to dry run".
2. Create a crates.io API token scoped to `publish-new` (and `publish-update`) for the crate name
   `rstory`, with a short expiry, and store it in the password store.
3. Publish, reading the token straight from the password store into the one command's
   environment (replace `<entry>` with the entry's name; the value never appears on a command
   line, and `make` passes only the variable's name to the container):

   ```bash
   CARGO_REGISTRY_TOKEN="$(pass show <entry>)" make publish
   ```

   If it fails on the token's scope, nothing is published: fix the scope and repeat.
4. On crates.io, crate `rstory` > Settings > Trusted Publishing > Add: GitHub, owner
   `lucasvmigotto`, repository `rstory`, workflow file `release.yml`, environment `release`.
5. `0.1.0` exists only on crates.io: it has no git tag and no GitHub release (owner decision
   2026-10-25). `0.1.1` is the first version `tag.yml` tags and the release workflow publishes
   with tarballs and the extension zip, after the `release` environment approval. Publish the
   first version by hand (steps 1 to 3) before promoting `tag.yml` to `main`, or the first
   automatic run waits at `publish` for a crates.io upload the token-less workflow cannot make.

## Publish the extension on extensions.gnome.org (after 0.1.0)

A zip built before 0.1.1 has the UUID `rstory@lucasvmigotto.github.io` and a `url` that was not the
repository. Withdraw that upload on the site and upload the 0.1.1 zip: the UUID is
`rstory@lucasvmigotto.me`, and `url` must be the GitHub repository (the reviewers reject a URL that
returns 404 or is not a GitHub or GitLab repository).

The extension does nothing without the daemon, so it goes up only once 0.1.0 is released (owner
decision 2026-10-24). The first upload is manual, and later versions use the same form.

1. Take the zip from the GitHub release: every version tag's release workflow builds
   `rstory@lucasvmigotto.me.shell-extension.zip`, checks it with Shexli, and attaches it next to the tarballs
   (listed in `SHA256SUMS`). Locally, `make ext-shexli` does the same: `make ext-pack`, then Shexli, failing unless clean.
2. To run the analyzer by hand instead, in a throwaway container. It must report `clean`:
   `podman run --rm --name scratch-rstory-shexli -v "$PWD/dist":/ego:ro,Z docker.io/library/python:3.13-slim sh -c 'pip install -q shexli && shexli /ego/rstory@lucasvmigotto.me.shell-extension.zip'`
3. Upload the zip at <https://extensions.gnome.org/upload/> and accept both terms (the GNOME Shell
   license clause, and that site staff may remove, modify or reassign the extension).
4. On the extension's page, add a description that says it needs the rstory daemon, with the
   install steps (`cargo install rstory` or the release binary, then `rstory install-autostart --now`).
   Add a screenshot of the menu.
5. Review results arrive by e-mail. Record the outcome and the site's version number in the
   "Manual check log".

For each later version, raise `version-name` in `metadata.json` with the crate version (a test
enforces it), then repeat steps 1–3. Add a Shell version to `shell-version` only after its manual
matrix passes and Shexli accepts it.

## Every later release (0.1.1 on)

1. On a branch off `dev`, bump `version` in `Cargo.toml`, update `Cargo.lock` with `cargo update
   --workspace` inside `make shell` (the diff must change only rstory's own version), and open the
   release PR. Copy the "Tier-1 manual matrix: sign-off" table into the PR body and
   fill it in.
2. Promote `dev` to `main` with a PR; `ci-ok` must be green on it.
3. Nothing to tag: the push to `main` runs `tag.yml`, which tags the merge commit `X.Y.Z` and
   dispatches `release.yml`. If that run failed (tag rejected, dispatch failed), fix the cause and
   run it again from `main`: `gh workflow run tag.yml --ref main` (add `-f redispatch=true` when
   the tag exists on the commit and only the dispatch is missing). Tagging by hand
   (`git tag -a X.Y.Z -m X.Y.Z && git push origin X.Y.Z`) still works and starts `release.yml` too.
4. Approve the `release` environment when `publish` waits for it. Follow the run with
   `gh run watch`.
5. Check: crates.io shows `X.Y.Z` published by the trusted publisher (ADR-0009 Confirmation); the
   GitHub release has both tarballs and `SHA256SUMS`; `sha256sum -c SHA256SUMS` passes on a
   download.

**Token retirement** (SC-001): after 0.1.1 is published by the workflow, revoke the personal
token on crates.io and delete its password-store entry. Optionally enable the crate's
"Trusted Publishing only" setting, if crates.io offers it. From then on no publishing token
exists anywhere.

### When a release goes wrong

| Failure | State | What to do |
|---|---|---|
| `verify` fails (tag format, not on `main`, version mismatch) | nothing built or published | delete the tag (`git push origin :refs/tags/X.Y.Z`), fix, then bump to a new patch version, or re-run `tag.yml` from `main` (or tag by hand) |
| `ci`, `package`, `SHA256SUMS` or `cargo publish --dry-run` fails | nothing published | fix on `dev`, promote with a new patch version (or, if nothing was published, delete the tag and re-run `tag.yml` or re-push the tag by hand) |
| `publish` fails (OIDC exchange, Trusted Publisher config) | nothing published, no GitHub release | check the Trusted Publisher fields (workflow `release.yml`, environment `release`), re-run the failed jobs |
| `GitHub release` fails after a successful publish | crate published, no GitHub release | re-run the failed jobs: `publish` sees the version on crates.io and skips; the release is created or its assets re-uploaded |
| A published version is broken | crate on crates.io | `cargo yank --version X.Y.Z` (with a short-lived token or the web UI), release a fixed patch version; never delete a tag that was published |

## Tier-1 manual matrix: sign-off

FR-006 and test-strategy §7: a release is signed off when every row below is done on the release
candidate (the binary from the release dry run, installed to `~/.local/bin`). Copy the table into
the release PR. A failing row blocks the release.

| Environment | Check (section below) | Version and distribution | Result | Signed off by |
|---|---|---|---|---|
| GNOME 48 Wayland | GNOME 48–51 manual check (bridge capture with GUI apps, brief Q7) | | | |
| GNOME 49 Wayland | GNOME 48–51 manual check | | | |
| GNOME 50 Wayland | GNOME 48–51 manual check | | | |
| GNOME 51 Wayland | GNOME 48–51 manual check | | | |
| GNOME 48, 49, 50 (light and dark panel) | GNOME Shell extension manual matrix | | | |
| KDE Plasma 6 Wayland | KDE Plasma Wayland manual check | | | |
| KDE Plasma 6 Wayland | Tray manual check | | | |
| sway with waybar | Tray manual check | | | |
| GNOME, KDE Plasma, sway | Autostart re-login check | | | |
| CI (release dry run) | `ci-ok` green on both architectures; both tarballs pass F2 and F5 | | | |

## Manual check log

Checks run outside a release candidate, for example on a `dev` build, so they inform the release
but don't replace the sign-off above.

| Date | Environment | Check | Build | Result | By |
|---|---|---|---|---|---|
| 2026-10-24 | GNOME Shell 48.7, Wayland | GNOME 48–51 manual check (10 rows) | `dev` static build after PR #74 | all pass | owner |
| 2026-10-24 | GNOME Shell 48.7, Wayland | GNOME Shell extension manual matrix (18 rows) | `dev` static build after PR #74 | all pass | owner |

## GNOME 48–51 manual check

Why manual: headless mutter (used in CI for bridge mode) has no keyboard seat, so no Wayland
client can own a selection there. Cross-protocol copies, Wayland app to XWayland and back, can
only be checked on a real GNOME session. This covers SC-004, brief Q7 and the confirmation of
ADR-0005. Use only throwaway text you typed for the check, never real clipboard contents.

Repeat on each GNOME Shell version: 48, 49, 50, 51 (Wayland session, no extension installed).

Setup: start `rstory daemon` from a terminal with `RUST_LOG=rstory=debug`. The log must say
`backend=x11 mode=xwayland_bridge` (`rstory doctor` should report the same). Keep another
terminal for `rstory ls`.

| # | Step | Expected |
|---|---|---|
| 1 | Copy a line from a GUI app (Text Editor, a Files rename box, Firefox) | New entry at the top of `rstory ls`, text correct |
| 2 | Copy with `wl-copy "wayland-native copy"` | New entry, text correct (a Wayland client seen through XWayland) |
| 3 | Copy a selection in Terminal with Ctrl+Shift+C | New entry |
| 4 | Copy the same text twice | One entry, moved to the top (BR1) |
| 5 | Copy from a secrets-manager entry (it offers the sensitive hint) | No entry recorded; `skipped_sensitive` rises |
| 6 | Restore an entry (`rstory copy <ref>` or the picker), then paste into a Wayland-native app (Text Editor, Console) | Pasted text equals the entry |
| 7 | Paste the same restored value into an X11-only app (`xterm`, or `xclip -o -selection clipboard`) | Pasted text equals the entry |
| 8 | Restore a value of about 1 MiB and paste into a Wayland-native app | Complete paste, no hang |
| 9 | Kill XWayland (`pkill Xwayland`), then copy again | The daemon reconnects (100 ms to 5 s backoff) and the copy is captured |
| 10 | Copy from a GUI app after the screen locked and unlocked | New entry |

Record per Shell version: version, distribution, result per row, and anything odd (for example a
GUI app that does not update the selection while unfocused). A failing row blocks the release.

Quick CI cross-check before the manual run:
`make test-integration TEST_ARGS="--test bridge_capture"`.

## KDE Plasma Wayland manual check

Why manual: headless sway (CI) covers the data-control protocol, but only a real Plasma session
has KWin's own data-control and the clients that set `x-kde-passwordManagerHint` (secrets
managers). This covers SC-003 of spec 003. Use only throwaway text you typed for the check, never
real clipboard contents.

Repeat on each supported Plasma 6 release (Wayland session). Setup: start `rstory daemon` from a
terminal with `RUST_LOG=rstory=debug`. The log must say `backend=wayland mode=native`. Keep
another terminal for `rstory ls`.

| # | Step | Expected |
|---|---|---|
| 1 | Copy a line from a GUI app (Kate, a Dolphin rename box, Firefox) | New entry at the top of `rstory ls`, text correct |
| 2 | Copy with `wl-copy "wayland-native copy"` | New entry, text correct |
| 3 | Copy a selection in Konsole with Ctrl+Shift+C | New entry |
| 4 | Copy the same text twice | One entry, moved to the top (BR1) |
| 5 | Copy an entry from a secrets manager (it offers the sensitive hint) | No entry recorded; `skipped_sensitive` rises |
| 6 | Restore an entry, then paste into Kate and into an XWayland app (`xterm`) | Pasted text equals the entry |
| 7 | Restore a value of about 1 MiB and paste into Kate | Complete paste, no hang |
| 8 | Copy something else after a restore | The new copy is captured; rstory stops serving (no stale paste) |
| 9 | Copy from a GUI app after the screen locked and unlocked | New entry |

Record per Plasma version: version, distribution, result per row, and anything odd. A failing
row blocks the release.

Quick CI cross-check before the manual run:
`make test-integration TEST_ARGS="--test wayland_capture"` and `--test wayland_serve`.

## Autostart re-login check (GNOME, KDE Plasma, sway)

Why manual: only a real login runs the XDG autostart entry and the systemd user unit. This covers
SC-001 of spec 012. Install the release binary to `~/.local/bin` first, so the entry does not
point at a temporary path.

Repeat on GNOME (Wayland), KDE Plasma (Wayland) and sway, once for each autostart kind.

| # | Step | Expected |
|---|---|---|
| 1 | `rstory install-autostart`, then log out and back in | `rstory status` answers; the daemon is running |
| 2 | Run `rstory install-autostart` again | Same file, exit 0, nothing else changes |
| 3 | `rstory install-autostart --systemd`, then log out and back in | `systemctl --user status rstory` is active; no `~/.config/autostart/rstory.desktop` is left |
| 4 | `pkill rstory` while the systemd unit is installed | The daemon is back within a few seconds (`Restart=on-failure`) |
| 5 | `rstory install-autostart --uninstall`, then log out and back in | No daemon is running; `rstory status` exits 3 |

Record per desktop: version, distribution, result per row. A failing row blocks the release.

## Tray manual check (KDE Plasma and sway with waybar)

Why manual: the D-Bus tests use a stub `StatusNotifierWatcher`; only a real Plasma panel or
waybar's tray module shows the icon, draws the menu and renders the pixmaps. This covers SC-001
of spec 009. Use only throwaway text you typed for the check, never real clipboard contents.

Run once on KDE Plasma 6 (Wayland) and once on sway with waybar (`tray` module enabled). Setup:
start `rstory daemon` from a terminal with `RUST_LOG=rstory=info`. The log must show
`tray.state` with `state="active"`, and `rstory status --json` must say `"tray": "active"`.
Copy three short throwaway lines first.

| # | Step | Expected |
|---|---|---|
| 1 | Look at the panel | The rstory icon (clipboard outline) is visible and sharp at the panel size |
| 2 | Open the menu | The three lines, newest first, then a separator, `Recording`, `Pause recording`, `Pause for 1 minute`, `Open picker` (when a terminal exists), a separator, `Clear history…` |
| 3 | Click the third line, then paste into a text field | The pasted text equals that line, and the menu now lists it first |
| 4 | Click `Pause recording` | The icon changes to the paused icon (two bars, a different shape, not only another color); the menu shows `Paused` and `Resume recording`; copying something new adds no entry |
| 5 | Click `Resume recording` | The normal icon returns; the next copy is recorded |
| 6 | Click `Pause for 1 minute`, reopen the menu after about 20 s | `Paused — resumes in 0:4x`; after a minute the icon is back to normal by itself |
| 7 | Click `Clear history…` | The menu offers `Delete N unpinned entries` and `Cancel`; nothing is deleted yet; `Cancel` leaves the history as it was |
| 8 | Click `Clear history…` then `Delete N unpinned entries` | Only the unpinned entries go (`rstory ls`); pinned ones stay |
| 9 | Stop the panel's tray host (restart `plasmashell`, or `pkill waybar`) and start it again | The daemon keeps running; the icon reappears within a second or two; `status.tray` goes `no_host`, then `active` |
| 10 | Start the daemon with `tray = false` in `config.toml` | No icon; `status.tray` is `disabled`. `rstory tray` in another terminal shows the icon instead |

Light panels: the embedded icons are the variant for dark panels (light glyph). On a light panel
check that the icon is still legible and record it as a finding (see the spec's Doubts).

Record per desktop: version, result per row, and anything odd. A failing row blocks the release.

CI cross-check before the manual run: `make test TEST_ARGS=` runs `tests/tray_dbus.rs` against a
private `dbus-daemon` and a stub watcher inside the tools container.

## GNOME Shell extension manual matrix (48, 49, 50)

Why manual: the panel button, the Shell's extension loader and its theme only exist in a real
GNOME Shell. The gjs fixture tests (`make ext-test`) cover the client, the menu model and the
cleanup in `disable()` against fakes. This covers SC-001 of spec 010. Use only throwaway text you
typed for the check, never real clipboard contents.

Repeat on each GNOME Shell version the extension declares (48, 49 and 50), on a Wayland session. Run it on the light
and the dark panel (Settings > Appearance, or `gsettings set org.gnome.desktop.interface
color-scheme prefer-dark`). Install the release binary to `~/.local/bin` first.

Setup: `rstory install-autostart --now` so a daemon is running. Keep a second terminal for
`rstory ls` and `rstory status`. For the log: `journalctl --user -f /usr/bin/gnome-shell | grep -i
rstory`. Nothing in that log may contain entry text; only error codes.

| # | Step | Expected |
|---|---|---|
| 1 | `rstory install-gnome-extension` | Exit 0; prints `GNOME extension installed. Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.me`; no stderr warning |
| 2 | Log out and back in, then `gnome-extensions enable rstory@lucasvmigotto.me` (or `rstory install-gnome-extension --enable` on the next login) | `gnome-extensions info rstory@lucasvmigotto.me` shows `Enabled: Yes` and a State other than ERROR |
| 3 | Look at the top bar | An rstory clipboard icon (recording shape: three lines). It follows the panel color on light and dark |
| 4 | Copy three different lines of text, then open the menu | The newest entries appear first, one per line, at most 10; the status line says `Recording`; Pause recording, Pause for 1 minute and Open picker are present; `Clear history…` is last |
| 5 | Copy a two-line text | Its menu line shows `⏎` where the newline is |
| 6 | Click an entry, then paste into a text editor | The pasted text equals the entry |
| 7 | `rstory pin 0`, then open the menu | The pinned entry shows the `📌 ` prefix |
| 8 | Click `Pause recording` | The icon changes to the paused shape (two bars); the menu shows `Paused` and `Resume recording`; `Pause for 1 minute` is hidden; `rstory status` says paused |
| 9 | Click `Resume recording` | The icon returns to the recording shape |
| 10 | Click `Pause for 1 minute`, keep the menu open | The status line counts down (`Paused — resumes in 0:59`, ...) and the icon returns to recording by itself at zero |
| 11 | `rstory pause` in a terminal while the menu is closed, then open it | The icon is already the paused shape (the change came from the daemon, not the click) |
| 12 | Click `Open picker` | A terminal with the picker opens; the extension spawned nothing itself (the process parent is the daemon) |
| 13 | Click `Clear history…` then `Cancel` | The menu closes; `rstory ls` is unchanged |
| 14 | Click `Clear history…` then `Delete N unpinned entries` | N matches the unpinned count in `rstory status`; pinned entries remain |
| 15 | `pkill rstory` with the menu open | The icon becomes the disconnected shape (clipboard with a slash); the menu shows only `rstory daemon not running` |
| 16 | Start the daemon again (`rstory daemon &`) | Within 30 s (1 s, 2 s, 4 s, ... backoff) the icon is back to recording and the entries are listed |
| 17 | `rstory install-gnome-extension` again | Same version: the plain `installed` message. After a version bump: `GNOME extension updated to X. Log out and back in to load it.` |
| 18 | `rstory install-gnome-extension --uninstall`, then log out and back in | The icon is gone; `~/.local/share/gnome-shell/extensions/rstory@lucasvmigotto.me` does not exist |

Record per Shell version: version, distribution, panel theme, result per row, and anything odd
(for example the icon not following the panel color, or a Shell error in the journal on enable).
A failing row blocks the release. A Shell outside 48 to 50 must refuse to load the extension
because of `metadata.json`; `rstory doctor` explains it.

Quick CI cross-check before the manual run: `make ext-test ext-typecheck ext-lint` and
`make test` (the install tests in `tests/install_gnome.rs`).

### Extension leak check (once per Shell version)

The gjs tests check `disable()` cleanup against fakes (20 enable and disable cycles). This repeats
it in the real Shell, where leaked signal handlers, timers or actors would show.

1. Note the Shell's memory: `ps -o rss= -C gnome-shell`.
2. Run 50 cycles: `for i in $(seq 50); do gnome-extensions disable rstory@lucasvmigotto.me; gnome-extensions enable rstory@lucasvmigotto.me; sleep 0.2; done`.
3. Open and close the menu a few times, then check the journal: `journalctl --user -b /usr/bin/gnome-shell | grep -i rstory`.

Expected: exactly one rstory icon in the top bar; no `rstory` warnings or JS errors in the journal
(no "already disposed" or "signal handler" messages); Shell RSS within about 5 MiB of step 1.
A clear upward trend over repeated runs counts as a leak and blocks the release.
