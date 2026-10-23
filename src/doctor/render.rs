//! The text and `--json` renderers.

use super::Check;
use crate::cli::render::Term;

/// The report for people: one line per check, a hint line under each non-ok one, then a summary.
#[must_use]
pub fn text(checks: &[Check], term: &Term) -> String {
    let _ = (checks, term);
    todo!()
}

/// The report for scripts: `{"schema":1,"checks":[{id,status,message,hint}]}` and a newline.
#[must_use]
pub fn json(checks: &[Check]) -> String {
    let _ = checks;
    todo!()
}
