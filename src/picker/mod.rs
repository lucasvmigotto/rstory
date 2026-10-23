//! Terminal picker UI: search, preview, actions (spec 007, ADR-0007).
//!
//! Layout: `state` is the pure core (no I/O); `view` draws it; `data` reads from the daemon
//! or the read-only store; `app` carries out the core's commands; `term` owns the terminal
//! lifecycle. Allowed dependencies (architecture 5.3, fitness F1): `ipc` (client), `store`
//! (read-only API), `config`, `domain` and the design system in `ui`; never `backend` or
//! `daemon`.

pub mod app;
pub mod data;
pub mod state;
pub mod term;
pub mod view;

#[cfg(test)]
mod tests;

use std::io::{IsTerminal, Write};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::{Terminal, TerminalOptions, Viewport, backend::CrosstermBackend};

use crate::ui::glyphs::{Glyphs, LocaleEnv, ascii_mode};
use crate::ui::theme::{Background, TermEnv, Theme, ThemePref};
use crate::ui::widgets::Ctx;

use self::app::{Debounce, execute};
use self::data::{Paths, Source};
use self::state::{Cmd, EXIT_NO_TTY, Exit, Msg, PickerState, update};

/// `rstory pick` usage.
pub const USAGE: &str = "Usage: rstory pick [--plain]";

/// The parsed `pick` options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    /// Inline, ASCII, no box drawing (screen readers, simple terminals).
    pub plain: bool,
}

/// Parses the arguments after `pick`. `Ok(None)` means `--help` was asked for.
///
/// # Errors
/// An unknown argument, with the usage line.
pub fn parse_args(args: &[String]) -> Result<Option<Options>, String> {
    let mut opts = Options::default();
    for a in args {
        match a.as_str() {
            "--plain" => opts.plain = true,
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("rstory pick: unknown argument {other}\n{USAGE}")),
        }
    }
    Ok(Some(opts))
}

/// Runs `rstory pick` and returns the exit status (FR-009).
#[must_use]
pub fn run(args: &[String], paths: &Paths) -> u8 {
    let opts = match parse_args(args) {
        Ok(Some(o)) => o,
        Ok(None) => {
            println!("Open the interactive picker\n\n{USAGE}");
            return 0;
        }
        Err(e) => {
            eprintln!("{e}");
            return EXIT_NO_TTY;
        }
    };
    if let Err(exit) = term::require_tty(std::io::stdout().is_terminal()) {
        return finish(&exit);
    }
    let exit = match interactive(opts, paths) {
        Ok(exit) => exit,
        Err(message) => Exit {
            code: state::EXIT_ENVIRONMENT,
            message: Some(message),
        },
    };
    finish(&exit)
}

fn finish(exit: &Exit) -> u8 {
    if let Some(m) = &exit.message {
        let _ = writeln!(std::io::stderr(), "{m}");
    }
    exit.code
}

/// Feeds `msg` and carries out the resulting commands. Preview fetches go through the
/// debounce unless `immediate`; actions run synchronously after a redraw.
fn feed(
    state: &mut PickerState,
    source: &mut dyn Source,
    debounce: &mut Debounce,
    msg: Msg,
    immediate: bool,
    redraw: &mut dyn FnMut(&PickerState),
) -> Option<Exit> {
    let mut queue = std::collections::VecDeque::from(update(state, msg));
    while let Some(cmd) = queue.pop_front() {
        match cmd {
            Cmd::Quit(exit) => return Some(exit),
            Cmd::Show(id) if !immediate => debounce.request(id, Instant::now()),
            cmd => {
                if !matches!(cmd, Cmd::Show(_) | Cmd::Reload) {
                    redraw(state); // e.g. "Restoring…" before the call blocks
                }
                if let Some(answer) = execute(&cmd, source) {
                    queue.extend(update(state, answer));
                }
            }
        }
    }
    None
}

fn interactive(opts: Options, paths: &Paths) -> Result<Exit, String> {
    // Load before the first draw (research R2): no empty frame, no flicker.
    let mut source = data::open(paths)?;
    let snapshot = source.list().map_err(|f| match f {
        state::Failure::Other(m) => m,
        state::Failure::DaemonDown => crate::ui::copy::CLI_DAEMON_DOWN.to_string(),
        state::Failure::PinLimit(_) => "unexpected answer".to_string(),
    })?;
    let size = crossterm::terminal::size().unwrap_or((80, 24));
    let read_only = source.read_only();
    let mut state = PickerState::new(snapshot, read_only, size);
    let mut debounce = Debounce::default();
    let mut nothing = |_: &PickerState| {};
    let first = Msg::Resize {
        width: size.0,
        height: size.1,
    };
    if let Some(exit) = feed(
        &mut state,
        &mut *source,
        &mut debounce,
        first,
        true,
        &mut nothing,
    ) {
        return Ok(exit);
    }

    let theme = Theme::detect(
        ThemePref::Terminal,
        &TermEnv::from_process(),
        Background::Dark,
    );
    let glyphs = Glyphs::new(opts.plain || ascii_mode(false, &LocaleEnv::from_process()));
    let ctx = Ctx::new(&theme, &glyphs);

    let (tx, rx) = mpsc::channel::<Msg>();
    if !read_only && let Some(socket) = &paths.socket {
        let tx = tx.clone();
        // Live updates are a nicety: without them the list is still correct when opened.
        let _ = data::subscribe(socket, move || tx.send(Msg::HistoryChanged).is_ok());
    }

    let session = term::Session::start(Box::new(term::CrosstermScreen {
        alternate: !opts.plain,
    }))
    .map_err(|e| format!("rstory pick: cannot set up the terminal: {e}"))?;
    let viewport = if opts.plain {
        Viewport::Inline(view::plain_height(size.1))
    } else {
        Viewport::Fullscreen
    };
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })
        .map_err(|e| format!("rstory pick: cannot set up the terminal: {e}"))?;

    let mut draw = |s: &PickerState| {
        let _ = terminal.draw(|f| {
            let area = f.area();
            if opts.plain {
                view::render_plain(s, area, f.buffer_mut(), ctx);
            } else {
                view::render(s, area, f.buffer_mut(), ctx);
            }
        });
    };
    let exit = loop {
        draw(&state);
        let now = Instant::now();
        if let Some(id) = debounce.due(now) {
            let msg = execute(&Cmd::Show(id), &mut *source);
            if let Some(msg) = msg
                && let Some(exit) = feed(
                    &mut state,
                    &mut *source,
                    &mut debounce,
                    msg,
                    false,
                    &mut draw,
                )
            {
                break exit;
            }
            continue;
        }
        let wait = debounce.wait(now).unwrap_or(Duration::from_millis(250));
        let mut inputs = Vec::new();
        match crossterm::event::poll(wait) {
            Ok(true) => match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(k)) => {
                    if let Some(key) = term::key_from(k) {
                        inputs.push(Msg::Key(key));
                    }
                }
                Ok(crossterm::event::Event::Resize(width, height)) => {
                    inputs.push(Msg::Resize { width, height });
                }
                Ok(_) => {}
                Err(_) => break Exit::code(state::EXIT_DISMISSED),
            },
            Ok(false) => {}
            Err(_) => break Exit::code(state::EXIT_DISMISSED),
        }
        inputs.extend(rx.try_iter());
        let mut done = None;
        for msg in inputs {
            if let Some(exit) = feed(
                &mut state,
                &mut *source,
                &mut debounce,
                msg,
                false,
                &mut draw,
            ) {
                done = Some(exit);
                break;
            }
        }
        if let Some(exit) = done {
            break exit;
        }
    };
    if opts.plain {
        // Leave the inline picker on its own line, below the drawn rows.
        let _ = terminal.clear();
    }
    drop(terminal);
    drop(session);
    Ok(exit)
}
