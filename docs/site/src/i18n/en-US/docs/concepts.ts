import type { PageContent } from "../../../content/types";

type C = "concept-history" | "concept-privacy" | "concept-daemon" | "concept-backends";

export const concepts: Record<C, PageContent> = {
  "concept-history": {
    title: "Entries and history",
    summary: "What rstory records, how many entries it keeps, and when they go away.",
    blocks: [
      {
        t: "p",
        text: "An **entry** is one recorded clipboard value. In this version an entry is text. It has an id, a timestamp, a content hash, a size and a pinned flag. It never records which application it came from.",
      },
      {
        t: "p",
        text: "The **history** is the ordered, capped set of entries, most recent first. Only the **clipboard** (the Ctrl+C selection) is recorded. The **primary selection** (middle-click) is not; recording it is on the [Roadmap](#/project/roadmap).",
      },
      { t: "h2", text: "Rules" },
      {
        t: "ul",
        items: [
          "**Duplicates.** The content hash identifies duplicates. Copying the same text again moves the existing entry to the top and refreshes its time; it never creates a second entry.",
          "**Count cap.** At most 20 entries, pinned ones included, by default. Past it, the oldest unpinned entry is deleted.",
          "**Expiry.** An unpinned entry expires 7 days after it was last copied, by default.",
          "**Size cap.** A copy over 1 MiB is dropped whole and counted as `skipped_oversize`. It is never truncated. Empty copies and copies with no text are ignored.",
          "**Pins.** A pinned entry is exempt from expiry, the count cap and `clear`; at most 10 by default.",
          "**Deletion.** Removing or clearing deletes the entry from disk, not only from view.",
          "**Order.** Entries are addressed by **index** (`0` is the newest) or **short id**.",
        ],
      },
      {
        t: "p",
        text: "The caps are configurable; see [Configuration](#/reference/config) and [Pins and clearing](#/guides/pins).",
      },
      { t: "h2", text: "Where the history lives" },
      {
        t: "p",
        text: "In the **store**: an `index.json`, a `store.lock` and a `content/` directory under `$XDG_STATE_HOME/rstory` (`~/.local/state/rstory`). Only the daemon writes to it. The directory is mode `0700` and the files `0600`, and a store that is open to others is refused. Recovery after a crash or power loss loses at most the entry being written.",
      },
      {
        t: "callout",
        kind: "note",
        text: "A restored value lives on the clipboard only while the daemon runs. When the daemon stops, both clipboard protocols drop the value; this is a limit of the protocols.",
      },
    ],
  },

  "concept-privacy": {
    title: "Privacy and threat model",
    summary: "What rstory protects, what it cannot, and how to cover the gap.",
    blocks: [
      {
        t: "p",
        text: "rstory keeps clipboard text in a private store on your machine and never makes a network connection. It sends no telemetry.",
      },
      { t: "h2", text: "What rstory does" },
      {
        t: "table",
        head: ["Protection", "How it works", "Status"],
        rows: [
          [
            "Sensitive hint",
            "When the app that owns a copy offers the type `x-kde-passwordManagerHint` (KeePassXC, KDE and other password managers set it), the copy is skipped without being read. The decision uses the offered type names only. Matching is exact and case-sensitive. Counted as `skipped_sensitive`.",
            "[[implemented]]",
          ],
          [
            "Size cap",
            "A copy over `max_entry_bytes` (1 MiB by default) is dropped whole and counted as `skipped_oversize`.",
            "[[implemented]]",
          ],
          [
            "Pause",
            "While paused nothing is read; changes are counted as `skipped_paused`. The pause is checked before the hint.",
            "[[implemented]]",
          ],
          [
            "No content in logs",
            "Decisions log their name and size, never content. Errors and `doctor` output carry no content either.",
            "[[implemented]]",
          ],
          [
            "Private files",
            "Store directory `0700`, files `0600`. The control socket accepts only your user id: a connection from another uid is closed without a byte.",
            "[[implemented]]",
          ],
          [
            "Apps that set no hint",
            "Another program running as you can read the clipboard while a copy is on it, and `pass` sets no hint. A secret copied from such an app is recorded like any other text.",
            "[[upstream]]",
          ],
        ],
      },
      { t: "h2", text: "Cover the gap" },
      {
        t: "p",
        text: "For tools that set no hint, pause recording around the copy. See [Pause and passc](#/guides/pause).",
      },
      { t: "code", lang: "sh", text: 'passc() { rstory pause --for 50s && pass -c "$@"; }' },
      {
        t: "p",
        text: "Encrypting the store is on the [Roadmap](#/project/roadmap). Today, rely on file permissions, the 7 day expiry and `rstory clear`.",
      },
      { t: "h2", text: "Threat model" },
      {
        t: "p",
        text: "rstory defends against a copy that an app marks as sensitive ending up in the history, and against other local users and casual disk access, through permissions and expiry. It cannot defend against another process running with your own rights, which can read the clipboard and the store anyway.",
      },
    ],
  },

  "concept-daemon": {
    title: "Daemon and control socket",
    summary: "One process owns the history; every front end is a thin client of it.",
    blocks: [
      {
        t: "p",
        text: "The **daemon** (`rstory daemon`) captures clipboard changes, holds all state, and owns the clipboard while it serves a restore. The **clients** (CLI, picker, tray, panel icon) hold no logic of their own. They talk to the daemon over the **control socket**.",
      },
      { t: "h2", text: "One writer" },
      {
        t: "p",
        text: "Only the daemon writes to the store, so there is no lock contention and no corruption from two writers. When the daemon is down, `ls`, `show` and the picker read the store read-only; commands that change state exit with code 3. A second daemon for the same user exits with code 4.",
      },
      { t: "h2", text: "The socket" },
      {
        t: "ul",
        items: [
          "Path: `$XDG_RUNTIME_DIR/rstory/control.sock`, in a directory of mode `0700`. Without `XDG_RUNTIME_DIR`, `/run/user/<uid>` is used when it is yours.",
          "Protocol: newline-delimited JSON, version 1. See [Control protocol v1](#/reference/protocol).",
          "Authentication: the peer's user id must match the daemon's.",
        ],
      },
      { t: "h2", text: "Running it" },
      {
        t: "p",
        text: "Run `rstory daemon` in a terminal, or start it at login with `rstory install-autostart`. It logs to stderr (the journal under systemd), filtered by `RUST_LOG`, default `info`. Logs never contain clipboard content. `rstory daemon --help` lists the one option, `--backend`.",
      },
    ],
  },

  "concept-backends": {
    title: "Backends and the XWayland bridge",
    summary: "How rstory reaches the clipboard on Wayland and X11 without you choosing.",
    blocks: [
      {
        t: "p",
        text: "A **backend** is the code that talks to the display server: `wayland` or `x11`. The daemon selects one from the **session** when it starts, in this order:",
      },
      {
        t: "ol",
        items: [
          "A `--backend` flag, or the `backend` key in `config.toml`, wins. The flag wins over the key.",
          "Wayland with **data-control** (`ext-data-control-v1`, falling back to `zwlr_data_control_manager_v1`): the `wayland` backend.",
          "Wayland without data-control but with XWayland, as on GNOME: the `x11` backend, through the **XWayland bridge**.",
          "An X11 session: the `x11` backend, natively.",
          "Wayland with neither data-control nor XWayland: no backend. The daemon is inactive, and restore commands exit with code 7.",
        ],
      },
      {
        t: "p",
        text: "The XWayland bridge is the compositor's mirroring of the Wayland clipboard to XWayland's X11 selections and back. GNOME's compositor offers no data-control, so this is the path GNOME users take. `rstory status` shows the backend and the mode (`native` or `xwayland_bridge`).",
      },
      {
        t: "callout",
        kind: "warning",
        text: "Wayland with neither data-control nor XWayland is an upstream limit: there is no way for a background client to watch the clipboard. See [Limitations](#/project/limitations).",
      },
    ],
  },
};
