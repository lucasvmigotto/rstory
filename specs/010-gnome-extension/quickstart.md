# Quickstart: GNOME Shell extension

How to exercise this feature once built.

```bash
cargo run -- install-gnome-extension --enable
# log out/in on Wayland, then:
gnome-extensions info rstory@lucasvmigotto.me
journalctl --user -f /usr/bin/gnome-shell | grep -i rstory
```
