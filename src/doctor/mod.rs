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

/// What the arguments after `doctor` ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub json: bool,
}

/// Parses the arguments after `doctor` (only `--json`).
///
/// # Errors
/// A message for any other argument.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options { json: false };
    for a in args {
        match a.as_str() {
            "--json" => o.json = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(o)
}

/// The command against injected inputs: parses `args`, gathers, evaluates, writes the report
/// to `out` and usage errors to `err`. Returns the exit code.
#[must_use]
pub fn run_with(
    args: &[String],
    env: &probes::ProbeEnv,
    probe: &mut dyn crate::session::Probe,
    term: &crate::cli::render::Term,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> u8 {
    let opts = match parse_args(args) {
        Ok(o) => o,
        Err(m) => {
            let _ = writeln!(err, "rstory doctor: {m}\nUsage: rstory doctor [--json]");
            return 2;
        }
    };
    let checks = checks::evaluate(&probes::gather(env, probe));
    let report = if opts.json {
        render::json(&checks)
    } else {
        render::text(&checks, term)
    };
    let _ = out.write_all(report.as_bytes());
    exit_code(&checks)
}

/// `rstory doctor [--json]`: probes the real session, prints the report on stdout and returns
/// the doctor exit code (0, 1 or 2; 2 also for a usage error).
#[must_use]
pub fn run(args: &[String]) -> u8 {
    let env = probes::ProbeEnv::from_process();
    let mut probe = crate::session::RealProbe::new(&env.session, env.xauthority.clone());
    let term = crate::cli::render::Term::detect();
    let (mut out, mut err) = (std::io::stdout().lock(), std::io::stderr().lock());
    run_with(args, &env, &mut probe, &term, &mut out, &mut err)
}

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
        match self {
            Self::Ok => "ok",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
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
        Self {
            id,
            status: Status::Ok,
            message: message.into(),
            hint: None,
        }
    }

    /// A check with a status other than ok, and its next step.
    #[must_use]
    pub fn with_hint(
        id: &'static str,
        status: Status,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            id,
            status,
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}

/// The doctor exit scale (`contracts/cli.md`): 0 when everything is ok or info, 1 when any
/// warn, 2 when any fail.
#[must_use]
pub fn exit_code(checks: &[Check]) -> u8 {
    match checks.iter().map(|c| c.status).max() {
        Some(Status::Fail) => 2,
        Some(Status::Warn) => 1,
        _ => 0,
    }
}

/// How many results are warnings and failures.
#[must_use]
pub fn counts(checks: &[Check]) -> (usize, usize) {
    let n = |s| checks.iter().filter(|c| c.status == s).count();
    (n(Status::Warn), n(Status::Fail))
}
