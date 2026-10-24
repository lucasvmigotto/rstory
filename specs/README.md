# rstory — feature index

Owner: `project:spec`. Build and QA stages update only the status columns.
Statuses: Planned · In progress · Implemented · Verified (see
`.specify/memory/constitution.md` and the pipeline rules).

rstory has no web frontend. Its "frontend" layer is the terminal and menu
surfaces: the CLI, picker, tray and GNOME extension. Those surfaces are
specified in `tui.md` by `frontend:spec`, and built by `frontend:tui`.
The "backend" layer is the daemon, store, backends and protocol
(`backend.md`, `backend:build`). A dash (—) means the feature has no work
in that layer.

## MVP (v0.1)

| # | Feature | Scope | Priority | Depends on | Frontend | Backend |
|---|---|---|---|---|---|---|
| 000 | [design-system](000-design-system/) | Terminal and menu design system: color roles × tiers, glyphs with ASCII fallback, keyed copy, components, icons | P1 | — | Implemented | — |
| 001 | [store](001-store/) | Entry model, history rules (de-dup, count/pin/size/expiry caps), crash-safe private store, read-only open | P1 | — | — | Implemented |
| 002 | [x11-capture](002-x11-capture/) | Session detection, `Backend` trait, X11 backend (XFIXES capture and restore, INCR), XWayland bridge selection (GNOME path) | P1 | — | — | Implemented |
| 003 | [wayland-capture](003-wayland-capture/) | Data-control backend (`ext-data-control-v1`, `zwlr` fallback), capture and restore | P1 | 002 | — | Implemented |
| 004 | [daemon](004-daemon/) | `rstory daemon`: single instance, control socket protocol v1, capture pipeline, restore ownership, pause and timed pause, counters, expiry timer, inactive state | P1 | 001, 002 | — | Implemented |
| 005 | [privacy-filters](005-privacy-filters/) | Sensitive hint skip, size-cap drop, pause gate, no-content logging | P1 | 004 | — | Implemented |
| 006 | [cli](006-cli/) | `ls`, `show`, `copy`, `pin`, `unpin`, `rm`, `clear`, `pause [--for]`, `resume`, `status`, `config`; `--json`; index and short id; exit codes; read-only when the daemon is down | P1 | 004 | Implemented | Implemented |
| 007 | [picker](007-picker/) | `rstory pick` TUI: search, preview, restore, pin, unpin, delete, plain mode | P1 | 000, 006 | Implemented | — |
| 008 | [picker-launch](008-picker-launch/) | `picker_command`, terminal detection order, daemon-side `open_picker` | P2 | 007 | Implemented | Implemented |
| 009 | [tray](009-tray/) | SNI tray: icon (normal and paused), last 10, pause, pause 1 min, open picker, clear with confirm | P2 | 004, 008 | Implemented | Implemented |
| 010 | [gnome-extension](010-gnome-extension/) | Embedded GNOME Shell 48–51 extension: panel icon and menu over the control socket; `install-gnome-extension` | P1 (v0.1 gate) | 004, 008 | Implemented | Implemented |
| 011 | [doctor](011-doctor/) | `rstory doctor`: session, backend, bridge, store privacy, daemon, extension, tray host, terminal search | P2 | 002, 003, 004, 010 | Planned | Planned |
| 012 | [autostart](012-autostart/) | `install-autostart`: XDG `.desktop` or systemd user unit | P2 | 004 | Implemented | Implemented |
| 013 | [release](013-release/) | CI build matrix (musl x86_64 and aarch64), Tier-1 integration jobs, checksums, GitHub Releases, Trusted Publishing, first-release runbook | P2 | 001–012 | — | Implemented |

## Next and Later (from the brief, not specified yet)

| Brief # | Feature | Priority | Depends on |
|---|---|---|---|
| 016 | GNOME popup list | Next | 010 |
| 017 | Images (`image/png`) | Next | 001, 002, 003, 007 |
| 018 | Primary selection (opt-in) | Later | 002, 003 |
| 019 | Encrypted store | Later | 001 |
| 020 | Pinned export/import | Later | 001 |
| 021 | Source app and ignore list | Later | 004 |
| 022 | Rich text and file lists | Later | 017 |
| 023 | Portal / extension-fed capture | Later | 002, 010 |

## Shared artifacts

- `contracts/protocol.md` + `contracts/protocol.schema.json`: the control
  socket protocol v1, the seam between clients and the daemon
  (Constitution IV). It plays the role `contracts/openapi.yaml` plays in
  web projects. The skeleton comes from `project:spec`, and it becomes
  canonical through `backend:spec`.
- `contracts/cli.md`: the CLI surface (commands, `--json` shapes, exit
  codes), owned by spec 006.
