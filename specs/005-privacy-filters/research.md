# Research: Privacy filters

Phase 0 for `005-privacy-filters`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Sensitive hint names

- **Decision**: `x-kde-passwordManagerHint` (KDE convention; set by KeePassXC; present in Firefox ESR binary; survives XWayland bridge — S1/S2)
- **Alternatives rejected**: `CLIPBOARD_STATE` (no Linux evidence — dropped)

## R2. pass gap

- **Decision**: timed pause + documented `passc` wrapper (owner decision)
- **Alternatives rejected**: entropy heuristics (false positives on hashes/ids)

## R3. Decide before read

- **Decision**: types available in the change event on both backends
- **Alternatives rejected**: read then inspect (content would be in memory)
