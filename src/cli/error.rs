//! What goes wrong in a command: an exit code and one line for stderr.
//!
//! Messages come from `ui::copy` keys and never carry clipboard content (Constitution III).

use super::ExitCode;
use crate::ipc::client::ClientError;
use crate::ipc::error::ErrorCode;
use crate::ui::copy;

/// A failed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    /// How the process exits.
    pub exit: ExitCode,
    /// The line for stderr.
    pub message: String,
}

impl CliError {
    /// An error with an exit code and a message.
    #[must_use]
    pub fn new(exit: ExitCode, message: impl Into<String>) -> Self {
        Self {
            exit,
            message: message.into(),
        }
    }

    /// `cli.daemon_down`, exit 3.
    #[must_use]
    pub fn daemon_down() -> Self {
        Self::new(ExitCode::DaemonDown, copy::CLI_DAEMON_DOWN)
    }

    /// An environment error, exit 5.
    #[must_use]
    pub fn environment(message: impl Into<String>) -> Self {
        Self::new(ExitCode::Environment, message)
    }

    /// A usage error, exit 2.
    #[must_use]
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ExitCode::Usage, message)
    }

    /// `cli.not_found`, exit 2.
    #[must_use]
    pub fn not_found(reference: &str) -> Self {
        Self::usage(copy::format(copy::CLI_NOT_FOUND, &[("ref", reference)]))
    }

    /// `cli.ambiguous`, exit 2, naming the candidates.
    #[must_use]
    pub fn ambiguous(reference: &str, candidates: &[String]) -> Self {
        Self::usage(copy::format(
            copy::CLI_AMBIGUOUS,
            &[
                ("ref", reference),
                ("n", &candidates.len().to_string()),
                ("ids", &candidates.join(", ")),
            ],
        ))
    }

    /// Turns a client failure into the exit code and message of the contract. `reference` is
    /// the entry reference the user typed, when the command had one.
    #[must_use]
    pub fn from_client(error: ClientError, reference: Option<&str>) -> Self {
        let reference = reference.unwrap_or("");
        match error {
            ClientError::NotRunning => Self::daemon_down(),
            ClientError::Timeout => Self::new(
                ExitCode::Unexpected,
                "The daemon did not answer in time. Run `rstory doctor` for details.",
            ),
            ClientError::Io(_) | ClientError::Protocol(_) => {
                Self::new(ExitCode::Unexpected, error.to_string())
            }
            ClientError::Remote(e) => match e.code {
                ErrorCode::NotFound => Self::not_found(reference),
                ErrorCode::Ambiguous => {
                    let names = e.message.split_once("candidates:").map_or("", |(_, c)| c);
                    let list: Vec<String> = names
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    Self::ambiguous(reference, &list)
                }
                ErrorCode::PinLimit => {
                    let max: String = e
                        .message
                        .split_once('(')
                        .map(|(_, rest)| rest.chars().take_while(char::is_ascii_digit).collect())
                        .unwrap_or_default();
                    if max.is_empty() {
                        Self::new(ExitCode::LimitReached, e.message)
                    } else {
                        Self::new(
                            ExitCode::LimitReached,
                            copy::format(copy::CLI_PIN_LIMIT, &[("max", &max)]),
                        )
                    }
                }
                ErrorCode::Inactive => Self::new(ExitCode::Inactive, copy::CLI_INACTIVE),
                code => Self::new(ExitCode::from_proto(code), e.message),
            },
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::error::ProtoError;

    fn remote(code: ErrorCode, message: &str) -> ClientError {
        ClientError::Remote(ProtoError::new(code, message))
    }

    #[test]
    fn a_missing_daemon_is_exit_3_with_the_hint() {
        let e = CliError::from_client(ClientError::NotRunning, None);
        assert_eq!(e.exit, ExitCode::DaemonDown);
        assert_eq!(e.message, copy::CLI_DAEMON_DOWN);
    }

    #[test]
    fn not_found_names_the_reference() {
        let e = CliError::from_client(remote(ErrorCode::NotFound, "no entry ab12"), Some("ab12"));
        assert_eq!(e.exit, ExitCode::Usage);
        assert!(e.message.starts_with("No entry ab12."), "{}", e.message);
    }

    #[test]
    fn ambiguous_lists_the_candidates() {
        let e = CliError::from_client(
            remote(
                ErrorCode::Ambiguous,
                "ambiguous; candidates: 01JBD8X, 01JBD8Z",
            ),
            Some("01JB"),
        );
        assert_eq!(e.exit, ExitCode::Usage);
        assert_eq!(
            e.message,
            "Short id 01JB matches 2 entries: 01JBD8X, 01JBD8Z. Use more characters."
        );
    }

    #[test]
    fn pin_limit_is_exit_6_with_the_cap() {
        let e = CliError::from_client(remote(ErrorCode::PinLimit, "pin limit reached (10)"), None);
        assert_eq!(e.exit, ExitCode::LimitReached);
        assert_eq!(e.message, "Pin limit reached (10). Unpin an entry first.");
    }

    #[test]
    fn inactive_is_exit_7() {
        let e = CliError::from_client(remote(ErrorCode::Inactive, "no backend"), None);
        assert_eq!(
            (e.exit, e.message.as_str()),
            (ExitCode::Inactive, copy::CLI_INACTIVE)
        );
    }

    #[test]
    fn other_daemon_errors_keep_the_catalog_exit_code() {
        let e = CliError::from_client(remote(ErrorCode::Internal, "internal error (c1)"), None);
        assert_eq!(e.exit, ExitCode::Unexpected);
        let e = CliError::from_client(remote(ErrorCode::BadRequest, "bad"), None);
        assert_eq!(e.exit, ExitCode::Usage);
    }

    #[test]
    fn transport_failures_are_exit_1() {
        let e = CliError::from_client(ClientError::Timeout, None);
        assert_eq!(e.exit, ExitCode::Unexpected);
        let e = CliError::from_client(ClientError::Protocol("x".into()), None);
        assert_eq!(e.exit, ExitCode::Unexpected);
    }
}
