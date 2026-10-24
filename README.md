# rstory

A clipboard history for any Linux session. It ships as one static Rust binary that works the same on Wayland and X11, under any desktop or distribution, without installing anything else or leaving secrets on disk.

> [!NOTE]
> rstory is in development. Nothing is released yet. The product, its scope and its plan are in [`docs/product/brief.md`](docs/product/brief.md), and the features and their status are in [`specs/README.md`](specs/README.md).

## Privacy

rstory keeps clipboard text in a private store on your machine and never uses the network. What it protects, and what it does not:

- **Sensitive hints.** When the app that owns a copy offers the type `x-kde-passwordManagerHint` (KeePassXC, KDE and other password managers set it), rstory skips the copy without reading it. The decision uses the offered type names only, so the content never enters rstory's memory. Matching is exact and case-sensitive, and the list of hints is one constant (`SENSITIVE_HINTS`); extending it is a spec amendment. Skips are counted in `rstory status` as `skipped_sensitive`.
- **Size cap.** A copy larger than `max_entry_bytes` (1 MiB by default) is dropped whole, never truncated, and counted as `skipped_oversize`. Empty copies and copies with no text are ignored.
- **Pause.** While recording is paused nothing is read, and the changes are counted as `skipped_paused`. The pause is checked before the hint.
- **Logs.** Decisions log their name and size, never content.

**Threat model.** rstory defends against a copy that an app marks as sensitive ending up in the history. It cannot defend against apps that set no hint: another program running as you can read the clipboard while the copy is on it, and the `pass` password manager sets no hint. A secret copied from an app that gives no hint is recorded like any other text.

**The `passc` wrapper.** For tools that set no hint, pause recording around the copy. The `pass` copy mode clears the clipboard after 45 seconds, so a 50 second pause covers it:

```sh
passc() { rstory pause --for 50s && pass -c "$@"; }
```

## Development

> [!IMPORTANT]
> You need Podman (preferred) or Docker, and `make`. Every task runs in the project's `tools` container, so no Rust toolchain on the host is required.

### Tasks

```bash
make help               # list recipes
make image              # build the tools image (Containerfile, target tools)
make fetch              # resolve crates into the cache volume (the only networked step)
make fmt | fmt-check    # rustfmt
make lint               # clippy -D warnings, offline
make test               # unit, property, snapshot, contract and fitness tests (cargo-nextest), offline
make test-integration   # Xvfb, headless sway and headless mutter session tests, offline
make build-static       # static musl release build + static-binary and size checks
make ext-lint ext-test  # GNOME extension: eslint + gjs fixture tests
make deny audit         # cargo-deny: bans/licences/sources (F3, F4) offline; RustSec advisories (networked)
make secret-scan        # gitleaks over the working tree
make doctor             # tool versions per layer; fails on drift from rust-toolchain.toml
make clean [dry=1]      # remove this worktree's containers and volumes and our dangling images
make shell              # interactive shell in the tools image
```

CI runs the same recipes, so a green run locally means the same as a green run in CI. Parallel worktrees pass their own build cache: `make TARGET_VOLUME=rstory-tools-target-<name> test` (or set `AI_GENT_WORKTREE=<name>`). Everything the recipes create carries the labels `me.lucasvmigotto.ai-gent.project=rstory` and `me.lucasvmigotto.ai-gent.worktree=rstory/<name>`, so `make clean` removes exactly that worktree's leftovers and never another project's; `make image` drops our own dangling images after each rebuild.

### Moving parts

| Layer | What | Where |
|---|---|---|
| Version source | the Rust toolchain, components and musl targets | `rust-toolchain.toml` |
| `tools` image | the reference environment for people, agents and CI: Rust + Xvfb, sway, mutter, XWayland, D-Bus, gjs, eslint, rsvg-convert | `Containerfile` (`base → deps → tools → build → runtime`), `make image` |
| Recipes | one entry point per task, run with limits (`CPUS=4`, `MEMORY=3g`, overridable) and `--network=none` after `make fetch` | `Makefile` |
| Human devcontainer | your editor's environment: `devenv:debian-trixie` + the Rust feature at the same version, rust-analyzer running clippy | `.devcontainer/devcontainer.json` |

With Podman, point the devcontainer tooling at it: the CLI flag `devcontainer up --docker-path podman`, or the VS Code setting `dev.containers.dockerPath: podman`.

### Decisions

- **Why a separate `tools` image?** It's thin, pinned by digest and identical in CI, so test results don't depend on whose editor ran them. The devcontainer is for comfort. When the two disagree, `tools` is right.
- **Why not `dhi.io/rust:*-dev` for `tools`?** That hardened image ships no `rustup`, so the `*-unknown-linux-musl` targets the static-build check needs can't be added. The official `rust:1.98.1-slim-trixie` (pinned by digest) is used instead.
- **Why one devcontainer, not two?** The GNOME extension is a protocol client, but it's embedded in and shipped with the same binary. It has no own runtime, debugger or env file. One scope, so one devcontainer and one `tools` image.
- **Why no `compose.tools.yml`?** There's a single tools service and no external infrastructure: the sessions under test start inside the container. Plain `podman run` / `docker run` avoids needing a compose provider on the host.
- **Why headless mutter in `tools`?** It's GNOME's own compositor (no data-control, XWayland on demand), so CI checks backend selection and capture on Mutter's XWayland, the path GNOME users take. Headless mutter has no keyboard seat, so a Wayland client can't own a selection there. Cross-protocol copies (a Wayland app read through XWayland) are therefore checked by hand on GNOME 48–51 before each release.
- **No shared network or `.env`.** rstory has no services to reach and no secrets at development time.

### Branches and commits

Work happens on `feat/…`, `fix/…`, `docs/…` or `ci/…` branches off `dev` and lands through pull requests into `dev`. Promotions to `main` are PRs too. Commits follow Conventional Commits.

## Links

- Product brief: [`docs/product/brief.md`](docs/product/brief.md)
- Architecture and ADRs: [`docs/product/architecture.md`](docs/product/architecture.md), [`docs/product/adr/`](docs/product/adr/)
- Features (Spec Kit): [`specs/README.md`](specs/README.md), constitution: [`.specify/memory/constitution.md`](.specify/memory/constitution.md)
- Control protocol and CLI contract: [`contracts/`](contracts/)
- Test strategy: [`docs/product/test-strategy.md`](docs/product/test-strategy.md)

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
