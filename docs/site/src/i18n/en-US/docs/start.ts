import type { PageContent } from "../../../content/types";

export const start: Record<"overview" | "install" | "first-run" | "desktops", PageContent> = {
  overview: {
    title: "rstory",
    summary: "A small, private clipboard history for Linux. Version 0.1.0 is not released yet.",
    blocks: [
      {
        t: "p",
        text: "rstory records what you copy and lets you restore it later. It ships as one static Rust binary that works the same on Wayland and X11, under any desktop or distribution, without installing anything else. It never uses the network and never stores a copy that an app marks as sensitive.",
      },
      {
        t: "callout",
        kind: "note",
        text: "rstory 0.1.0 has not been released. Building from source works today. The release tarballs and crates.io are [[planned]] until the first release; see [Install](#/install).",
      },
      { t: "h2", text: "What you can do" },
      {
        t: "ul",
        items: [
          "Find and restore a recent copy in a terminal picker: [Recall with the picker](#/guides/picker).",
          "Browse, restore, pin and delete entries from scripts: [Script with the CLI](#/guides/cli).",
          "Stop recording for a while, including around a password manager: [Pause and passc](#/guides/pause).",
          "Restore from the top bar with a tray icon, or a panel icon on GNOME: [Tray and GNOME panel icon](#/guides/tray).",
          "Check the setup and get a reason for every problem it can detect: [Troubleshoot with doctor](#/guides/doctor).",
        ],
      },
      { t: "h2", text: "How it works, in short" },
      {
        t: "p",
        text: "A long-running **daemon** captures clipboard changes and keeps the **history** in a private **store**. Everything else (the CLI, the picker, the tray, the panel icon) is a thin **client** that talks to the daemon over a **control socket**. When the daemon is down, the CLI and the picker still read the store, read-only.",
      },
      { t: "h2", text: "Privacy in one paragraph" },
      {
        t: "p",
        text: "A copy is skipped, without being read, when the app that owns it offers the type `x-kde-passwordManagerHint`. Copies over 1 MiB are dropped whole, and nothing is recorded while paused. Apps that set no hint, such as `pass`, are recorded like any other text. The details and the workaround are in [Privacy and threat model](#/concepts/privacy).",
      },
      { t: "h2", text: "How to read the status labels" },
      {
        t: "p",
        text: "Every page carries a label for what is real today. A label never describes intent as behavior.",
      },
      {
        t: "table",
        head: ["Label", "Meaning"],
        rows: [
          ["[[implemented]]", "Works today, traced to code and tests."],
          ["[[partial]]", "Some of it works today; the page says which part."],
          ["[[planned]]", "Intended, not built, or built but not released yet."],
          ["[[unavailable]]", "Mentioned in plans or docs, but the code does not do it."],
          [
            "[[upstream]]",
            "A limit of the desktop, a protocol or an application, outside rstory's control.",
          ],
        ],
      },
    ],
  },

  install: {
    title: "Install",
    summary: "Building from source works today; every other channel starts with the first release.",
    blocks: [
      {
        t: "callout",
        kind: "planned",
        text: "Nothing is released yet. There are no tags, no published crate and no release tarballs. The channels below are labelled with what they are today.",
      },
      {
        t: "table",
        head: ["Channel", "Status"],
        rows: [
          ["Build from source with `make build-static`", "[[implemented]]"],
          ["`cargo install rstory` from crates.io", "[[planned]] from 0.1.0"],
          ["GitHub Release tarballs and `SHA256SUMS`", "[[planned]] from 0.1.0"],
          ["GNOME panel icon from extensions.gnome.org", "[[planned]] after 0.1.0"],
          ["Distribution packages (AUR, deb, rpm, apk, Nix)", "[[planned]] (Later)"],
        ],
      },
      { t: "h2", text: "Build from source" },
      {
        t: "p",
        text: "You need Podman (preferred) or Docker, and `make`. Every task runs in the project's `tools` container, so no Rust toolchain is needed on the host.",
      },
      {
        t: "code",
        lang: "sh",
        text: "make image         # build the tools image\nmake fetch         # resolve crates into the cache volume (the only networked step)\nmake build-static  # static musl release build, with the static-binary and size checks",
      },
      {
        t: "p",
        text: "`make build-static` builds `<arch>-unknown-linux-musl` for your machine's architecture, checks that the binary is statically linked and at most 15 MiB, and leaves it in the build cache volume. No recipe copies it to the host yet.",
      },
      { t: "h2", text: "From 0.1.0" },
      {
        t: "p",
        text: "[[planned]] The first release will publish `rstory` on crates.io, and the release workflow will attach one tarball per architecture, `rstory-X.Y.Z-x86_64-unknown-linux-musl.tar.gz` and `rstory-X.Y.Z-aarch64-unknown-linux-musl.tar.gz`, with a `SHA256SUMS` file. Each tarball holds the `rstory` binary, `LICENSE` and `README.md`. Tags are plain `X.Y.Z`, with no `v`. See [Releases and versions](#/project/releases).",
      },
      {
        t: "code",
        lang: "sh",
        caption: "Planned: not available until 0.1.0 is released",
        text: "cargo install rstory",
      },
      {
        t: "p",
        text: "Next: [First run](#/first-run).",
      },
    ],
  },

  "first-run": {
    title: "First run",
    summary: "Start the daemon, check the setup, and make rstory start at login.",
    blocks: [
      {
        t: "p",
        text: "These steps assume you have an `rstory` binary on your `PATH`; see [Install](#/install).",
      },
      { t: "h2", text: "1. Check the setup" },
      {
        t: "code",
        lang: "sh",
        text: "rstory doctor",
      },
      {
        t: "p",
        text: "`doctor` checks the session, the capture backend, the store, the daemon, the pause state, the tray, the GNOME Shell and extension, the picker terminal and autostart. Each problem comes with a hint. See [Troubleshoot with doctor](#/guides/doctor).",
      },
      { t: "h2", text: "2. Start the daemon" },
      {
        t: "code",
        lang: "sh",
        text: "rstory daemon",
      },
      {
        t: "p",
        text: "The daemon runs in the foreground and logs to stderr (`RUST_LOG`, default `info`). Copy some text in any app, then list the history from another terminal:",
      },
      { t: "code", lang: "sh", text: "rstory ls\nrstory pick" },
      { t: "h2", text: "3. Start at login" },
      {
        t: "code",
        lang: "sh",
        text: "rstory install-autostart            # XDG autostart entry\nrstory install-autostart --systemd  # systemd user unit instead\nrstory install-autostart --now      # also start the daemon now\nrstory install-autostart --uninstall",
      },
      {
        t: "p",
        text: "The entry is `$XDG_CONFIG_HOME/autostart/rstory.desktop`; with `--systemd` the unit is `$XDG_CONFIG_HOME/systemd/user/rstory.service`. `--uninstall` removes both kinds and can't be combined with `--systemd` or `--now`. On success it prints:",
      },
      {
        t: "code",
        lang: "text",
        text: "Autostart installed: {path}. rstory starts at your next login, or run `rstory daemon` now.",
      },
      { t: "h2", text: "4. On GNOME: the panel icon" },
      {
        t: "p",
        text: "GNOME has no tray, so rstory ships a small GNOME Shell extension inside the binary (GNOME Shell 48 to 50).",
      },
      {
        t: "code",
        lang: "sh",
        text: "rstory install-gnome-extension           # install it\nrstory install-gnome-extension --enable  # install and enable it\nrstory install-gnome-extension --uninstall",
      },
      {
        t: "p",
        text: "The extension goes to `$XDG_DATA_HOME/gnome-shell/extensions/rstory@lucasvmigotto.github.io/`. Without `--enable` it prints:",
      },
      {
        t: "code",
        lang: "text",
        text: "GNOME extension installed. Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.github.io",
      },
      {
        t: "p",
        text: "GNOME Shell 51 is [[unavailable]] until its manual check passes. See [Tray and GNOME panel icon](#/guides/tray).",
      },
      { t: "h2", text: "5. Shell completions" },
      {
        t: "code",
        lang: "sh",
        text: "rstory completions bash   # or zsh, or fish",
      },
    ],
  },

  desktops: {
    title: "Your desktop",
    summary: "Which sessions rstory captures on, and which backend it picks.",
    blocks: [
      {
        t: "p",
        text: "rstory picks its **backend** from your **session**, so the same commands work everywhere. You can force one with `rstory daemon --backend x11|wayland`, or with the `backend` key in `config.toml`; the flag wins.",
      },
      {
        t: "table",
        head: ["Session", "Backend", "Status"],
        rows: [
          ["X11", "`x11`, native", "[[implemented]]"],
          ["Wayland with `ext-data-control-v1`", "`wayland`", "[[implemented]]"],
          [
            "Wayland with `zwlr_data_control_manager_v1` (fallback)",
            "`wayland`",
            "[[implemented]]",
          ],
          [
            "Wayland without data-control, with XWayland (GNOME)",
            "`x11`, through the XWayland bridge",
            "[[implemented]]",
          ],
          [
            "Wayland with neither data-control nor XWayland",
            "none: the daemon is inactive",
            "[[upstream]]",
          ],
        ],
      },
      {
        t: "p",
        text: "How the choice works is in [Backends and the XWayland bridge](#/concepts/backends).",
      },
      { t: "h2", text: "Environments checked before a release" },
      {
        t: "p",
        text: "These are the Tier 1 environments of the product brief. CI covers X11 (Xvfb), sway and a GNOME bridge-mode session headlessly. GNOME and KDE are also checked by hand before each release, because headless mutter has no keyboard seat. The release matrix is in [Releases and versions](#/project/releases).",
      },
      {
        t: "table",
        head: ["Environment", "What is checked", "Status"],
        rows: [
          [
            "GNOME Shell 48, 49, 50 on Wayland",
            "capture through the XWayland bridge, and the panel icon",
            "[[implemented]]",
          ],
          [
            "GNOME Shell 51 on Wayland",
            "capture is in the manual matrix; the panel icon is not declared",
            "[[unavailable]]",
          ],
          [
            "KDE Plasma 6 on Wayland",
            "capture and the tray; the manual KDE check belongs to the release sign-off and is not recorded yet",
            "[[implemented]]",
          ],
          ["sway with waybar", "capture and the tray", "[[implemented]]"],
          ["X11", "capture and restore", "[[implemented]]"],
        ],
      },
      {
        t: "callout",
        kind: "note",
        text: "Other compositors that offer a data-control protocol are expected to work. They are not in the release matrix.",
      },
      {
        t: "p",
        text: "rstory targets Linux on x86_64 and aarch64, with static musl binaries and any init system.",
      },
    ],
  },
};
