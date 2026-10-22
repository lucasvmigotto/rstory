# Quickstart: Autostart

How to exercise this feature once built.

```bash
cargo run -- install-autostart && cat ~/.config/autostart/rstory.desktop
cargo run -- install-autostart --systemd --now && systemctl --user status rstory
cargo run -- install-autostart --uninstall
```
