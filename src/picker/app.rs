//! Carries out [`Cmd`] values against a [`Source`] and feeds the answers back into
//! [`update`]. [`drive`] is the headless loop the journey tests use; the interactive loop in
//! `run` shares [`execute`] and adds drawing, key reading and the preview debounce.

use super::data::Source;
use super::state::{Action, Cmd, Exit, Msg, PickerState, update};

/// Runs one command; returns the message that answers it (`None` for `Quit`).
pub fn execute(cmd: &Cmd, source: &mut dyn Source) -> Option<Msg> {
    let act =
        |source: &mut dyn Source, action, id: &str| Some(Msg::Done(action, source.act(action, id)));
    match cmd {
        Cmd::Restore(id) => act(source, Action::Restore, id),
        Cmd::Pin(id) => act(source, Action::Pin, id),
        Cmd::Unpin(id) => act(source, Action::Unpin, id),
        Cmd::Remove(id) => act(source, Action::Remove, id),
        Cmd::Show(id) => Some(match source.show(id) {
            Ok(text) => Msg::Content {
                id: id.clone(),
                text,
            },
            Err(_) => Msg::ContentFailed { id: id.clone() },
        }),
        // A failed relist keeps the current list; the next change tries again.
        Cmd::Reload => source.list().ok().map(Msg::Loaded),
        Cmd::Quit(_) => None,
    }
}

/// Applies `msg`, then executes every resulting command (and the commands their answers
/// produce) until the queue is empty or one quits.
pub fn step(state: &mut PickerState, source: &mut dyn Source, msg: Msg) -> Option<Exit> {
    let mut queue = std::collections::VecDeque::from(update(state, msg));
    while let Some(cmd) = queue.pop_front() {
        if let Cmd::Quit(exit) = cmd {
            return Some(exit);
        }
        if let Some(answer) = execute(&cmd, source) {
            queue.extend(update(state, answer));
        }
    }
    None
}

/// Feeds `inputs` in order through [`step`]; returns the exit once one is asked for.
pub fn drive(
    state: &mut PickerState,
    source: &mut dyn Source,
    inputs: impl IntoIterator<Item = Msg>,
) -> Option<Exit> {
    for msg in inputs {
        if let Some(exit) = step(state, source, msg) {
            return Some(exit);
        }
    }
    None
}

/// How long the selection must rest before the preview content is fetched (tui.md "Data").
pub const SHOW_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(30);

/// A pending preview fetch: the latest requested id and when to fetch it.
#[derive(Debug, Default)]
pub struct Debounce {
    pending: Option<(String, std::time::Instant)>,
}

impl Debounce {
    /// Replaces any pending fetch with `id`, due [`SHOW_DEBOUNCE`] after `now`.
    pub fn request(&mut self, id: String, now: std::time::Instant) {
        self.pending = Some((id, now + SHOW_DEBOUNCE));
    }

    /// The id to fetch now, if its time has come.
    pub fn due(&mut self, now: std::time::Instant) -> Option<String> {
        match &self.pending {
            Some((_, at)) if *at <= now => self.pending.take().map(|(id, _)| id),
            _ => None,
        }
    }

    /// How long until the pending fetch is due (`None` when nothing is pending).
    #[must_use]
    pub fn wait(&self, now: std::time::Instant) -> Option<std::time::Duration> {
        self.pending
            .as_ref()
            .map(|(_, at)| at.saturating_duration_since(now))
    }
}
