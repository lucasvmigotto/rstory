import type { PageContent } from "../../../content/types";

type P =
  | "project-architecture"
  | "project-roadmap"
  | "project-releases"
  | "project-limitations"
  | "project-accessibility"
  | "project-development"
  | "glossary";

export const project: Record<P, PageContent> = {
  "project-architecture": {
    title: "Architecture and decisions",
    summary: "How rstory is built, and the decisions behind it.",
    blocks: [
      {
        t: "p",
        text: "rstory is one Rust crate that builds one multi-call binary. The daemon is the only writer of the store; the CLI, picker, tray and GNOME extension are clients of its control socket. The full text is in [architecture.md](src:docs/product/architecture.md).",
      },
      { t: "h2", text: "Decision records" },
      {
        t: "table",
        head: ["ADR", "Decision"],
        rows: [
          [
            "[0001](src:docs/product/adr/0001-single-crate-multi-call-binary.md)",
            "One crate, one multi-call binary (modular monolith)",
          ],
          [
            "[0002](src:docs/product/adr/0002-daemon-single-writer-ndjson-socket.md)",
            "The daemon is the single writer; NDJSON protocol v1 over a Unix socket for every client",
          ],
          [
            "[0003](src:docs/product/adr/0003-store-index-plus-content-files.md)",
            "Store: an atomic index file plus write-once content files",
          ],
          [
            "[0004](src:docs/product/adr/0004-tokio-current-thread-runtime.md)",
            "tokio current_thread as the daemon's event loop",
          ],
          [
            "[0005](src:docs/product/adr/0005-backend-selection-xwayland-bridge.md)",
            "Backend selection, with the XWayland bridge on Wayland without data-control",
          ],
          [
            "[0006](src:docs/product/adr/0006-pure-rust-protocol-stack.md)",
            "A pure-Rust protocol stack: wayland-client, x11rb, zbus and ksni",
          ],
          [
            "[0007](src:docs/product/adr/0007-ratatui-picker-clap-cli.md)",
            "The picker in ratatui and crossterm, the CLI in clap, fuzzy search with nucleo-matcher",
          ],
          [
            "[0008](src:docs/product/adr/0008-embedded-gnome-extension-socket-client.md)",
            "GNOME extension: embedded ESM GJS, a control-socket client with no logic",
          ],
          [
            "[0009](src:docs/product/adr/0009-release-musl-trusted-publishing.md)",
            "Release: static musl builds on native runners; crates.io Trusted Publishing",
          ],
        ],
      },
      { t: "h2", text: "What is checked" },
      {
        t: "p",
        text: "Fitness tests in the repository check properties of the build: the binary is statically linked (F2) and at most 15 MiB (F5), dependency bans and licences (F3, F4), file permissions, the picker's first frame (F6), capture latency (F7) and that no clipboard content reaches logs. CI runs them on x86_64 and aarch64. See [Development](#/project/development).",
      },
    ],
  },

  "project-roadmap": {
    title: "Roadmap",
    summary: "What may come after 0.1.0. Nothing here is built.",
    blocks: [
      {
        t: "callout",
        kind: "planned",
        text: "Every item on this page is planned. None is specified in detail or implemented, and the order can change. The source is [specs/README.md](src:specs/README.md) and the product brief.",
      },
      {
        t: "table",
        head: ["Item", "When", "Status"],
        rows: [
          ["GNOME popup list of entries", "Next", "[[planned]]"],
          ["Images (`image/png`)", "Next", "[[planned]]"],
          ["Primary selection, opt-in", "Later", "[[planned]]"],
          ["Encrypted store", "Later", "[[planned]]"],
          ["Export and import of pinned entries", "Later", "[[planned]]"],
          ["Source application and an ignore list", "Later", "[[planned]]"],
          ["Rich text and file lists", "Later", "[[planned]]"],
          ["Portal or extension-fed capture", "Later", "[[planned]]"],
          ["Interface translations", "Later", "[[planned]]"],
          ["Paste injection", "Later", "[[planned]]"],
        ],
      },
    ],
  },

  "project-releases": {
    title: "Releases and versions",
    summary: "How rstory will be released, and what 0.1.0 requires.",
    blocks: [
      {
        t: "callout",
        kind: "planned",
        text: "Nothing has been released. The version in `Cargo.toml` is 0.1.0, there are no tags, and the release workflow has never run.",
      },
      { t: "h2", text: "How a release works" },
      {
        t: "ul",
        items: [
          "[[planned]] A tag of the form `X.Y.Z` (plain SemVer, no `v`) on `main`, equal to the `Cargo.toml` version, starts the release workflow.",
          "[[planned]] It reruns the main CI set, builds a static musl tarball for x86_64 and aarch64 with a `SHA256SUMS` file, and verifies the crate with `cargo publish --dry-run`.",
          "[[planned]] It publishes to crates.io with Trusted Publishing, a short-lived token from the job's identity, in a protected `release` environment; no publishing token is stored.",
          "[[planned]] Then it creates the GitHub Release with the tarballs.",
          "[[planned]] The first release is done by hand, then the token is retired. The runbook is [docs/release.md](src:docs/release.md).",
        ],
      },
      { t: "h2", text: "What CI does today" },
      {
        t: "p",
        text: "[[implemented]] Every pull request runs the CI gates on x86_64 and aarch64: formatting, lints, tests, session tests on Xvfb, sway and mutter, end-to-end journeys, the static build and its checks, advisories and a secret scan. A nightly run adds long property tests and a soak.",
      },
      { t: "h2", text: "Sign-off matrix" },
      {
        t: "p",
        text: "Before each release the maintainer checks GNOME Shell 48 to 51 on Wayland, KDE Plasma 6 on Wayland, sway with waybar, and the autostart re-login on GNOME, KDE and sway. A failing row blocks the release.",
      },
      { t: "h2", text: "Versions on these pages" },
      {
        t: "p",
        text: "The footer shows `rstory@v<version>` from `Cargo.toml` and `docs@v<version>` from the site's own `package.json`. Each links to the repository at the release tag, or at the commit when the build is past it. The docs site is deployed with the release and never before 0.1.0.",
      },
    ],
  },

  "project-limitations": {
    title: "Limitations",
    summary: "What rstory does not do, and why.",
    blocks: [
      {
        t: "table",
        head: ["Limitation", "Why", "Status"],
        rows: [
          [
            "Wayland with neither data-control nor XWayland: rstory is inactive",
            "A background client has no way to watch the clipboard.",
            "[[upstream]]",
          ],
          [
            "Apps that set no sensitive hint, such as `pass`, are recorded",
            "Another process running as you can read the clipboard too. Pause around the copy: [Pause and passc](#/guides/pause).",
            "[[upstream]]",
          ],
          [
            "A restored value disappears from the clipboard when the daemon stops",
            "Both clipboard protocols need a live owner to serve a value.",
            "[[upstream]]",
          ],
          [
            "GNOME Shell 51 panel icon",
            "Not declared by the extension until its manual check passes. `rstory doctor` warns that the panel icon needs GNOME Shell 48 to 50.",
            "[[unavailable]]",
          ],
          ["Release tarballs, crates.io, extensions.gnome.org", "Not released yet.", "[[planned]]"],
          [
            "Images, rich text, file lists and the primary selection",
            "Only text from the clipboard is recorded.",
            "[[planned]]",
          ],
          [
            "Encryption of the store",
            "Protection today is file permissions and expiry.",
            "[[planned]]",
          ],
        ],
      },
      {
        t: "p",
        text: "Only English is available for the interface.",
      },
    ],
  },

  "project-accessibility": {
    title: "Accessibility",
    summary: "How the picker, the CLI and the menus work without a mouse, color or sight.",
    blocks: [
      {
        t: "ul",
        items: [
          "Every action is available by keyboard in the picker, and by the host's keyboard navigation in menus. The CLI reaches every action too.",
          "State is never shown by color alone. The selection has a marker, a pinned entry has a glyph, and the words `Paused` and `Read-only` appear in text. Tray and panel icons change shape.",
          "`rstory pick --plain` is an inline mode for screen readers.",
          "`NO_COLOR` (non-empty) and `TERM=dumb` turn color off. A non-UTF-8 locale, or `--ascii`, switches to ASCII glyphs.",
          "The default theme uses your terminal's own palette, so your contrast settings apply.",
        ],
      },
      {
        t: "p",
        text: "The target is WCAG 2.2 AA principles, translated to terminal and menu surfaces. This documentation site is built to the same bar: semantic HTML, a skip link, visible focus, keyboard operation, reduced-motion support, status shown with words, and automated axe checks in the test suite.",
      },
    ],
  },

  "project-development": {
    title: "Development",
    summary: "Build, test and change rstory with containers and make.",
    blocks: [
      {
        t: "p",
        text: "You need Podman (preferred) or Docker, and `make`. Every task runs in the project's `tools` container, so no Rust toolchain is needed on the host. CI runs the same recipes.",
      },
      {
        t: "code",
        lang: "sh",
        text: "make help               # list recipes\nmake image              # build the tools image\nmake fetch              # resolve crates (the only networked step)\nmake fmt | fmt-check    # rustfmt\nmake lint               # clippy -D warnings, offline\nmake test               # unit, property, snapshot, contract and fitness tests, offline\nmake test-integration   # Xvfb, headless sway and mutter session tests\nmake build-static       # static musl release build and its checks\nmake ext-lint ext-test  # GNOME extension\nmake deny audit         # dependency policy and advisories\nmake doctor             # tool versions; fails on drift from rust-toolchain.toml",
      },
      { t: "h2", text: "This documentation site" },
      {
        t: "p",
        text: "The site lives in `docs/site`. Bun runs only in a pinned container, never on your machine. `make docs-install` is the only networked step.",
      },
      {
        t: "code",
        lang: "sh",
        text: "make docs-install     # packages into a volume\nmake docs-dev         # http://127.0.0.1:5173/rstory/\nmake docs-check       # lint, typecheck, unit, accessibility and LLM-output tests\nmake docs-build       # docs/site/dist, with llms.txt and a .md per page\nmake docs-reference   # regenerate the CLI reference from the built binary",
      },
      {
        t: "p",
        text: "The CLI reference is generated, never written by hand. After changing any `--help` text, run `make docs-reference` and commit the result; CI fails when it is stale.",
      },
      { t: "h2", text: "Branches and commits" },
      {
        t: "p",
        text: "Work happens on `feat/…`, `fix/…`, `docs/…` or `ci/…` branches off `dev` and lands through pull requests. Promotions to `main` are pull requests too. Commits follow Conventional Commits.",
      },
    ],
  },

  glossary: {
    title: "Glossary",
    summary: "The terms rstory uses everywhere: in the interface, the help text and these pages.",
    blocks: [
      {
        t: "table",
        head: ["Term", "Definition", "Not"],
        rows: [
          [
            "**Entry**",
            "One recorded clipboard value (MVP: text) with id, timestamp, content hash, size and pinned flag. Never holds the source application.",
            "item, clip, record, snippet",
          ],
          [
            "**History**",
            "The ordered, capped set of entries, most recent first.",
            "log, list, archive",
          ],
          [
            "**Clipboard**",
            "The selection set by Ctrl+C / Ctrl+V (`CLIPBOARD` on X11, the regular selection on Wayland). The only selection recorded in the MVP.",
            "buffer, pasteboard",
          ],
          [
            "**Primary selection**",
            "The middle-click selection. Not recorded in the MVP (Later, opt-in).",
            "X selection, highlight buffer",
          ],
          [
            "**Selection**",
            "Umbrella term for the clipboard and the primary selection, used only where both are meant.",
            "—",
          ],
          [
            "**Capture**",
            "The daemon noticing a clipboard change and deciding whether to record it.",
            "grab, sniff",
          ],
          [
            "**Restore**",
            "Putting an entry back on the clipboard so the user can paste it. rstory never pastes.",
            "paste, copy back, apply",
          ],
          [
            "**Backend**",
            "The code that talks to the session's display server: `wayland` or `x11` (a `portal` backend is Later).",
            "driver, provider, adapter",
          ],
          [
            "**XWayland bridge**",
            "The compositor's mirroring of the Wayland clipboard to XWayland's X11 selections and back. rstory's `x11` backend uses it on Wayland sessions without data-control, such as GNOME.",
            "X11 fallback, compat mode",
          ],
          [
            "**Session**",
            "The graphical login the daemon runs in, either Wayland or X11.",
            "desktop, display",
          ],
          [
            "**Data-control**",
            "The Wayland protocols that let a background client watch and set the clipboard: `ext-data-control-v1`, falling back to `zwlr_data_control_manager_v1`.",
            "clipboard protocol",
          ],
          [
            "**Daemon**",
            "The long-running `rstory daemon` process that captures, holds all state and owns the clipboard while serving a restore.",
            "server, service, agent, background process",
          ],
          [
            "**Client**",
            "Any thin front end talking to the daemon: CLI, picker, tray, panel icon.",
            "frontend, UI process",
          ],
          [
            "**Control socket**",
            "The per-user Unix socket in `$XDG_RUNTIME_DIR` that clients use to reach the daemon.",
            "IPC pipe, API",
          ],
          [
            "**Store**",
            "The private on-disk files under `$XDG_STATE_HOME/rstory` that hold the history.",
            "database, DB, cache",
          ],
          [
            "**Pinned entry**",
            "An entry kept regardless of expiry, eviction by the count cap, and `clear`. At most 10 by default (pin cap).",
            "favorite, starred, saved",
          ],
          [
            "**Pause**",
            "The state in which the daemon records nothing. Shown on the tray icon and by `status`.",
            "disable, mute, incognito",
          ],
          [
            "**Timed pause**",
            "A pause with an end time (`pause --for <duration>`), after which recording resumes on its own.",
            "snooze, temporary disable",
          ],
          [
            "**Sensitive hint**",
            "A signal offered by the copying application that the content must not be recorded (for example the `x-kde-passwordManagerHint` MIME type).",
            "secret flag, password flag",
          ],
          [
            "**Size cap**",
            "The 1 MiB per-entry maximum. Larger copies are dropped and counted, never truncated.",
            "limit, max size",
          ],
          [
            "**Count cap**",
            "The maximum number of entries in the history, pinned ones included (default 20). When it's exceeded, the oldest unpinned entry is deleted.",
            "history length, max entries",
          ],
          [
            "**Pin cap**",
            "The maximum number of pinned entries (default 10). Pinning past it is refused.",
            "max pins",
          ],
          [
            "**Expiry**",
            "Deletion of an unpinned entry 7 days after it was last copied.",
            "TTL (allowed only in config and code: `ttl`), retention",
          ],
          [
            "**Index**",
            "An entry's position in the history, `0` being the most recent. Changes as entries arrive.",
            "number, position",
          ],
          [
            "**Short id**",
            "A stable short prefix of an entry's id, usable wherever an index is.",
            "hash, key",
          ],
          [
            "**Picker**",
            "The built-in terminal UI opened by `rstory pick`.",
            "menu, launcher, selector, fzf",
          ],
          [
            "**Picker command**",
            "The `picker_command` config value used to open the picker in a terminal emulator.",
            "terminal command",
          ],
          [
            "**Tray**",
            "The StatusNotifierItem icon and menu shown by a tray host (KDE, waybar and similar).",
            "applet, indicator, systray",
          ],
          [
            "**Tray host**",
            "The bar or shell component that displays StatusNotifierItem icons.",
            "—",
          ],
          [
            "**GNOME extension**",
            "The small GNOME Shell extension embedded in the binary and installed by `rstory install-gnome-extension`.",
            "plugin, add-on, AppIndicator",
          ],
          [
            "**Panel icon**",
            "The rstory icon and menu that the GNOME extension draws in GNOME's top bar, which plays the tray's role on GNOME.",
            "tray (on GNOME)",
          ],
          ["**Doctor**", "`rstory doctor`, the diagnostic report.", "health check, debug"],
          [
            "**Autostart**",
            "Starting the daemon at login, through an XDG autostart entry or a systemd user unit.",
            "startup, service install",
          ],
          [
            "**Tier 1**",
            "An environment that must pass before every release (§11).",
            "supported (unqualified)",
          ],
        ],
      },
    ],
  },
};
