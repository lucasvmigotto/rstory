//! `Redacted<T>`: wraps anything content-derived so it can never reach a log or a message.

use std::fmt;

/// Prints as `<N bytes>`, never the content. Use for every value derived from clipboard data
/// that might end up in a `tracing` field, a panic message or an error.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Redacted<T: AsRef<[u8]>>(pub T);

impl<T: AsRef<[u8]>> Redacted<T> {
    /// Size of the hidden content in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    /// Whether the hidden content is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: AsRef<[u8]>> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} bytes>", self.len())
    }
}

impl<T: AsRef<[u8]>> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANARY: &str = "hunter2-CANARY-secret";

    #[test]
    fn display_and_debug_show_only_the_size() {
        let r = Redacted(CANARY.to_string());
        assert_eq!(format!("{r}"), "<21 bytes>");
        assert_eq!(format!("{r:?}"), "<21 bytes>");
        assert_eq!(format!("{r:#?}"), "<21 bytes>");
    }

    #[test]
    fn content_never_appears_for_any_formatting() {
        let r = Redacted(CANARY.as_bytes().to_vec());
        for s in [
            format!("{r}"),
            format!("{r:?}"),
            format!("{r:>40}"),
            format!("{r:#?}"),
        ] {
            assert!(!s.contains("CANARY") && !s.contains("hunter2"), "{s}");
        }
        let nested = Some(Redacted(CANARY));
        assert!(!format!("{nested:?}").contains("CANARY"));
    }

    #[test]
    fn empty_and_len() {
        let r = Redacted("");
        assert!(r.is_empty());
        assert_eq!(format!("{r}"), "<0 bytes>");
    }
}
