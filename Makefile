# rstory task recipes — one entry point for people, agents and CI.
# Every recipe runs in the `tools` image (containers.md §6). Podman first, Docker fallback.

ENGINE ?= $(or $(CONTAINER_ENGINE),$(shell command -v podman >/dev/null 2>&1 && echo podman || echo docker))
IMAGE  := localhost/rstory-tools:dev
RUST_VERSION := $(shell sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)

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
	-v rstory-tools-target:/target \
	$(RUNDIR) \
	-e TZ=UTC -e LANG=C.UTF-8 -e RUST_BACKTRACE=1 -e CARGO_TERM_COLOR=always
OFFLINE := --network=none
# Test-only feature: FaultFs for the crash and power-loss suites (never in release builds).
FEATURES := --features fault-injection
TTY := $(shell [ -t 0 ] && echo -it)

.PHONY: help image fetch fmt fmt-check lint check test test-integration build build-static ext-lint ext-test doctor shell clean-cache

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
	$(RUN) $(OFFLINE) $(IMAGE) cargo clippy --locked --offline $(FEATURES) --all-targets -- -D warnings

check: image ## Type-check, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo check --locked --offline $(FEATURES) --all-targets

test: image ## Unit, property, snapshot and contract tests, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo test --locked --offline $(FEATURES)

test-integration: image ## Session tests (Xvfb, headless sway, bridge mode), offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo test --locked --offline --features session-tests -- --test-threads=2

build: image ## Debug build, offline
	$(RUN) $(OFFLINE) $(IMAGE) cargo build --locked --offline

build-static: image ## Static musl release build for this host's arch + F2/F5 checks
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 't=$$(uname -m)-unknown-linux-musl; \
	  cargo build --locked --offline --release --target $$t && \
	  b=/target/$$t/release/rstory && file $$b && file $$b | grep -Eq "static(-pie)? linked|statically linked" && \
	  s=$$(stat -c %s $$b) && echo "size $$s bytes" && [ $$s -le 15728640 ]'

ext-lint: image ## Lint the GNOME extension
	$(RUN) $(OFFLINE) $(IMAGE) eslint extension/

ext-test: image ## Run the extension's gjs fixture tests
	$(RUN) $(OFFLINE) $(IMAGE) sh -c 'for t in extension/tests/*.test.js; do gjs -m "$$t" || exit 1; done'

doctor: image ## Print tool versions per layer; fail on drift from rust-toolchain.toml
	@echo "rust-toolchain.toml: $(RUST_VERSION)"
	@$(RUN) $(OFFLINE) $(IMAGE) sh -c 'v=$$(rustc --version | cut -d" " -f2); echo "tools rustc: $$v"; \
	  echo "tools cargo: $$(cargo --version | cut -d" " -f2)"; rustup target list --installed | sed "s/^/tools target: /"; \
	  echo "sway: $$(sway --version)"; echo "gjs: $$(gjs --version)"; [ "$$v" = "$(RUST_VERSION)" ] || { echo "DRIFT: tools rustc $$v != $(RUST_VERSION)"; exit 1; }'
	@if command -v rustc >/dev/null 2>&1; then echo "host rustc: $$(cd $(CURDIR) && RUSTUP_AUTO_INSTALL=0 rustc --version 2>/dev/null | cut -d' ' -f2) (informational)"; fi

shell: image ## Interactive shell in the tools image (networked)
	$(RUN) $(TTY) $(IMAGE) bash

clean-cache: ## Remove this project's cache volumes
	-$(ENGINE) volume rm rstory-tools-cargo rstory-tools-target
