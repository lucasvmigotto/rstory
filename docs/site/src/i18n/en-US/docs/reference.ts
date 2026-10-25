import type { PageContent } from "../../../content/types";

type R =
  | "ref-cli"
  | "ref-config"
  | "ref-env"
  | "ref-exit-codes"
  | "ref-files"
  | "ref-keys"
  | "ref-doctor"
  | "ref-protocol";

export const reference: Record<R, PageContent> = {
  "ref-cli": {
    title: "CLI reference",
    summary: "Every command, as the built binary prints it.",
    blocks: [
      {
        t: "p",
        text: "Everything below is generated from the real `--help` output of the built binary. Nothing is written by hand, and CI fails when it is out of date. `REF` is an index (`0` is the newest) or a short id of at least 4 characters.",
      },
      {
        t: "p",
        text: "`--ascii` is a global flag: it forces ASCII glyphs, as a non-UTF-8 locale does. Shell completions are available for `bash`, `zsh` and `fish`.",
      },
      { t: "cli" },
    ],
  },

  "ref-config": {
    title: "Configuration",
    summary: "The keys of config.toml, their defaults and their ranges.",
    blocks: [
      {
        t: "p",
        text: "The file is `$XDG_CONFIG_HOME/rstory/config.toml`, falling back to `~/.config/rstory/config.toml`. It is optional. `rstory config` prints the effective values and `rstory config --path` prints the path. Unknown keys are ignored.",
      },
      {
        t: "table",
        head: ["Key", "Default", "Allowed", "Meaning"],
        rows: [
          ["`max_entries`", "`20`", "2 to 500", "Count cap, pinned entries included."],
          ["`max_pinned`", "`10`", "0 to `max_entries` - 1", "Pin cap."],
          ["`max_entry_bytes`", "`1048576`", "1024 to 1048576", "Size cap per entry, in bytes."],
          [
            "`ttl`",
            '`"7d"`',
            "`1h` to `90d`, units `s`, `m`, `h`, `d`",
            "Expiry of unpinned entries.",
          ],
          [
            "`picker_command`",
            "unset",
            "any command; blank counts as unset",
            "Command that opens the picker in a terminal (tray, panel icon).",
          ],
          ["`tray`", "`true`", "`true` or `false`", "Show the tray icon from the daemon."],
          [
            "`theme`",
            '`"terminal"`',
            "`terminal`, `rstory` or `mono`",
            "Color of the CLI and picker output.",
          ],
          [
            "`backend`",
            "unset",
            "`x11` or `wayland`",
            "Force the capture backend. The `--backend` flag wins.",
          ],
        ],
      },
      { t: "h2", text: "Defaults, as printed by rstory config" },
      { t: "configdefaults" },
      { t: "h2", text: "Theme" },
      {
        t: "ul",
        items: [
          "`terminal` (default): your terminal's own 16-color palette.",
          "`rstory`: rstory's palette. Truecolor when `COLORTERM` is `truecolor` or `24bit`, else 256 colors when `TERM` names them, else 16 colors.",
          "`mono`: no color.",
        ],
      },
      {
        t: "p",
        text: "`NO_COLOR` (non-empty) and `TERM=dumb` always give no color. State is never carried by color alone: it also has a word or a glyph.",
      },
      { t: "h2", text: "Errors" },
      {
        t: "p",
        text: "A bad file exits with code 5 and names the key:",
      },
      {
        t: "code",
        lang: "text",
        text: "config.toml: …\n{key} = {value} is out of range; allowed: {allowed}\n{key} = {value:?} is invalid: {reason}",
      },
      {
        t: "code",
        lang: "toml",
        caption: "Example config.toml",
        text: 'max_entries = 50\nttl = "14d"\ntheme = "rstory"\npicker_command = "xterm -e rstory pick"',
      },
    ],
  },

  "ref-env": {
    title: "Environment variables",
    summary: "The variables rstory reads.",
    blocks: [
      {
        t: "p",
        text: "rstory has no variable of its own that switches a behavior on. It reads the standard ones below.",
      },
      {
        t: "table",
        head: ["Variable", "Used for"],
        rows: [
          [
            "`XDG_RUNTIME_DIR`",
            "The control socket, `<dir>/rstory/control.sock`. Falls back to `/run/user/<uid>`.",
          ],
          ["`XDG_STATE_HOME`", "The store, `<dir>/rstory`."],
          ["`XDG_CONFIG_HOME`", "`config.toml`, the autostart entry and the systemd unit."],
          ["`XDG_DATA_HOME`", "The GNOME extension directory."],
          ["`HOME`", "Fallback for all XDG directories, and `~/.Xauthority`."],
          ["`WAYLAND_DISPLAY`", "Session detection."],
          ["`DISPLAY`", "Session detection and the XWayland bridge."],
          ["`XDG_SESSION_TYPE`", "Session detection."],
          ["`XDG_CURRENT_DESKTOP`", "The warning when the desktop is not GNOME, and `doctor`."],
          ["`XAUTHORITY`", "X11 authentication."],
          [
            "`TERMINAL`",
            "First in the search for a terminal that opens the picker. A lookup input, not a setting rstory owns.",
          ],
          ["`PATH`", "Terminal lookup."],
          ["`NO_COLOR`", "When non-empty, no color."],
          [
            "`TERM`",
            '`dumb` gives no color; a name with `256color` allows 256 colors under `theme = "rstory"`.',
          ],
          ["`COLORTERM`", '`truecolor` or `24bit` allows truecolor under `theme = "rstory"`.'],
          ["`LC_ALL`, `LC_CTYPE`, `LANG`", "A non-UTF-8 locale switches output to ASCII glyphs."],
          ["`COLUMNS`", "CLI width when the terminal size is unknown."],
          ["`RUST_LOG`", "The daemon's log filter, default `info`."],
        ],
      },
    ],
  },

  "ref-exit-codes": {
    title: "Exit codes",
    summary: "Stable within a major version.",
    blocks: [
      {
        t: "table",
        head: ["Code", "Meaning"],
        rows: [
          ["`0`", "ok"],
          ["`1`", 'unexpected error. The picker also uses it for "dismissed" (Esc).'],
          [
            "`2`",
            "usage error, entry not found, ambiguous short id, or no terminal where one is needed",
          ],
          [
            "`3`",
            "the daemon is not running (CLI commands that change state, picker restore, `rstory tray`)",
          ],
          ["`4`", "the daemon is already running"],
          [
            "`5`",
            "environment error: runtime directory, store, config, or no systemd user session",
          ],
          ["`6`", "limit reached (pin cap)"],
          ["`7`", "inactive: there is no capture backend"],
          ["`130`", "interrupted (Ctrl-C)"],
        ],
      },
      { t: "h2", text: "doctor" },
      {
        t: "table",
        head: ["Code", "Meaning"],
        rows: [
          ["`0`", "everything works"],
          ["`1`", "warnings"],
          ["`2`", "failures, or a usage error"],
        ],
      },
    ],
  },

  "ref-files": {
    title: "Files and paths",
    summary: "Where rstory reads and writes.",
    blocks: [
      {
        t: "table",
        head: ["Path", "What"],
        rows: [
          [
            "`$XDG_CONFIG_HOME/rstory/config.toml`",
            "Configuration (`~/.config/rstory/config.toml`).",
          ],
          [
            "`$XDG_STATE_HOME/rstory/`",
            "The store (`~/.local/state/rstory/`): `index.json`, `store.lock` and `content/`. Mode `0700`; files `0600`.",
          ],
          [
            "`$XDG_RUNTIME_DIR/rstory/control.sock`",
            "The control socket (`/run/user/<uid>/…` fallback). Directory mode `0700`.",
          ],
          [
            "`$XDG_CONFIG_HOME/autostart/rstory.desktop`",
            "The autostart entry, from `rstory install-autostart`.",
          ],
          [
            "`$XDG_CONFIG_HOME/systemd/user/rstory.service`",
            "The systemd user unit, from `rstory install-autostart --systemd`.",
          ],
          [
            "`$XDG_DATA_HOME/gnome-shell/extensions/rstory@lucasvmigotto.me/`",
            "The GNOME extension, from `rstory install-gnome-extension`.",
          ],
        ],
      },
    ],
  },

  "ref-keys": {
    title: "Picker keys",
    summary: "Every key of rstory pick.",
    blocks: [
      {
        t: "table",
        head: ["Keys", "Does"],
        rows: [
          ["Enter", "Restore the selected entry and exit"],
          ["Tab", "Toggle the preview"],
          ["Ctrl-T", "Pin or unpin"],
          ["Ctrl-D", "Delete, then `y` or `n`"],
          ["Esc", "Quit (exit code 1)"],
          ["Ctrl-C", "Interrupt (exit code 130)"],
          ["F1", "Key map"],
          ["Up, Down, Ctrl-P, Ctrl-N", "Move the selection"],
          ["PageUp, PageDown, Home, End", "Move further"],
          ["Backspace, Ctrl-U, Ctrl-W", "Edit the query: one character, the whole query, one word"],
        ],
      },
      { t: "h2", text: "Texts" },
      {
        t: "code",
        lang: "text",
        text: "Enter restore  Tab preview  Ctrl-T pin  Ctrl-D delete  Esc quit\nDelete this entry? y/n\nPin limit reached ({max}). Unpin an entry first.\nRestoring…\nTerminal too small (need 20×5).\nNo entries yet. Copy something and it will appear here.\nRead-only: daemon not running",
      },
      {
        t: "p",
        text: 'The picker updates live while the daemon records. Below 20 columns by 5 rows it shows the "too small" message. `--plain` selects the inline mode.',
      },
    ],
  },

  "ref-doctor": {
    title: "Doctor checks",
    summary: "The checks of rstory doctor, in order.",
    blocks: [
      {
        t: "p",
        text: 'In order: `session`, `backend`, `store`, `daemon`, `pause`, `tray`, `gnome_shell`, `gnome_extension`, `picker`, `autostart`. Some only appear when they apply (the GNOME checks, for example). The JSON report is `{"schema":1,"checks":[{"id","status","message","hint"}]}` with status `ok`, `info`, `warn` or `fail`.',
      },
      { t: "h2", text: "What it prints with no graphical session" },
      { t: "doctor" },
      {
        t: "p",
        text: "The table above is the real report of the built binary in an empty environment. On a working desktop the same checks pass; other checks (`pause`, `tray`, the GNOME ones) report on your session.",
      },
    ],
  },

  "ref-protocol": {
    title: "Control protocol v1",
    summary: "The newline-delimited JSON protocol that every client uses.",
    blocks: [
      {
        t: "p",
        text: "The CLI, picker, tray and GNOME extension all speak this protocol to the daemon. The machine-readable schema is [protocol.schema.json](src:contracts/protocol.schema.json) and the contract is [protocol.md](src:contracts/protocol.md); contract tests cover it. A change to either is a spec change, and a breaking one bumps `v`.",
      },
      { t: "h2", text: "Transport" },
      {
        t: "ul",
        items: [
          "A Unix stream socket at `$XDG_RUNTIME_DIR/rstory/control.sock`.",
          "The daemon checks the peer's user id; another uid is closed without a byte.",
          "One JSON object per line, UTF-8, at most 2 MiB. A longer line gets `too_large`, then the connection closes.",
          "Several requests per connection. Responses carry the request `id`; events carry none.",
          "Unknown fields are ignored. A `v` other than 1 gets `unsupported_version`.",
        ],
      },
      {
        t: "code",
        lang: "jsonl",
        text: '{"v":1,"id":7,"op":"list","args":{}}\n{"v":1,"id":7,"ok":true,"result":{}}\n{"v":1,"id":7,"ok":false,"error":{"code":"not_found","message":"no entry ab12"}}\n{"v":1,"event":"history_changed","data":{"entries":20,"top":"01JB…"}}',
      },
      { t: "h2", text: "Operations" },
      {
        t: "table",
        head: ["op", "args", "errors"],
        rows: [
          ["`list`", "`limit` (1 to 500)", "none"],
          ["`show`", "`ref`", "`not_found`, `ambiguous`"],
          ["`restore`", "`ref`", "`not_found`, `ambiguous`, `inactive`, `backend_error`"],
          ["`pin`", "`ref`", "`not_found`, `ambiguous`, `pin_limit`"],
          ["`unpin`", "`ref`", "`not_found`, `ambiguous`"],
          ["`rm`", "`ref`", "`not_found`, `ambiguous`"],
          ["`clear`", "`all` (default false)", "none"],
          ["`pause`", "`for` (`50s`, `2m`, `1h`; 1s to 24h)", "`bad_request`"],
          ["`resume`", "none", "none"],
          ["`status`", "none", "none"],
          ["`open_picker`", "none", "`no_terminal`, `picker_command_failed`"],
          ["`subscribe`", "`events`: `history_changed`, `pause_changed`", "none"],
        ],
      },
      { t: "h2", text: "Status" },
      {
        t: "p",
        text: "`status` returns `state` (`recording`, `paused` or `inactive`), `paused_until`, `reason`, `backend` (`wayland`, `x11` or `none`), `mode` (`native`, `xwayland_bridge` or null), `entries`, `pinned`, `max_entries`, `max_pinned`, `counters` (`skipped_sensitive`, `skipped_oversize`, `skipped_paused`, `dropped_burst`), `tray` (`active`, `no_host`, `no_bus` or `disabled`), `picker_available`, `version` and `protocol`.",
      },
      { t: "h2", text: "Errors" },
      {
        t: "table",
        head: ["Code", "CLI exit"],
        rows: [
          ["`bad_request`", "2"],
          ["`too_large`", "1"],
          ["`unsupported_version`", "1"],
          ["`internal`", "1"],
          ["`not_found`", "2"],
          ["`ambiguous`", "2"],
          ["`pin_limit`", "6"],
          ["`inactive`", "7"],
          ["`backend_error`", "1"],
          ["`no_terminal`", "none (1)"],
          ["`picker_command_failed`", "none (1)"],
        ],
      },
      { t: "h2", text: "Guarantees" },
      {
        t: "ul",
        items: [
          "Content appears only in `show` results, never in events.",
          "A `restore` response is sent only after the backend confirmed ownership of the clipboard.",
          "Every change to the history emits exactly one `history_changed` event.",
          "Requests are applied in arrival order on one event loop. An index resolves at the moment of processing, so scripts that need stability use short ids.",
        ],
      },
    ],
  },
};
