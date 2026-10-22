# syntax=docker/dockerfile:1
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
# a session D-Bus for the tray tests, reference clipboard clients, gjs + eslint
# for the GNOME extension, rsvg-convert to render the icon PNGs from their SVG sources.
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        xvfb sway mutter xwayland dbus-daemon dbus-x11 \
        wl-clipboard xclip x11-utils \
        gjs eslint \
        jq socat file procps \
        librsvg2-bin
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
