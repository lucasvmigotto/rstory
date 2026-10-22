# Research: Session detection & X11 capture (incl. XWayland bridge)

Phase 0 for `002-x11-capture`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Watching CLIPBOARD on X11

- **Decision**: XFIXES `SelectSelectionInput` on a hidden InputOnly window (no polling)
- **Alternatives rejected**: periodic `GetSelectionOwner` (polling — violates G6)

## R2. XWayland bridge viability

- **Decision**: Spike S1 (GNOME 48.7): unfocused x11rb client receives every Wayland copy with MIME targets; X11-served values are pasteable by Wayland clients
- **Alternatives rejected**: clipboard portal (prompt), extension-fed capture (Shell coupling) — Later

## R3. Large transfers

- **Decision**: Implement ICCCM INCR both directions; chunk size = max request length − overhead
- **Alternatives rejected**: refuse > 256 KiB (breaks the 1 MiB promise)

## R4. Serving targets

- **Decision**: TARGETS, TIMESTAMP, UTF8_STRING, STRING (Latin-1 lossy), TEXT, text/plain;charset=utf-8
- **Alternatives rejected**: MULTIPLE (rarely used; refuse cleanly)

## R5. Detecting data-control

- **Decision**: Wayland registry roundtrip, look for `ext_data_control_manager_v1`/`zwlr_data_control_manager_v1`
- **Alternatives rejected**: guess from XDG_CURRENT_DESKTOP (wrong for new compositors)

## R6. Self-owned detection

- **Decision**: compare `XFixesSelectionNotify.owner` with our window id
- **Alternatives rejected**: content comparison (racy)
