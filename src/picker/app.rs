//! Carries out [`Cmd`] values against a [`Source`] and feeds the answers back into
//! [`update`]. [`drive`] is the headless loop the journey tests use; the interactive loop in
//! `run` shares [`execute`] and adds drawing, key reading and the preview debounce.

use super::data::Source;
use super::state::{Action, Cmd, Exit, Msg, PickerState, update};

/// Runs one command; returns the message that answers it (`None` for `Quit`).
pub fn execute(cmd: &Cmd, source: &mut dyn Source) -> Option<Msg> {
    let _ = (cmd, source, Action::Restore);
    todo!()
}

/// Applies `msg`, then executes every resulting command (and the commands their answers
/// produce) until the queue is empty or one quits.
pub fn step(state: &mut PickerState, source: &mut dyn Source, msg: Msg) -> Option<Exit> {
    let _ = (state, source, msg, update);
    todo!()
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
