# rstory — UX and language vision

Status: Draft
Last updated: 2026-10-21
Owner stage: `frontend:uiux`

## Summary

rstory has no window. Its interface is four small surfaces that each
appear for a few seconds:
- the **picker**, a terminal UI opened from a keybinding;
- the **CLI**, for scripts and quick commands;
- the **tray** menu, on KDE and SNI bars;
- the **panel icon** menu, on GNOME.

The design goal is **zero friction for recall**: from keybinding to "on
the clipboard" in under two seconds and three keystrokes. The surfaces
must also be **impossible to misread** about whether rstory is recording.
The voice is terse, exact and calm, like a good Unix tool that respects
your clipboard's privacy.

## Inputs used

- Used: `docs/product/brief.md` (audiences, journeys K1–K9, glossary),
  `docs/product/domain-model.md`, `specs/006–012` (CLI, picker, launch,
  tray, extension, doctor, autostart), `contracts/cli.md`,
  `specs/009-tray/contracts/menu.md`.
- Missing: no reference folder and no existing UI. The interface
  language is **English** (owner decision 2026-10-21).
- The intake filter isn't needed here: all inputs are pipeline artifacts.

## Reference analysis

There are no user-provided references. The genre conventions considered
are below (taste evidence, not templates):

| Reference | What to take | What to avoid | Why |
|---|---|---|---|
| fzf-style finders | type-to-filter immediately, matched characters highlighted, Enter acts and exits | a cramped single-line UI with no preview; color-only match marks | the primary persona already has this muscle memory |
| Unix CLIs (clig.dev) | quiet success, stderr diagnostics, `--json`, stable exit codes | chatty "Success!" output, emojis in machine paths | scripts and pipes |
| Desktop tray menus | short text items, a status line at the top of the actions, destructive item last with a confirm | icon-only items, nested menus deeper than one level | J7, accessibility |

## Personas & jobs

1. **Terminal power user** (primary, decides trade-offs). Lives in a
   terminal, has `Super+V` bound to the picker, and wants the old copy
   back *now*. Leaves if the picker is slower than their thought, or if
   it hijacks their usual readline keys.
   Jobs: J1 recall, J3 script, J4 pin, J6b pause around `pass`.
2. **Tray user.** On GNOME or KDE, clicks the top-bar icon. Leaves if the
   menu is cluttered or if they can't tell whether recording is paused.
   Jobs: J7, J6.
3. **Dotfiles maintainer.** Runs `install-autostart` and `doctor` once
   per machine. Leaves if setup messages are vague. Jobs: J8, J9.

## Principles

1. **Type first, act on Enter.** The picker accepts the query from the
   first keystroke. Every key that isn't text is a modifier or a named
   key.
2. **Recording state is never ambiguous.** Every surface (icon shape,
   menu status line, `status`, picker footer) says *Recording*, *Paused*
   or *Inactive* in words.
3. **Show the copy, not the chrome.** Previews get the width. Metadata
   (age, pin) is a narrow gutter.
4. **Quiet success, precise failure.** Success is silent or one short
   line. A failure names what happened and the exact command that fixes
   it.
5. **Never show more of a secret than needed.** Previews are one line and
   escaped. Nothing is echoed to logs.

## Information architecture

```
rstory
├── Picker (rstory pick)                 — find · preview · restore · pin · delete
├── CLI
│   ├── History: ls · show · copy · pin · unpin · rm · clear [--all]
│   ├── Recording: pause [--for] · resume · status
│   ├── Setup: install-autostart · install-gnome-extension · completions · config
│   └── Diagnostics: doctor
├── Tray (SNI) / Panel icon (GNOME)      — last 10 · status · pause/resume · pause 1 min · open picker · clear…
└── Daemon (rstory daemon)               — no UI; reports through status, doctor, icons
```

Navigation model: no navigation. Each surface is single-screen. The
picker has one screen with modal sub-states (confirm delete) shown in the
footer.

## Key flows

**F1 — Recall from the keybinding (J1, K2).** `Super+V` → a terminal
opens with the picker, the newest entry selected → type `reb` (the list
narrows live) → `Enter` → the picker closes → `Ctrl+V` in the target app.
Three keystrokes plus Enter. *Failure branches:* daemon down → the footer
shows *Read-only: daemon not running* and Enter shows the fix line,
exit 3. Empty history → "No entries yet. Copy something and it will
appear here." *Removable steps:* none.

**F2 — Restore from the top bar (J7, K7).** Click the icon → click an
entry. Two clicks. *Failure:* inactive → the status line says *Inactive —
no clipboard access*, and entry clicks show nothing new (the daemon
returns `inactive`).

**F3 — Pause around a secret (J6, J6b, K3b).** `passc github` (the
README function) or the menu item *Pause for 1 minute*. Feedback: the
icon switches to the paused shape, and the status line shows *Paused —
resumes in 0:59*.

**F4 — First setup (J8, J9, K1, K8).** `rstory install-autostart` →
`rstory install-gnome-extension --enable` (GNOME) → log out and in →
`rstory doctor`, all ✓. Each command ends with its single next step.

## Screen inventory & wireframes

| Screen | Purpose | Primary action | Content priority | Persona |
|---|---|---|---|---|
| Picker | find and restore an entry | Enter (restore) | query › list › preview › footer hints | 1 |
| Picker — confirm delete | avoid accidental removal | `y` | footer question | 1 |
| Picker — read-only | browse while the daemon is down | Esc | banner › list | 1 |
| Picker — empty / too small | explain and exit | Esc | message | 1 |
| CLI `ls` / `status` / `doctor` outputs | scan state | — | one line per item | 1, 3 |
| Tray / panel menu | restore, pause, clear | an entry item | entries › status › actions › clear | 2 |

**Picker at 80×24** (the terminal window opened by the keybinding):

```
 > reb_                                                      2/14  Recording
 ──────────────────────────────────────────────────────────────────────────────
 > 3m      git REBase -i HEAD~3              │ git rebase -i HEAD~3
   2h  📌  git REset --soft HEAD~1 && git…   │
                                             │
                                             │
 ...                                         │
 ──────────────────────────────────────────────────────────────────────────────
 Enter restore  Tab preview  Ctrl-T pin  Ctrl-D delete  Esc quit
```

**Picker at 120×40**: the same regions. The preview pane takes 50% of the
width and wraps the full content. The list shows up to 34 rows.

**Picker at 40×10** (preview hidden, toggled with Tab):

```
 > _                       14  Rec
 ───────────────────────────────────
 > 3m     https://github.com/lucas…
   2h 📌  git reset --soft HEAD~1…
   1d     cargo test -p rstory --…
 ───────────────────────────────────
 Enter restore  Tab preview  Esc quit
```

**Plain mode** (`--plain`, inline, ASCII, for screen readers and simple
terminals):

```
query: reb
> [0] 3m  git rebase -i HEAD~3
  [1] 2h  [pinned] git reset --soft HEAD~1 && git push --force-with-lease
2 of 14 shown. Enter restores, Esc quits.
```

**Tray / panel menu** (order fixed by `specs/009-tray/contracts/menu.md`):

```
┌──────────────────────────────────────────────┐
│ https://github.com/lucasvmigotto/rstory      │
│ 📌 git reset --soft HEAD~1 && git push --f…  │
│ cargo test -p rstory --test daemon_restore   │
│ … (up to 10)                                 │
├──────────────────────────────────────────────┤
│ Recording                          (inert)   │
│ Pause recording                              │
│ Pause for 1 minute                           │
│ Open picker                                  │
├──────────────────────────────────────────────┤
│ Clear history…                               │
└──────────────────────────────────────────────┘
```

## Visual direction & draft tokens

**Subject matter.** A clipboard is a stack of slips. The icon is a
rounded clipboard outline with **three stacked lines**: history, not a
single paste.
- **Paused**: the same clipboard with a **pause bar pair** replacing the
  lines. The silhouette changes, not only the color.
- **Disconnected** (extension only): the clipboard with a **diagonal
  slash**.
- The icons are monochrome symbolic (they follow panel foreground color
  on GNOME, via `-symbolic` SVG). For the SNI tray, pixmaps come in light
  and dark variants.

**Terminal colors.** rstory inherits the user's terminal theme. It uses
only semantic roles mapped onto the 16 ANSI colors by default. That
respects the user's palette, and the primary persona has tuned it.

| Token | 16-color tier | 256 tier | Truecolor tier | Monochrome |
|---|---|---|---|---|
| `color.accent` (query cursor, match marks) | ANSI cyan | 37 | #2AA198 | bold + underline |
| `color.selected` (selected row) | reverse video | reverse | reverse | reverse video |
| `color.muted` (age, counters, hints) | ANSI bright black | 245 | #8A8A8A | normal (no dim) |
| `color.pinned` (pin glyph) | ANSI yellow | 178 | #D7AF00 | glyph only |
| `color.warning` (read-only, paused) | ANSI yellow | 178 | #D7AF00 | the word "Paused" / "Read-only" |
| `color.danger` (delete confirm) | ANSI red | 160 | #D70000 | the word "Delete?" |

Rules: truecolor is used only when `COLORTERM=truecolor` *and* a config
`theme = "rstory"` opts in (default `theme = "terminal"` = the 16-color
tier). `NO_COLOR` and `TERM=dumb` → monochrome. Meaning never depends on
color alone: selected = `>` + reverse, pinned = 📌/`[pinned]`, match =
bold/underline, state = words.

**Glyphs** (UTF-8 → ASCII fallback with `--ascii` or a non-UTF-8 locale):
`📌`→`[p]`, `│`→`|`, `─`→`-`, `…`→`...`, `⏎`→`\n`, `✓`→`ok`, `!`→`warn`,
`✗`→`FAIL`.

**Spacing.** One-cell gutters. The age column is right-aligned to 3–4
cells (`3m`, `2h`, `5d`). The pin column is 2 cells.

**Motion.** None. The list redraws on change. No spinners: restore waits
for ownership, which takes at most a few hundred ms, and above 300 ms the
footer shows "Restoring…".

**Review against AI defaults.** I rejected a custom brand palette
painted over the user's terminal (generic "dashboard" look; clashes with
their theme), rounded box-drawing panels around every region (wasted
cells at 40 cols), and an emoji per menu item (noise and screen-reader
chatter). Kept: one accent role and a shape-distinct paused icon.

## Interaction patterns & states

- **Feedback:** silent success (picker exits; CLI prints nothing).
  Errors appear inline in the picker footer, on stderr for the CLI, and
  in a menu status line for the tray.
- **Destructive actions:** picker `Ctrl-D` asks *Delete this entry?
  y/n* in the footer. CLI `clear` asks with the count, and `clear --all`
  names the pinned count. The menus use a confirm submenu. There's no
  undo (deletion is real by design, BR10), so every confirm states the
  count.
- **Loading:** the picker loads before its first frame (< 100 ms), so
  there's no loading state. A restore above 300 ms shows "Restoring…".
- **Empty:** the picker shows "No entries yet. Copy something and it will
  appear here." The menu shows the inert item "No entries yet".
- **Errors:** the daemon is down, the backend is inactive, the pin limit
  is reached, or the terminal is too small. Copy below.
- **Live updates:** a picker that's open follows `history_changed`, and
  the selection sticks to the same entry id.

## Responsive behavior

| Width | Picker layout |
|---|---|
| ≥ 100 cols | list + preview side by side (preview 50%) |
| 60–99 | list + preview (preview 40%) |
| 40–59 | list only; Tab toggles a full-width preview |
| < 40 or < 8 rows | list only, the hints line shortened; < 20×5: "Terminal too small (need 20×5)." |

Menus: the host decides width. Labels are capped at 48 chars.

## Accessibility targets

- WCAG 2.2 AA principles applied to terminal and menu surfaces
  (Constitution VII):
  - every action works by keyboard (picker) or by menu keyboard
    navigation (hosts);
  - non-color state cues everywhere;
  - `--plain` line mode for screen readers;
  - the CLI is the fully accessible path to every action (parity table in
    `contracts/cli.md`).
- Contrast: the default inherits the user's terminal theme. In truecolor
  opt-in, the contrast pairs are: accent #2AA198 on #002B36 = 4.6:1 and
  on #FDF6E3 = 3.0:1 (used only with bold + underline, never as the only
  cue); muted #8A8A8A on #1C1C1C = 5.3:1, on #FFFFFF = 3.5:1 (hints only,
  never information-only text in light mode: the light theme uses #6C6C6C
  = 5.3:1).
- Keyboard paths: F1 (above) is fully keyboard. Menus use the hosts'
  keyboard navigation. The CLI covers F2–F4.

## Voice & tone

- **Voice:** terse, exact, calm. Present tense, active voice, glossary
  nouns. No exclamation marks, no apologies, no "Oops".
- **Tone by moment:**
  - *Success*: silent, or a fact ("Removed 12 entries.").
  - *Error*: what happened, then the command that fixes it.
  - *Destructive*: the count, then the irreversible fact.
  - *Setup*: the single next step.
- Do: "Daemon not running. Start it with `rstory daemon` or run `rstory
  install-autostart`."
- Don't: "Oops! Something went wrong connecting to the background
  service 😢".

## Terminology

The glossary terms appear verbatim in the UI: **entry**, **history**,
**pinned**, **pause/paused**, **restore** (used in help text; the CLI
verb is `copy`, as in the brief), **daemon**, **picker**, **tray**,
**panel icon**, **GNOME extension**, **doctor**.

UI-only verbs: *Pause recording*, *Resume recording*, *Pause for
1 minute*, *Open picker*, *Clear history…*, *Delete*.

Avoid: clip, item, snippet, buffer, paste (rstory never pastes), service.

> Note: `copy <ref>` is the CLI verb for **restore**, kept from the brief
> because it matches the user's mental model ("copy entry 1 back").
> Help text says "Restore entry <ref> to the clipboard".

## Microcopy patterns

- Command help: one usage line, one sentence, 1–3 real examples.
- Errors: `<what happened>. <how to fix>.`, one line, with a backtick
  command when there is one.
- Counts: "1 entry" / "N entries"; durations "50s", "2m", "1h";
  relative ages "now", "3m", "2h", "5d".
- Menu labels: sentence case, verbs first, an ellipsis only when a
  confirm follows.

## Key-screen copy

| Key | Text |
|---|---|
| `picker.empty` | No entries yet. Copy something and it will appear here. |
| `picker.readonly.banner` | Read-only: daemon not running |
| `picker.readonly.restore` | Daemon not running. Start it with `rstory daemon` or run `rstory install-autostart`. |
| `picker.hints.full` | Enter restore  Tab preview  Ctrl-T pin  Ctrl-D delete  Esc quit |
| `picker.hints.short` | Enter restore  Tab preview  Esc quit |
| `picker.confirm.delete` | Delete this entry? y/n |
| `picker.pin.limit` | Pin limit reached (10). Unpin an entry first. |
| `picker.restoring` | Restoring… |
| `picker.too_small` | Terminal too small (need 20×5). |
| `picker.count` | {shown}/{total} |
| `state.recording` | Recording |
| `state.paused` | Paused |
| `state.paused_until` | Paused — resumes in {m}:{ss} |
| `state.inactive` | Inactive — {reason} |
| `menu.empty` | No entries yet |
| `menu.pause` / `menu.resume` | Pause recording / Resume recording |
| `menu.pause_minute` | Pause for 1 minute |
| `menu.open_picker` | Open picker |
| `menu.clear` | Clear history… |
| `menu.clear.confirm` | Delete {n} unpinned entries |
| `menu.clear.cancel` | Cancel |
| `menu.disconnected` | rstory daemon not running |
| `cli.daemon_down` | Daemon not running. Start it with `rstory daemon` or run `rstory install-autostart`. |
| `cli.not_found` | No entry {ref}. Run `rstory ls` to see indexes and short ids. |
| `cli.ambiguous` | Short id {ref} matches {n} entries: {ids}. Use more characters. |
| `cli.pin_limit` | Pin limit reached ({max}). Unpin an entry first. |
| `cli.clear.prompt` | Delete {n} unpinned entries? This can't be undone. [y/N] |
| `cli.clear_all.prompt` | Delete ALL {n} entries, including {m} pinned? This can't be undone. [y/N] |
| `cli.clear.done` | Removed {n} entries. |
| `cli.needs_yes` | Refusing to clear without a terminal. Add `--yes`. |
| `cli.inactive` | No clipboard access in this session. Run `rstory doctor` for details. |
| `cli.readonly` | (daemon not running, read-only) |
| `install.autostart.done` | Autostart installed: {path}. rstory starts at your next login, or run `rstory daemon` now. |
| `install.gnome.done` | GNOME extension installed. Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.github.io |
| `doctor.all_ok` | Everything looks good. |

The CLI and picker strings are Rust constants grouped by key (English
only, decision 2026-10-21). There are no locale files in v1. The keys
are kept so translation stays a data change later.

## Localization

- English only (v1). Ages and durations are locale-neutral short forms.
  `--json` timestamps are RFC 3339 UTC. No currency.
- Text expansion isn't planned. Labels are capped by width rules, not
  by language.

## Terminal interface

**Scope:** requested by the user (brief §6: "a built-in terminal UI").
The TUI is the **picker** (spec 007). The CLI is spec 006.

**Minimum size:** 20×5, with a "too small" message below that. Designed
for 80×24 and 120×40, and works at 40×10.

**Keymap (picker):**

| Key | Action | When |
|---|---|---|
| printable chars, Backspace, Ctrl-U, Ctrl-W | edit the query | always |
| Up / Down, Ctrl-P / Ctrl-N | move the selection | list not empty |
| PgUp / PgDn, Home / End | jump | list not empty |
| Enter | restore and exit | an entry is selected (not read-only) |
| Tab | toggle the preview | always |
| Ctrl-T | pin / unpin | not read-only |
| Ctrl-D | delete (asks y/n) | not read-only |
| F1 | show the full keymap in the footer | always |
| Esc | quit (exit 1), or cancel the confirm | always |
| Ctrl-C | quit (exit 130) | always |

**Deviation from the generic TUI keymap** (`terminal-design.md`): `q`,
`/`, `?`, `h j k l` and `:` aren't bound, because the picker filters as
you type, and those characters must reach the query (fzf convention,
principle 1). `?` help becomes `F1`. Recorded in the decision log.

**Focus:** a single focus (the list), with the query always active. The
selection is shown as `>` plus reverse video.

**Color tiers & themes:** see the visual direction above (16-color
default, truecolor opt-in, monochrome under `NO_COLOR`). High contrast =
monochrome tier with bold.

**ASCII fallback:** `--ascii`, or automatic when the locale isn't UTF-8.

**CLI command tree and `--json`:** `contracts/cli.md` (commands, schemas,
exit codes, parity table).

## Open questions

- None blocking. Brief Q7 (GNOME 49–51 bridge check) doesn't affect the
  UX.

## Decision log

| Date | Decision | Alternatives | Why |
|---|---|---|---|
| 2026-10-21 | Inherit the terminal's 16-color palette by default; truecolor theme opt-in | Fixed brand palette | Respects the primary persona's theme; contrast is the user's |
| 2026-10-21 | Type-to-filter with no `q`/`/`/`?`/`hjkl` bindings; F1 for help | Modal (normal/insert) picker | Fewer keystrokes for recall; fzf muscle memory |
| 2026-10-21 | Paused and disconnected icons change silhouette | Color-only state | Constitution VII; monochrome GNOME panels |
| 2026-10-21 | No spinner; "Restoring…" only above 300 ms | Spinner always | Restores are near-instant; avoid flicker |
| 2026-10-21 | The CLI verb stays `copy` for restore | `restore` | Brief and mental model; help text uses "restore" |
| 2026-10-21 | Strings as keyed Rust constants, no locale files in v1 | i18n catalogs | English-only decision |
