# Research: Wayland data-control capture

Phase 0 for `003-wayland-capture`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Which protocol

- **Decision**: `ext-data-control-v1` (standard, KDE 6.x, wlroots ≥0.19) with `zwlr-data-control-v1` fallback (older wlroots/KDE)
- **Alternatives rejected**: only zwlr (deprecated path)

## R2. Binding generation

- **Decision**: wayland-protocols crates; vendor XML via `wayland-scanner` macros if a version lags
- **Alternatives rejected**: smithay-client-toolkit (more surface than needed)

## R3. Reading offers

- **Decision**: `receive(mime, write_fd)` + non-blocking read end in `AsyncFd`, limit + timeout
- **Alternatives rejected**: blocking read (stalls the loop)

## R4. Serving

- **Decision**: data-control source offering text MIME set; each `send(fd)` handled in its own task writing with EPIPE tolerance
- **Alternatives rejected**: synchronous writes
