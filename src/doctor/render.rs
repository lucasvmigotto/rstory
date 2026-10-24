//! The text and `--json` renderers.

use serde_json::json;

use super::{Check, Status, counts};
use crate::cli::render::Term;
use crate::ui::ansi;
use crate::ui::copy::DOCTOR_ALL_OK;
use crate::ui::text::display_width;
use crate::ui::widgets::{CheckLine, CheckStatus};

/// The word in front of a check's message.
fn label(id: &str) -> &str {
    match id {
        "session" => "Session",
        "backend" => "Backend",
        "config" => "Config",
        "store" => "Store",
        "daemon" => "Daemon",
        "pause" => "Pause",
        "tray" => "Tray",
        "gnome_shell" => "GNOME Shell",
        "gnome_extension" => "Extension",
        "picker" => "Picker",
        "autostart" => "Autostart",
        other => other,
    }
}

fn widget_status(s: Status) -> CheckStatus {
    match s {
        Status::Ok => CheckStatus::Ok,
        Status::Info => CheckStatus::Info,
        Status::Warn => CheckStatus::Warn,
        Status::Fail => CheckStatus::Fail,
    }
}

fn plural(n: usize, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}

/// `1 warning.`, `2 warnings, 1 failure.`, or the all-clear line.
fn summary(checks: &[Check]) -> String {
    match counts(checks) {
        (0, 0) => DOCTOR_ALL_OK.to_string(),
        (w, 0) => format!("{}.", plural(w, "warning")),
        (0, f) => format!("{}.", plural(f, "failure")),
        (w, f) => format!("{}, {}.", plural(w, "warning"), plural(f, "failure")),
    }
}

/// The report for people: one line per check, a hint line under each non-ok one, then a summary.
#[must_use]
pub fn text(checks: &[Check], term: &Term) -> String {
    let ctx = term.ctx();
    let width = checks
        .iter()
        .map(|c| display_width(label(c.id)))
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for c in checks {
        let name = label(c.id);
        let padded = format!("{name}{}", " ".repeat(width - display_width(name)));
        let message = term.text(&c.message);
        let hint = c.hint.as_deref().map(|h| term.text(h));
        let line = CheckLine {
            status: widget_status(c.status),
            label: &padded,
            message: &message,
            hint: hint.as_deref(),
        };
        for l in line.lines(ctx) {
            out.push_str(&if term.styled() {
                ansi::ansi(&l)
            } else {
                ansi::plain(&l)
            });
            out.push('\n');
        }
    }
    out.push('\n');
    out.push_str(&term.text(&summary(checks)));
    out.push('\n');
    out
}

/// The report for scripts: `{"schema":1,"checks":[{id,status,message,hint}]}` and a newline.
#[must_use]
pub fn json(checks: &[Check]) -> String {
    let items: Vec<_> = checks
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "status": c.status.as_str(),
                "message": c.message,
                "hint": c.hint,
            })
        })
        .collect();
    format!("{}\n", json!({"schema": 1, "checks": items}))
}
