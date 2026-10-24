import type { PageContent } from "../../../content/types";

type G =
  | "guide-picker"
  | "guide-cli"
  | "guide-pause"
  | "guide-pins"
  | "guide-tray"
  | "guide-doctor";

export const guides: Record<G, PageContent> = {
  "guide-picker": {
    title: "Recall with the picker",
    summary: "Type to find a recent copy, preview it, and restore it with Enter.",
    blocks: [
      {
        t: "p",
        text: "The **picker** is a terminal UI opened by `rstory pick`. Type to filter the history, move with the arrow keys, and press Enter to restore the selected entry. The picker restores the entry to the clipboard; you then use your app's normal shortcut to insert it.",
      },
      {
        t: "code",
        lang: "sh",
        text: "rstory pick           # full-screen picker\nrstory pick --plain   # inline mode, for screen readers",
      },
      { t: "h2", text: "The keys" },
      {
        t: "table",
        head: ["Key", "Does"],
        rows: [
          ["Enter", "Restore the selected entry and exit"],
          ["Tab", "Toggle the preview"],
          ["Ctrl-T", "Pin or unpin the selected entry"],
          ["Ctrl-D", "Delete the selected entry, after a y/n question"],
          ["Esc", "Quit (exit code 1)"],
          ["F1", "Show the key map"],
        ],
      },
      {
        t: "p",
        text: "The full list, including movement and editing keys, is in [Picker keys](#/reference/keys).",
      },
      { t: "h2", text: "When the daemon is not running" },
      {
        t: "p",
        text: "The picker still opens, read-only, with the banner `Read-only: daemon not running`. Pressing Enter exits with code 3 and this message:",
      },
      {
        t: "code",
        lang: "text",
        text: "Daemon not running. Start it with `rstory daemon` or run `rstory install-autostart`.",
      },
      { t: "h2", text: "Open it from a menu" },
      {
        t: "p",
        text: "The tray and the GNOME panel icon have an **Open picker** item. The daemon opens the picker in a terminal emulator: `picker_command` from `config.toml` if set, otherwise the first terminal it finds, searching `$TERMINAL`, then `xdg-terminal-exec`, then `ghostty`, `kitty`, `alacritty`, `foot`, `wezterm`, `ptyxis`, `kgx`, `gnome-terminal`, `konsole`, `xfce4-terminal` and `xterm`. With no terminal, the menu item is hidden.",
      },
      {
        t: "code",
        lang: "toml",
        caption: "~/.config/rstory/config.toml",
        text: 'picker_command = "xterm -e rstory pick"',
      },
      { t: "demo", name: "picker-recall", title: "Find and restore a copy with rstory pick" },
    ],
  },

  "guide-cli": {
    title: "Script with the CLI",
    summary:
      "List, show, restore, pin and delete entries from the shell, with JSON output and stable exit codes.",
    blocks: [
      {
        t: "p",
        text: "Every action in the picker has a command. `ls` and `show` read the store directly, so they work while the daemon is down (the output then says `(daemon not running, read-only)`). The commands that change state (`copy`, `pin`, `unpin`, `rm`, `clear`, `pause`, `resume`) and `status` need the daemon, and exit with code 3 without it.",
      },
      {
        t: "code",
        lang: "sh",
        text: "rstory ls\nrstory ls --limit 5\nrstory show 0\nrstory show 01JB | wc -c\nrstory copy 1\nrstory pin 0\nrstory unpin 0\nrstory rm 2",
      },
      { t: "h2", text: "Refer to entries" },
      {
        t: "p",
        text: "A `REF` is an **index** (`0` is the newest) or a **short id** of at least 4 characters. Indexes change as entries arrive, so scripts that need a stable reference use short ids. A short id that matches several entries is refused with exit code 2.",
      },
      { t: "h2", text: "Machine-readable output" },
      {
        t: "p",
        text: "Piped, `ls` prints tab-separated columns: index, short id, pinned, age in seconds, preview. `--json` prints a stable shape with `schema: 1`.",
      },
      {
        t: "code",
        lang: "sh",
        text: "rstory ls --json | jq '.entries[].short_id'\nrstory show 1 --json\nrstory status --json | jq .entries",
      },
      {
        t: "p",
        text: 'Read-only listings add `read_only: true`, for example `{"entries":[],"read_only":true,"schema":1}`.',
      },
      { t: "h2", text: "Exit codes" },
      {
        t: "p",
        text: "Exit codes are stable: 2 for usage, not found or ambiguous, 3 for no daemon, 6 for the pin cap, 7 for no capture backend. The full table is in [Exit codes](#/reference/exit-codes).",
      },
      { t: "h2", text: "Clear the history" },
      {
        t: "p",
        text: "`rstory clear` deletes the unpinned entries after a question; `--all` also deletes pinned ones; `--yes` skips the question and is required without a terminal (without it, `rstory clear` refuses and tells you to add `--yes`).",
      },
      { t: "demo", name: "cli", title: "List, show and restore from the shell" },
    ],
  },

  "guide-pause": {
    title: "Pause and passc",
    summary:
      "Stop recording for a while, and keep tools that set no sensitive hint out of the history.",
    blocks: [
      {
        t: "p",
        text: "A **pause** makes the daemon record nothing. A **timed pause** ends by itself, between 1 second and 24 hours.",
      },
      {
        t: "code",
        lang: "sh",
        text: "rstory pause              # until resume\nrstory pause --for 50s    # 50s, 2m or 1h\nrstory status             # shows Paused, with the time left\nrstory resume",
      },
      {
        t: "p",
        text: "While paused, `rstory status` shows `Paused` (or `Paused — resumes in {m}:{ss}`), and the tray and panel icon change shape. Changes that happen during the pause are counted as `skipped_paused` and never read.",
      },
      { t: "h2", text: "The passc wrapper" },
      {
        t: "p",
        text: "The `pass` password manager sets no sensitive hint, so rstory would record what it copies. Pause around the copy. `pass -c` clears the clipboard after 45 seconds, so a 50 second pause covers it:",
      },
      {
        t: "code",
        lang: "sh",
        text: 'passc() { rstory pause --for 50s && pass -c "$@"; }',
      },
      {
        t: "callout",
        kind: "note",
        text: "The pause is checked before the sensitive hint. Pausing is the only protection for apps that offer no hint; see [Privacy and threat model](#/concepts/privacy).",
      },
      { t: "demo", name: "pause-passc", title: "Pause around a password copy" },
    ],
  },

  "guide-pins": {
    title: "Pins and clearing",
    summary: "Keep entries past expiry and clearing, and know the caps that apply.",
    blocks: [
      {
        t: "p",
        text: "A **pinned entry** is kept regardless of expiry, eviction by the count cap, and `clear`. Pin from the picker with Ctrl-T, or with `rstory pin <REF>` and `rstory unpin <REF>`.",
      },
      {
        t: "table",
        head: ["Cap", "Default", "Range", "Config key"],
        rows: [
          ["Count cap (pinned entries included)", "20", "2 to 500", "`max_entries`"],
          ["Pin cap", "10", "0 to `max_entries` - 1", "`max_pinned`"],
          ["Size cap per entry", "1 MiB", "1024 bytes to 1 MiB", "`max_entry_bytes`"],
          ["Expiry of unpinned entries", "7 days", "1h to 90d", "`ttl`"],
        ],
      },
      {
        t: "p",
        text: "Past the count cap, the oldest unpinned entry is deleted. Pinning past the pin cap is refused: `Pin limit reached ({max}). Unpin an entry first.`, and the CLI exits with code 6.",
      },
      { t: "h2", text: "Delete entries" },
      {
        t: "ul",
        items: [
          "One entry: `rstory rm <REF>`, or Ctrl-D in the picker.",
          "The unpinned entries: `rstory clear`, or Clear history… in the tray or panel icon (it asks first).",
          "Everything, pinned included: `rstory clear --all`.",
        ],
      },
    ],
  },

  "guide-tray": {
    title: "Tray and GNOME panel icon",
    summary: "Restore, pause and clear from the top bar.",
    blocks: [
      {
        t: "p",
        text: "On desktops with a StatusNotifierItem host (KDE, waybar and similar), the daemon shows a **tray** icon. GNOME has no tray, so rstory ships a **GNOME extension** that draws a **panel icon** with the same menu.",
      },
      { t: "h2", text: "The menu" },
      {
        t: "ul",
        items: [
          "The last 10 entries; choosing one restores it. With none, `No entries yet`.",
          "`Pause recording` or `Resume recording`, and `Pause for 1 minute`.",
          "`Open picker`, hidden when no terminal is found.",
          "`Clear history…`, which asks to `Delete {n} unpinned entries` or `Cancel`.",
        ],
      },
      {
        t: "p",
        text: "The icon shape changes with the state, so the state never depends on color alone: recording, paused and disconnected each have their own symbol. Without a daemon the menu reads `rstory daemon not running`.",
      },
      { t: "h2", text: "Tray details" },
      {
        t: "ul",
        items: [
          "With no tray host or no session bus, the daemon keeps recording; `rstory status` reports the tray as `no_host` or `no_bus`.",
          "`tray = false` in `config.toml` turns the daemon's tray off. `rstory tray` then runs the icon as a separate process.",
        ],
      },
      { t: "h2", text: "GNOME panel icon" },
      {
        t: "p",
        text: "Install it from the binary with `rstory install-gnome-extension --enable` (see [First run](#/first-run)). It talks to the daemon over the control socket and spawns no processes. GNOME Shell 48, 49 and 50 are supported. GNOME Shell 51 is [[unavailable]]: it is not declared until its manual check passes, and `rstory doctor` warns that the panel icon needs GNOME Shell 48 to 50.",
      },
      {
        t: "callout",
        kind: "planned",
        text: "Publishing the extension on extensions.gnome.org is planned for after 0.1.0. A popup list of entries on GNOME is on the [Roadmap](#/project/roadmap).",
      },
    ],
  },

  "guide-doctor": {
    title: "Troubleshoot with doctor",
    summary: "One read-only command that checks the setup and explains each problem it can detect.",
    blocks: [
      {
        t: "code",
        lang: "sh",
        text: "rstory doctor\nrstory doctor --json | jq '.checks[] | select(.status != \"ok\")'",
      },
      {
        t: "p",
        text: "`doctor` changes nothing. Each check has a status (`ok`, `info`, `warn` or `fail`), a message and, when something is wrong, a hint. It never prints clipboard content. When everything works it prints `Everything looks good.`",
      },
      {
        t: "p",
        text: "Exit code: 0 when everything works, 1 for warnings, 2 for failures or a usage error. The checks and their hints are in [Doctor checks](#/reference/doctor).",
      },
      { t: "h2", text: "Common cases" },
      {
        t: "ul",
        items: [
          "`No clipboard access in this session.` (exit 7, with a pointer to `rstory doctor`): the session has no Wayland data-control and no XWayland. See [Your desktop](#/desktops).",
          "`Daemon not running.` (exit 3): start it with `rstory daemon` or run `rstory install-autostart`.",
          "`The daemon did not answer in time.` followed by a pointer to `rstory doctor`: the daemon is stuck or still starting.",
          "Exit code 4: a daemon is already running (`already running (pid {pid})`).",
        ],
      },
      { t: "demo", name: "doctor", title: "Run doctor on a working session" },
    ],
  },
};
