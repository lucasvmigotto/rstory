//! `rstory doctor`: read-only checks of the session, backend, store, daemon, tray, GNOME
//! extension, picker terminal and autostart, each with a plain message and a next step.
//!
//! Spec: `specs/011-doctor`. The flow is `probes::gather` (the only code that looks at the
//! machine, every input injected) into [`checks::Facts`], then the pure [`checks::evaluate`]
//! into [`Check`]s, then [`render`]. Doctor never writes, enables or starts anything and never
//! reads an entry's content (Constitution III).

pub mod checks;
pub mod probes;
pub mod render;

#[cfg(test)]
mod tests;

/// How a check came out. The order is the severity order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    /// Works.
    Ok,
    /// Worth knowing, nothing to fix.
    Info,
    /// Something is missing; rstory still works without it.
    Warn,
    /// rstory cannot do its job until this is fixed.
    Fail,
}

impl Status {
    /// The name in `--json`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        todo!()
    }
}

/// One line of the report (data-model.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// Stable id: session, backend, store, daemon, pause, tray, gnome_extension, gnome_shell,
    /// picker, autostart.
    pub id: &'static str,
    /// The result.
    pub status: Status,
    /// What was found, in plain words.
    pub message: String,
    /// The next step. Every result that is not `Ok` has one.
    pub hint: Option<String>,
}

impl Check {
    /// A check that passed.
    #[must_use]
    pub fn ok(id: &'static str, message: impl Into<String>) -> Self {
        let _ = (id, message.into());
        todo!()
    }

    /// A check with a status other than ok, and its next step.
    #[must_use]
    pub fn with_hint(
        id: &'static str,
        status: Status,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        let _ = (id, status, message.into(), hint.into());
        todo!()
    }
}

/// The doctor exit scale (`contracts/cli.md`): 0 when everything is ok or info, 1 when any
/// warn, 2 when any fail.
#[must_use]
pub fn exit_code(checks: &[Check]) -> u8 {
    let _ = checks;
    todo!()
}

/// How many results are warnings and failures.
#[must_use]
pub fn counts(checks: &[Check]) -> (usize, usize) {
    let _ = checks;
    todo!()
}
