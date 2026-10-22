# Research: Picker launch

Phase 0 for `008-picker-launch`. Every Technical Context item is resolved; no `NEEDS CLARIFICATION` remains.

## R1. Exec syntax

- **Decision**: per-terminal table: ghostty -e, kitty, alacritty -e, foot, wezterm start --, ptyxis --, kgx --, gnome-terminal --, konsole -e, xfce4-terminal -x, xterm -e
- **Alternatives rejected**: assume `-e` everywhere (breaks gnome-terminal, wezterm, kitty)

## R2. Standard

- **Decision**: `xdg-terminal-exec` before the list (emerging freedesktop proposal)
- **Alternatives rejected**: only list
