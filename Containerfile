# syntax=docker/dockerfile:1.27.1@sha256:4edf897a3ffa55b89f906fc8cc78afdb3f1834cc9c7083565e611a8a7d5fe99e
#
# rstory — one module, one multi-stage Containerfile (containers.md §2):
#   base → deps → tools (agents + CI run every task here) → build → runtime
# Not on dhi.io/rust:*-dev: that image ships no rustup, so the musl targets the
# static-build check (fitness F2) needs cannot be added. See README "Development".

FROM docker.io/library/rust:1.98.1-slim-trixie@sha256:4cd829461bd5c4d511c32e269da9cb8929223b666519d8004e35fc8d1d771ab7 AS base
# tag: 1.98.1-slim-trixie (multi-arch index digest)
ENV TZ=UTC \
    LANG=C.UTF-8 \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:$PATH
# Install exactly the toolchain, components and targets rust-toolchain.toml declares.
RUN --mount=type=bind,source=rust-toolchain.toml,target=/tmp/tc/rust-toolchain.toml \
    cd /tmp/tc && rustup toolchain install && rustup show active-toolchain \
    && chmod -R a+rX "$RUSTUP_HOME" /usr/local/cargo

FROM base AS deps
# Test-time session stack only (the product itself has no runtime dependencies):
# Xvfb + headless sway with XWayland for capture/restore integration tests,
# a session D-Bus for the tray tests, reference clipboard clients, gjs
# + Node/npm (type-check and eslint only, never in the release build; the JS tools come from
# `make ext-deps`) for the GNOME extension, rsvg-convert to render the icon PNGs from their SVG sources.
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        xvfb sway mutter xwayland dbus-daemon dbus-x11 \
        wl-clipboard xclip x11-utils \
        gjs nodejs npm \
        jq socat file procps \
        librsvg2-bin \
        curl ca-certificates git
# CI gate tools (spec 013), prebuilt release binaries checked against pinned SHA-256 sums:
# cargo-deny (fitness F3 bans, F4 licences, RustSec advisories), cargo-nextest (flaky-test
# policy: retries and JUnit, test-strategy §8), gitleaks (secret scan on PRs). git: cargo-deny
# clones the RustSec advisory DB with it (make audit).
ARG CARGO_DENY_VERSION=0.20.2
ARG CARGO_DENY_SHA256_X86_64=9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f
ARG CARGO_DENY_SHA256_AARCH64=995c82be0defc7a025cae49a2aa2644ce8245c9a3318fc4103907c6a285e8c7d
ARG NEXTEST_VERSION=0.9.148
ARG NEXTEST_SHA256_X86_64=bf92cb81450872d960ffa13857a83084d557ec0cdc141663d263bb23ef756f97
ARG NEXTEST_SHA256_AARCH64=94676506ceaed01746d78cfe9ed124ee841b2932fd18d9d10f438a347bd9ba5e
ARG GITLEAKS_VERSION=8.30.1
ARG GITLEAKS_SHA256_X86_64=551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb
ARG GITLEAKS_SHA256_AARCH64=e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080
RUN set -eu; arch=$(uname -m); \
    case "$arch" in \
      x86_64) deny_sum=$CARGO_DENY_SHA256_X86_64; nt_sum=$NEXTEST_SHA256_X86_64; gl_sum=$GITLEAKS_SHA256_X86_64; gl_arch=x64 ;; \
      aarch64) deny_sum=$CARGO_DENY_SHA256_AARCH64; nt_sum=$NEXTEST_SHA256_AARCH64; gl_sum=$GITLEAKS_SHA256_AARCH64; gl_arch=arm64 ;; \
      *) echo "unsupported arch $arch" >&2; exit 1 ;; \
    esac; \
    d=$(mktemp -d); cd "$d"; \
    fetch() { curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$1" "$2" && echo "$3  $1" | sha256sum -c -; }; \
    fetch deny.tgz "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/cargo-deny-${CARGO_DENY_VERSION}-${arch}-unknown-linux-musl.tar.gz" "$deny_sum"; \
    fetch nextest.tgz "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${NEXTEST_VERSION}/cargo-nextest-${NEXTEST_VERSION}-${arch}-unknown-linux-musl.tar.gz" "$nt_sum"; \
    fetch gitleaks.tgz "https://github.com/gitleaks/gitleaks/releases/download/v${GITLEAKS_VERSION}/gitleaks_${GITLEAKS_VERSION}_linux_${gl_arch}.tar.gz" "$gl_sum"; \
    tar -xzf deny.tgz --strip-components=1 "cargo-deny-${CARGO_DENY_VERSION}-${arch}-unknown-linux-musl/cargo-deny"; \
    tar -xzf nextest.tgz cargo-nextest; \
    tar -xzf gitleaks.tgz gitleaks; \
    install -m 0755 cargo-deny cargo-nextest gitleaks /usr/local/bin/; \
    cd /; rm -rf "$d"; \
    cargo-deny --version; cargo-nextest nextest --version; gitleaks version
ARG UID=1000
RUN useradd --uid "${UID}" --create-home --home-dir /home/dev --shell /bin/bash dev \
    && install -d -o dev -g dev /home/dev/.cargo /target /run/user/1000 \
    && install -d -m 1777 /tmp/.X11-unix

FROM deps AS tools
USER 1000
ENV HOME=/home/dev \
    CARGO_HOME=/home/dev/.cargo \
    CARGO_TARGET_DIR=/target \
    XDG_RUNTIME_DIR=/run/user/1000 \
    WLR_BACKENDS=headless \
    WLR_RENDERER=pixman \
    WLR_LIBINPUT_NO_DEVICES=1
WORKDIR /src
LABEL org.opencontainers.image.source="https://github.com/lucasvmigotto/rstory" \
      org.opencontainers.image.title="rstory-tools"

FROM tools AS build
ARG TARGETARCH
RUN --mount=type=bind,target=/src \
    --mount=type=cache,target=/home/dev/.cargo/registry,uid=1000 \
    --mount=type=cache,target=/target,uid=1000 \
    case "${TARGETARCH:-amd64}" in arm64) t=aarch64-unknown-linux-musl ;; *) t=x86_64-unknown-linux-musl ;; esac \
    && cargo build --release --locked --target "$t" \
    && mkdir -p /home/dev/out && cp "/target/$t/release/rstory" /home/dev/out/rstory

FROM scratch AS runtime
COPY --from=build --chown=65532:65532 /home/dev/out/rstory /rstory
USER 65532
ENTRYPOINT ["/rstory"]
