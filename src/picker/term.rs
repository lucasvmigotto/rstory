//! Terminal lifecycle (FR-008, FR-009): the TTY check, entering and leaving raw mode, and
//! restoring the terminal on every way out: return, error, panic, SIGTERM, SIGINT, SIGHUP.
//!
//! The one restore path is [`Restorer`]: a shared, idempotent handle to the active
//! [`Screen`]. The session guard's `Drop`, the panic hook and the signal thread all call
//! [`Restorer::restore`]; whichever runs first leaves the screen, the others do nothing.
//! [`Screen`] is a trait so tests drive the lifecycle with a fake and never touch a real
//! terminal.

use std::io;
use std::sync::{Arc, Mutex, PoisonError};

/// The stderr line when stdout is not a terminal (exit 2).
pub const NO_TTY_MESSAGE: &str = "rstory pick needs a terminal";

/// Fails with exit 2 when stdout is not a terminal.
///
/// # Errors
/// [`super::state::Exit`] with [`super::state::EXIT_NO_TTY`] and [`NO_TTY_MESSAGE`].
pub fn require_tty(stdout_is_tty: bool) -> Result<(), super::state::Exit> {
    if stdout_is_tty {
        Ok(())
    } else {
        Err(super::state::Exit {
            code: super::state::EXIT_NO_TTY,
            message: Some(NO_TTY_MESSAGE.to_string()),
        })
    }
}

/// What the picker does to a terminal, and how it undoes it.
pub trait Screen: Send {
    /// Raw mode on, cursor hidden, and the alternate screen when the screen uses one.
    ///
    /// # Errors
    /// The terminal refused a mode change.
    fn enter(&mut self) -> io::Result<()>;
    /// The exact reverse of [`Screen::enter`]: cursor shown, raw mode off.
    ///
    /// # Errors
    /// The terminal refused a mode change.
    fn leave(&mut self) -> io::Result<()>;
}

/// The real terminal through crossterm.
#[derive(Debug, Clone, Copy)]
pub struct CrosstermScreen {
    /// Full-screen mode uses the alternate screen; `--plain` draws inline.
    pub alternate: bool,
}

impl Screen for CrosstermScreen {
    fn enter(&mut self) -> io::Result<()> {
        use crossterm::{cursor::Hide, execute, terminal};
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        if self.alternate {
            execute!(out, terminal::EnterAlternateScreen, Hide)
        } else {
            execute!(out, Hide)
        }
    }

    fn leave(&mut self) -> io::Result<()> {
        use crossterm::{cursor::Show, execute, terminal};
        let mut out = io::stdout();
        // Every step runs even when an earlier one fails; the first error is reported.
        let screen = if self.alternate {
            execute!(out, terminal::LeaveAlternateScreen, Show)
        } else {
            execute!(out, Show)
        };
        let raw = terminal::disable_raw_mode();
        screen.and(raw)
    }
}

/// A shared, idempotent restore of the active screen.
#[derive(Clone)]
pub struct Restorer(Arc<Mutex<Option<Box<dyn Screen>>>>);

impl std::fmt::Debug for Restorer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Restorer")
    }
}

impl Restorer {
    /// Enters `screen` and returns the handle that will leave it.
    ///
    /// # Errors
    /// Entering failed; the screen is left again before returning.
    pub fn enter(mut screen: Box<dyn Screen>) -> io::Result<Self> {
        if let Err(e) = screen.enter() {
            let _ = screen.leave();
            return Err(e);
        }
        Ok(Self(Arc::new(Mutex::new(Some(screen)))))
    }

    /// Leaves the screen if it is still active. Returns true when this call left it.
    pub fn restore(&self) -> bool {
        // A panic while the lock was held must not stop the restore.
        let taken = self.0.lock().unwrap_or_else(PoisonError::into_inner).take();
        match taken {
            Some(mut screen) => {
                let _ = screen.leave();
                true
            }
            None => false,
        }
    }

    /// Whether the screen is still active.
    #[must_use]
    pub fn active(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }
}

/// Owns the terminal for the picker's lifetime; restores it when dropped.
#[derive(Debug)]
pub struct Session {
    restorer: Restorer,
}

impl Session {
    /// Enters `screen`, installs the panic hook and starts the signal watch.
    ///
    /// # Errors
    /// Entering the screen failed.
    pub fn start(screen: Box<dyn Screen>) -> io::Result<Self> {
        let restorer = Restorer::enter(screen)?;
        install_panic_hook(restorer.clone());
        if let Err(e) = watch_signals(restorer.clone(), exit_process) {
            restorer.restore();
            return Err(e);
        }
        Ok(Self { restorer })
    }

    /// The shared restore handle.
    #[must_use]
    pub fn restorer(&self) -> &Restorer {
        &self.restorer
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.restorer.restore();
    }
}

/// Chains a panic hook that restores the terminal before the previous hook prints.
pub fn install_panic_hook(restorer: Restorer) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restorer.restore();
        previous(info);
    }));
}

/// The exit status for a signal: 128 plus its number.
#[must_use]
pub fn signal_exit_code(signo: i32) -> u8 {
    u8::try_from(128 + signo).unwrap_or(u8::MAX)
}

/// Watches SIGTERM, SIGINT and SIGHUP on a background thread. On the first one it restores
/// the terminal and calls `on_signal` with [`signal_exit_code`]. Returns once the handlers
/// are registered.
///
/// # Errors
/// The handlers could not be registered.
pub fn watch_signals(
    restorer: Restorer,
    on_signal: impl FnOnce(u8) + Send + 'static,
) -> io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<io::Result<()>>();
    std::thread::Builder::new()
        .name("rstory-pick-signals".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                    return;
                }
            };
            rt.block_on(async move {
                let streams = (|| {
                    Ok::<_, io::Error>((
                        signal(SignalKind::terminate())?,
                        signal(SignalKind::interrupt())?,
                        signal(SignalKind::hangup())?,
                    ))
                })();
                let (mut term, mut int, mut hup) = match streams {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(()));
                let signo = tokio::select! {
                    _ = term.recv() => 15,
                    _ = int.recv() => 2,
                    _ = hup.recv() => 1,
                };
                restorer.restore();
                on_signal(signal_exit_code(signo));
            });
        })?;
    ready_rx
        .recv()
        .unwrap_or_else(|_| Err(io::Error::other("the signal thread stopped")))
}

/// The picker's key for a crossterm key event; `None` for releases and keys it ignores.
#[must_use]
pub fn key_from(ev: crossterm::event::KeyEvent) -> Option<super::state::Key> {
    use super::state::Key;
    use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
    if ev.kind == KeyEventKind::Release {
        return None;
    }
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    Some(match ev.code {
        KeyCode::Char(c) if ctrl => Key::Ctrl(c.to_ascii_lowercase()),
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::F(n) => Key::F(n),
        _ => return None,
    })
}

/// Exits the process with `code`: what the real signal watch does after restoring.
pub fn exit_process(code: u8) {
    std::process::exit(i32::from(code));
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::picker::state::EXIT_NO_TTY;

    /// Counts enters and leaves; optionally fails to enter.
    struct Fake {
        enters: Arc<AtomicUsize>,
        leaves: Arc<AtomicUsize>,
        fail_enter: bool,
    }

    fn fake(fail_enter: bool) -> (Box<Fake>, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let enters = Arc::new(AtomicUsize::new(0));
        let leaves = Arc::new(AtomicUsize::new(0));
        let f = Fake {
            enters: enters.clone(),
            leaves: leaves.clone(),
            fail_enter,
        };
        (Box::new(f), enters, leaves)
    }

    impl Screen for Fake {
        fn enter(&mut self) -> io::Result<()> {
            self.enters.fetch_add(1, Ordering::SeqCst);
            if self.fail_enter {
                return Err(io::Error::other("no raw mode"));
            }
            Ok(())
        }
        fn leave(&mut self) -> io::Result<()> {
            self.leaves.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn crossterm_keys_map_to_picker_keys() {
        use crate::picker::state::Key;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        let k = |code, m| key_from(KeyEvent::new(code, m));
        assert_eq!(
            k(KeyCode::Char('t'), KeyModifiers::CONTROL),
            Some(Key::Ctrl('t'))
        );
        assert_eq!(
            k(KeyCode::Char('C'), KeyModifiers::CONTROL),
            Some(Key::Ctrl('c'))
        );
        assert_eq!(
            k(KeyCode::Char('R'), KeyModifiers::SHIFT),
            Some(Key::Char('R'))
        );
        assert_eq!(k(KeyCode::Enter, KeyModifiers::NONE), Some(Key::Enter));
        assert_eq!(k(KeyCode::F(1), KeyModifiers::NONE), Some(Key::F(1)));
        assert_eq!(k(KeyCode::Insert, KeyModifiers::NONE), None);
        let mut release = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(key_from(release), None);
    }

    #[test]
    fn a_terminal_passes_the_tty_check() {
        assert!(require_tty(true).is_ok());
    }

    #[test]
    fn no_terminal_is_exit_2_with_the_message() {
        let exit = require_tty(false).unwrap_err();
        assert_eq!(exit.code, EXIT_NO_TTY);
        assert_eq!(exit.message.as_deref(), Some(NO_TTY_MESSAGE));
    }

    #[test]
    fn restore_leaves_once_however_often_it_is_called() {
        let (f, enters, leaves) = fake(false);
        let r = Restorer::enter(f).unwrap();
        assert_eq!(enters.load(Ordering::SeqCst), 1);
        assert!(r.active());
        let clone = r.clone();
        assert!(clone.restore());
        assert!(!r.restore());
        assert!(!r.active());
        assert_eq!(leaves.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_failed_enter_is_undone_and_reported() {
        let (f, _, leaves) = fake(true);
        assert!(Restorer::enter(f).is_err());
        assert_eq!(leaves.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn signals_exit_128_plus_their_number() {
        assert_eq!(signal_exit_code(1), 129); // SIGHUP
        assert_eq!(signal_exit_code(2), 130); // SIGINT
        assert_eq!(signal_exit_code(15), 143); // SIGTERM
    }

    #[test]
    fn dropping_the_session_restores() {
        let (f, _, leaves) = fake(false);
        let r = {
            let r = Restorer::enter(f).unwrap();
            let s = Session {
                restorer: r.clone(),
            };
            drop(s);
            r
        };
        assert!(!r.active());
        assert_eq!(leaves.load(Ordering::SeqCst), 1);
    }
}
