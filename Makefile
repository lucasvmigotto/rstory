# rstory task recipes — one entry point for people, agents and CI.
# Every recipe runs in the `tools` image (containers.md §6). Podman first, Docker fallback.

ENGINE ?= $(or $(CONTAINER_ENGINE),$(shell command -v podman >/dev/null 2>&1 && echo podman || echo docker))
IMAGE  := localhost/rstory-tools:dev
RUST_VERSION := $(shell sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)

# Ownership labels (ai-gent containers.md §9a): every image, volume and container this file makes
# carries the project label, and the worktree label `rstory/<slug>`, so `make clean` removes exactly
# ours and never another project's.
PROJECT := rstory
PKEY := me.lucasvmigotto.ai-gent.project
WKEY := me.lucasvmigotto.ai-gent.worktree
OWN := --label $(PKEY)=$(PROJECT)

# Per-worktree build cache, so parallel worktrees don't share one target dir. AI_GENT_WORKTREE
# (set by agents:parallel) picks it; an explicit TARGET_VOLUME=rstory-tools-target-<slug> also works.
TARGET_VOLUME ?= $(if $(AI_GENT_WORKTREE),rstory-tools-target-$(AI_GENT_WORKTREE),rstory-tools-target)
WT := $(or $(AI_GENT_WORKTREE),$(patsubst rstory-tools-target-%,%,$(filter rstory-tools-target-%,$(TARGET_VOLUME))),main)

# Limits: at most about half of an 8-CPU / 8 GiB host (containers.md §7); override per call.
CPUS   ?= 4
MEMORY ?= 3g

ifeq ($(notdir $(ENGINE)),podman)
USERNS := --userns=keep-id:uid=1000,gid=1000
RUNDIR := --tmpfs /run/user/1000:rw,mode=0700,U
else
USERNS := --user 1000:1000
RUNDIR := --tmpfs /run/user/1000:rw,mode=0700,uid=1000,gid=1000
endif

# Extension dev tools volume (EXT_MOUNT below).
NPM_VOLUME ?= rstory-tools-npm

# Named volumes are created labelled before first use (`run -v` would create them unlabelled).
# Host paths (CI passes $$RUNNER_TEMP directories) are left alone.
ensure_vol = case "$(1)" in */*) ;; *) $(ENGINE) volume inspect $(1) >/dev/null 2>&1 || $(ENGINE) volume create $(OWN) --label $(WKEY)=$(PROJECT)/$(2) $(1) >/dev/null ;; esac;
ENSURE := $(call ensure_vol,rstory-tools-cargo,main) $(call ensure_vol,$(TARGET_VOLUME),$(WT)) $(call ensure_vol,$(NPM_VOLUME),main)

RUN := $(ENSURE) $(ENGINE) run --rm --init $(USERNS) $(OWN) --label $(WKEY)=$(PROJECT)/$(WT) \
	--cpus $(CPUS) --memory $(MEMORY) --pids-limit 2048 \
	-v "$(CURDIR)":/src:Z \
	-v rstory-tools-cargo:/home/dev/.cargo \
	-v $(TARGET_VOLUME):/target \
	$(RUNDIR) \
	-e TZ=UTC -e LANG=C.UTF-8 -e RUST_BACKTRACE=1 -e CARGO_TERM_COLOR=always \
	$(if $(PROPTEST_CASES),-e PROPTEST_CASES=$(PROPTEST_CASES))
OFFLINE := --network=none
# Test runner profile (.config/nextest.toml): default locally, ci / ci-dev in CI (flaky-test policy).
NEXTEST_PROFILE ?= default
NEXTEST := cargo nextest run --locked --offline --no-tests=pass --profile $(NEXTEST_PROFILE)
# Test-only feature: FaultFs for the crash and power-loss suites (never in release builds).
FEATURES := --features fault-injection
# Lint and check also compile the session tests (Xvfb / mutter), which `make test` skips.
LINT_FEATURES := --features fault-injection,session-tests,perf-tests
# Extension dev tools (typescript, @girs types) live in a named volume, so nothing lands in the repo.
EXT_MOUNT := -v $(NPM_VOLUME):/src/extension/node_modules
TTY := $(shell [ -t 0 ] && echo -it)

.PHONY: help image fetch fmt fmt-check lint check test test-quarantine bench test-integration test-e2e test-e2e-smoke test-perf test-soak build build-static dist test-launch brand ext-lint ext-test ext-pack ext-shexli ext-deps ext-typecheck deny audit secret-scan publish-dry-run publish sc001-test doctor shell clean clean-cache docs-install docs-dev docs-check docs-build docs-reference docs-fmt

help: ## List recipes
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-17s %s\n", $$1, $$2}'

image: ## Build the tools image (from Containerfile, target tools), then drop our own dangling images
	$(ENGINE) build $(OWN) --target tools -t $(IMAGE) -f Containerfile .
	@$(ENGINE) images -q --filter dangling=true --filter label=$(PKEY)=$(PROJECT) | xargs -r $(ENGINE) rmi >/dev/null 2>&1 || true

fetch: image ## Resolve crates into the cache volume (the only networked step)
	$(RUN) $(IMAGE) cargo fetch --locked

fmt: image ## Format the code
	$(RUN) $(OFFLINE) $(IMAGE) cargo fmt --all

fmt-check: image ## Check formatting (CI)
	$(RUN) $(OFFLINE) $(IMAGE) cargo fmt --all -- --check

lint: image ## Clippy with warnings as errors, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo clippy --locked --offline $(LINT_FEATURES) --all-targets -- -D warnings

check: image ## Type-check, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo check --locked --offline $(LINT_FEATURES) --all-targets

test-launch: image ## Picker-launch tests only (quickstart: cargo test launch::), offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo test --locked --offline $(FEATURES) launch::

test: image ## Unit, property, snapshot, contract and fitness tests (nextest + doctests), offline; TEST_ARGS='--test x' narrows
	$(RUN) $(OFFLINE) $(IMAGE) $(NEXTEST) $(FEATURES) $(TEST_ARGS)
	$(if $(TEST_ARGS),,$(RUN) $(OFFLINE) $(IMAGE) cargo test --locked --offline $(FEATURES) --doc)

test-quarantine: image ## Quarantined tests only (#[ignore = "quarantine: ..."]), run nightly, offline
	$(RUN) $(OFFLINE) $(IMAGE) $(NEXTEST) $(FEATURES) --run-ignored only $(TEST_ARGS)

bench: image ## Store benchmarks (std harness); fails if a p95 is over 2x its budget, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo bench --locked --offline $(FEATURES) --bench store

test-integration: image ## Session tests (Xvfb, headless sway/mutter, bridge mode), offline; TEST_ARGS='--test x11_capture' narrows
	$(RUN) $(OFFLINE) $(IMAGE) $(NEXTEST) --features session-tests --test-threads 2 -E 'not binary(e2e)' $(TEST_ARGS)

# The PR gate runs the smoke journeys only (K2, K3, K6); pushes to dev run them all (test-strategy §6).
E2E_SMOKE := -E 'test(/::k2_/) | test(/::k3_/) | test(/::k6_/)'

test-e2e: image ## End-to-end journeys K1-K7 + PEER (real binary, pty, headless sessions), offline; TEST_ARGS='k2_' narrows
	$(RUN) $(OFFLINE) $(IMAGE) $(NEXTEST) --features session-tests --test e2e --test-threads 2 $(TEST_ARGS)

test-e2e-smoke: image ## The PR-gate subset of the journeys (K2, K3, K6), offline
	$(RUN) $(OFFLINE) $(IMAGE) $(NEXTEST) --features session-tests --test e2e --test-threads 2 $(E2E_SMOKE)

test-perf: image ## Perf smoke (F7 capture latency, burst, RSS and idle wake-ups of the real daemon on Xvfb), one at a time; CI=1 applies the CI budgets
	$(RUN) $(if $(CI),-e CI=$(CI)) $(OFFLINE) $(IMAGE) $(NEXTEST) --features perf-tests --success-output immediate --test-threads 1 $(or $(TEST_ARGS),--test burst --test capture_latency --test idle --test rss)

test-soak: image ## Soak (nightly): the daemon for 30 min with 1 copy a minute, RSS flat, wake-ups only at timers; SOAK_SECS=n shortens it
	$(RUN) $(if $(SOAK_SECS),-e RSTORY_SOAK_SECS=$(SOAK_SECS)) $(OFFLINE) $(IMAGE) $(NEXTEST) --features perf-tests --success-output immediate --test-threads 1 --test soak

build: image ## Debug build, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo build --locked --offline

build-static: image ## Static musl release build for this host's arch + F2/F5 checks
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 't=$$(uname -m)-unknown-linux-musl; \
	  cargo build --locked --offline --release --target $$t && \
	  b=/target/$$t/release/rstory && file $$b && file $$b | grep -Eq "static(-pie)? linked|statically linked" && \
	  s=$$(stat -c %s $$b) && echo "size $$s bytes" && [ $$s -le 15728640 ]'

dist: build-static ## Copy the static binary to dist/rstory (then: install -m 0755 dist/rstory ~/.local/bin/)
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'mkdir -p dist && cp /target/$$(uname -m)-unknown-linux-musl/release/rstory dist/rstory'
	@ls -l dist/rstory

brand: image ## Render the brand PNGs from assets/brand/*.svg (docs Open Graph banner, extensions.gnome.org icon), offline
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'rsvg-convert -w 1200 -h 630 assets/brand/og-banner.svg -o docs/site/public/og.png && rsvg-convert -w 256 -h 256 assets/brand/icon.svg -o assets/brand/icon-256.png && ls -l docs/site/public/og.png assets/brand/icon-256.png'

ext-lint: image ## Lint the GNOME extension (eslint from the npm volume), offline
	$(RUN) $(OFFLINE) $(EXT_MOUNT) -w /src/extension $(IMAGE) npx --no-install eslint .

ext-deps: image ## Install the extension's dev tools into the npm volume (the only networked ext step)
	$(RUN) $(EXT_MOUNT) -w /src/extension $(IMAGE) npm ci --no-audit --no-fund

ext-typecheck: image ## Type-check the extension (tsc --checkJs, JSDoc), offline
	$(RUN) $(OFFLINE) $(EXT_MOUNT) -w /src/extension $(IMAGE) npx --no-install tsc -p jsconfig.json

ext-test: image ## Run the extension's gjs fixture tests (fake socket server), offline
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'for t in extension/tests/*.test.js; do echo "== $$t"; gjs -m "$$t" || exit 1; done'

EXT_ZIP := dist/rstory@lucasvmigotto.github.io.shell-extension.zip
ext-pack: image ## Zip the extension for extensions.gnome.org (dev-only files excluded, as the installer does) into dist/, offline
	$(RUN) $(OFFLINE) $(IMAGE) python3 -I scripts/ext-pack.py extension $(EXT_ZIP)

# extensions.gnome.org's static analyzer. Shexli exits 0 even with findings, so the recipe requires "clean".
SHEXLI_IMAGE   := docker.io/library/python:3.13-slim@sha256:70729b46c69b4f1e97c4822c1af3df53a1476cf5ddc6c087c0c10bc3a5678c2f
SHEXLI_VERSION := 0.2.1
ext-shexli: ext-pack ## Check the zip with Shexli (extensions.gnome.org's analyzer); must be clean, networked (pip)
	$(ENGINE) run --rm $(OWN) -v "$(CURDIR)/dist":/ego:ro,Z $(SHEXLI_IMAGE) sh -c 'pip install -q --disable-pip-version-check --root-user-action=ignore shexli==$(SHEXLI_VERSION) && shexli /ego/$(notdir $(EXT_ZIP)) > /tmp/out; cat /tmp/out; grep -q "^shexli: clean" /tmp/out'

deny: image ## cargo-deny bans, licences, sources (fitness F3, F4), offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo deny --locked --offline check bans licenses sources

audit: image ## RustSec advisories via cargo-deny: vulnerabilities fail, unmaintained warns (fetches the advisory DB: networked)
	$(RUN) $(IMAGE) cargo deny --locked check --warn unmaintained advisories

secret-scan: image ## gitleaks over the working tree (redacted output), offline
	$(RUN) $(OFFLINE) $(IMAGE) gitleaks dir --redact --no-banner --exit-code 1 .

publish-dry-run: image ## cargo publish --dry-run (packages and verifies the crate; networked)
	$(RUN) $(IMAGE) cargo publish --locked --dry-run

publish: image ## cargo publish --no-verify (publish-dry-run verified the same commit; no build code runs with the token); reads CARGO_REGISTRY_TOKEN from the caller's environment, never from a file (networked)
	@[ -n "$$CARGO_REGISTRY_TOKEN" ] || { echo "CARGO_REGISTRY_TOKEN is not set (see docs/release.md)"; exit 1; }
	$(RUN) -e CARGO_REGISTRY_TOKEN $(IMAGE) cargo publish --locked --no-verify

sc001-test: ## Fixture test of the SC-001 token check (.github/scripts/no-publish-token.sh) and the release workflows' shape; pure bash, no container
	scripts/test-no-publish-token.sh
	scripts/test-release-workflows.sh

doctor: image ## Print tool versions per layer; fail on drift from rust-toolchain.toml
	@echo "rust-toolchain.toml: $(RUST_VERSION)"
	@$(RUN) $(OFFLINE) $(IMAGE) sh -c 'v=$$(rustc --version | cut -d" " -f2); echo "tools rustc: $$v"; \
	  echo "tools cargo: $$(cargo --version | cut -d" " -f2)"; rustup target list --installed | sed "s/^/tools target: /"; \
	  echo "sway: $$(sway --version)"; echo "gjs: $$(gjs --version)"; \
	  echo "cargo-deny: $$(cargo deny --version)"; echo "nextest: $$(cargo nextest --version | head -1)"; echo "gitleaks: $$(gitleaks version)"; [ "$$v" = "$(RUST_VERSION)" ] || { echo "DRIFT: tools rustc $$v != $(RUST_VERSION)"; exit 1; }'
	@if command -v rustc >/dev/null 2>&1; then echo "host rustc: $$(cd $(CURDIR) && RUSTUP_AUTO_INSTALL=0 rustc --version 2>/dev/null | cut -d' ' -f2) (informational)"; fi

shell: image ## Interactive shell in the tools image (networked)
	$(RUN) $(TTY) $(IMAGE) bash

clean: ## Remove this worktree's labelled containers and volumes and our dangling images; dry=1 only lists (keeps the main caches)
	@sel=label=$(WKEY)=$(PROJECT)/$(WT); \
	for c in $$($(ENGINE) ps -aq --filter $$sel); do echo "container $$c"; [ -n "$(dry)" ] || $(ENGINE) rm -f $$c >/dev/null; done; \
	if [ "$(WT)" != main ]; then for v in $$($(ENGINE) volume ls -q --filter $$sel); do echo "volume $$v"; [ -n "$(dry)" ] || $(ENGINE) volume rm $$v >/dev/null; done; fi; \
	for i in $$($(ENGINE) images -q --filter dangling=true --filter label=$(PKEY)=$(PROJECT)); do echo "image $$i"; [ -n "$(dry)" ] || $(ENGINE) rmi $$i >/dev/null; done

clean-cache: ## Remove this project's cache volumes
	-$(ENGINE) volume rm rstory-tools-cargo $(TARGET_VOLUME) $(DOCS_NODE_VOLUME) $(DOCS_CACHE_VOLUME)

# Docs site (docs/site): Bun runs only in this pinned container, never on the host (discovery section 7).
# docker.io/oven/bun:1.4.2, the multi-arch index digest. docs/site/Containerfile repeats it for Dependabot.
DOCS_IMAGE  := docker.io/oven/bun:1.4.2@sha256:9114c058aeae42162ee16dd5084b95fe9473970bb6bcb5b232ab1630f0546895
DOCS_NODE_VOLUME  ?= rstory-docs-node-modules
DOCS_CACHE_VOLUME ?= rstory-docs-bun-cache
# Git facts come from the host: a worktree's .git is a file pointing outside the mount, so the container can't ask git.
DOCS_GIT_SHA      := $(shell git rev-parse HEAD 2>/dev/null)
DOCS_GIT_TAG      := $(shell git describe --tags --exact-match --match '[0-9]*.[0-9]*.[0-9]*' 2>/dev/null)
DOCS_GIT_LAST_TAG := $(shell git describe --tags --abbrev=0 --match '[0-9]*.[0-9]*.[0-9]*' 2>/dev/null)
docs_ensure = $(call ensure_vol,$(DOCS_NODE_VOLUME),$(WT)) $(call ensure_vol,$(DOCS_CACHE_VOLUME),$(WT))
DOCS_RUN := $(docs_ensure) $(ENGINE) run --rm --init $(USERNS) $(OWN) --label $(WKEY)=$(PROJECT)/$(WT) \
	--cpus $(CPUS) --memory $(MEMORY) --pids-limit 2048 \
	-v "$(CURDIR)":/src:Z \
	-v $(DOCS_NODE_VOLUME):/src/docs/site/node_modules \
	-v $(DOCS_CACHE_VOLUME):/bun-cache \
	-w /src/docs/site \
	-e BUN_INSTALL_CACHE_DIR=/bun-cache -e HOME=/tmp -e TZ=UTC -e LANG=C.UTF-8 \
	-e RSTORY_GIT_SHA=$(DOCS_GIT_SHA) -e RSTORY_GIT_TAG=$(DOCS_GIT_TAG) -e RSTORY_GIT_LAST_TAG=$(DOCS_GIT_LAST_TAG)

docs-install: ## Install the docs site's packages into a volume (bun install --frozen-lockfile; the only networked docs step)
	$(DOCS_RUN) $(DOCS_IMAGE) bun install --frozen-lockfile

docs-dev: ## Serve the docs site with hot reload on http://127.0.0.1:5173/rstory/ (publishes a port, so not offline)
	$(DOCS_RUN) -p 127.0.0.1:5173:5173 $(TTY) $(DOCS_IMAGE) bun run dev --host 0.0.0.0 --port 5173 --strictPort

docs-check: ## Docs site: Biome lint, typecheck, unit, accessibility, footer and LLM-output tests, offline
	$(DOCS_RUN) $(OFFLINE) $(DOCS_IMAGE) sh -c 'bun run lint && bun run typecheck && bun run test'

docs-fmt: ## Format and fix the docs site with Biome (writes files), offline
	$(DOCS_RUN) $(OFFLINE) $(DOCS_IMAGE) bun run format

docs-build: ## Build docs/site/dist (pages, llms.txt, llms-full.txt, one .md per page), offline
	$(DOCS_RUN) $(OFFLINE) $(DOCS_IMAGE) bun run build

docs-reference: image ## Regenerate the CLI reference from the built binary's --help (tools image), offline
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'cargo build --locked --offline && python3 -I docs/site/scripts/cli-help.py /target/debug/rstory docs/site/src/content/generated/cli-help.json'
