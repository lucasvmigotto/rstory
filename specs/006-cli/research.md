# Research: Command-line interface

Phase 0 for `006-cli`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Client I/O

- **Decision**: blocking `std::os::unix::net::UnixStream` in the CLI (no runtime startup cost)
- **Alternatives rejected**: tokio in short-lived commands (startup ms)

## R2. Relative time

- **Decision**: own ~30-line formatter ("3m", "2h", "5d")
- **Alternatives rejected**: chrono-humanize (dependency)

## R3. Prompt

- **Decision**: read one line from /dev/tty when stdin is not a TTY
- **Alternatives rejected**: dialoguer (dependency for one prompt)
