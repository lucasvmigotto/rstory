# rstory task recipes — one entry point for people, agents and CI.
# Every recipe runs in the `tools` image (containers.md §6). Podman first, Docker fallback.

ENGINE ?= $(or $(CONTAINER_ENGINE),$(shell command -v podman >/dev/null 2>&1 && echo podman || echo docker))
IMAGE  := localhost/rstory-tools:dev
RUST_VERSION := $(shell sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)

# Per-worktree build cache, so parallel worktrees don't share one target dir
# (e.g. make TARGET_VOLUME=rstory-tools-target-x11 test).
TARGET_VOLUME ?= rstory-tools-target

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

RUN := $(ENGINE) run --rm --init $(USERNS) \
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
NPM_VOLUME ?= rstory-tools-npm
EXT_MOUNT := -v $(NPM_VOLUME):/src/extension/node_modules
TTY := $(shell [ -t 0 ] && echo -it)

.PHONY: help image fetch fmt fmt-check lint check test test-quarantine bench test-integration test-e2e test-e2e-smoke test-perf test-soak build build-static test-launch ext-lint ext-test ext-deps ext-typecheck deny audit secret-scan publish-dry-run publish doctor shell clean-cache

help: ## List recipes
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-17s %s\n", $$1, $$2}'

image: ## Build the tools image (from Containerfile, target tools)
	$(ENGINE) build --target tools -t $(IMAGE) -f Containerfile .

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
	$(RUN) $(if $(CI),-e CI=$(CI)) $(OFFLINE) $(IMAGE) $(NEXTEST) --features perf-tests --success-output immediate --test-threads 1 $(or $(TEST_ARGS),--test capture_latency --test burst --test idle)

test-soak: image ## Soak (nightly): the daemon for 30 min with 1 copy a minute, RSS flat, wake-ups only at timers; SOAK_SECS=n shortens it
	$(RUN) $(if $(SOAK_SECS),-e RSTORY_SOAK_SECS=$(SOAK_SECS)) $(OFFLINE) $(IMAGE) $(NEXTEST) --features perf-tests --success-output immediate --test-threads 1 --test soak

build: image ## Debug build, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo build --locked --offline

build-static: image ## Static musl release build for this host's arch + F2/F5 checks
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 't=$$(uname -m)-unknown-linux-musl; \
	  cargo build --locked --offline --release --target $$t && \
	  b=/target/$$t/release/rstory && file $$b && file $$b | grep -Eq "static(-pie)? linked|statically linked" && \
	  s=$$(stat -c %s $$b) && echo "size $$s bytes" && [ $$s -le 15728640 ]'

ext-lint: image ## Lint the GNOME extension (eslint from the npm volume), offline
	$(RUN) $(OFFLINE) $(EXT_MOUNT) -w /src/extension $(IMAGE) npx --no-install eslint .

ext-deps: image ## Install the extension's dev tools into the npm volume (the only networked ext step)
	$(RUN) $(EXT_MOUNT) -w /src/extension $(IMAGE) npm ci --no-audit --no-fund

ext-typecheck: image ## Type-check the extension (tsc --checkJs, JSDoc), offline
	$(RUN) $(OFFLINE) $(EXT_MOUNT) -w /src/extension $(IMAGE) npx --no-install tsc -p jsconfig.json

ext-test: image ## Run the extension's gjs fixture tests (fake socket server), offline
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'for t in extension/tests/*.test.js; do echo "== $$t"; gjs -m "$$t" || exit 1; done'

deny: image ## cargo-deny bans, licences, sources (fitness F3, F4), offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo deny --locked --offline check bans licenses sources

audit: image ## RustSec advisories via cargo-deny: vulnerabilities fail, unmaintained warns (fetches the advisory DB: networked)
	$(RUN) $(IMAGE) cargo deny --locked check --warn unmaintained advisories

secret-scan: image ## gitleaks over the working tree (redacted output), offline
	$(RUN) $(OFFLINE) $(IMAGE) gitleaks dir --redact --no-banner --exit-code 1 .

publish-dry-run: image ## cargo publish --dry-run (packages and verifies the crate; networked)
	$(RUN) $(IMAGE) cargo publish --locked --dry-run

publish: image ## cargo publish; reads CARGO_REGISTRY_TOKEN from the caller's environment, never from a file (networked)
	@[ -n "$$CARGO_REGISTRY_TOKEN" ] || { echo "CARGO_REGISTRY_TOKEN is not set (see docs/release.md)"; exit 1; }
	$(RUN) -e CARGO_REGISTRY_TOKEN $(IMAGE) cargo publish --locked

doctor: image ## Print tool versions per layer; fail on drift from rust-toolchain.toml
	@echo "rust-toolchain.toml: $(RUST_VERSION)"
	@$(RUN) $(OFFLINE) $(IMAGE) sh -c 'v=$$(rustc --version | cut -d" " -f2); echo "tools rustc: $$v"; \
	  echo "tools cargo: $$(cargo --version | cut -d" " -f2)"; rustup target list --installed | sed "s/^/tools target: /"; \
	  echo "sway: $$(sway --version)"; echo "gjs: $$(gjs --version)"; \
	  echo "cargo-deny: $$(cargo deny --version)"; echo "nextest: $$(cargo nextest --version | head -1)"; echo "gitleaks: $$(gitleaks version)"; [ "$$v" = "$(RUST_VERSION)" ] || { echo "DRIFT: tools rustc $$v != $(RUST_VERSION)"; exit 1; }'
	@if command -v rustc >/dev/null 2>&1; then echo "host rustc: $$(cd $(CURDIR) && RUSTUP_AUTO_INSTALL=0 rustc --version 2>/dev/null | cut -d' ' -f2) (informational)"; fi

shell: image ## Interactive shell in the tools image (networked)
	$(RUN) $(TTY) $(IMAGE) bash

clean-cache: ## Remove this project's cache volumes
	-$(ENGINE) volume rm rstory-tools-cargo $(TARGET_VOLUME)
