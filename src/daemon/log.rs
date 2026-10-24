//! Logging setup. Content is never logged (Constitution III); wrap it in `Redacted`.

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

/// Installs a `tracing` fmt subscriber on stderr, filtered by `RUST_LOG` (default `info`).
/// ANSI colours are used only when stderr is a terminal. Safe to call more than once.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_is_idempotent() {
        super::init_logging();
        super::init_logging();
    }
}
