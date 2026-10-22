//! Target choice and text decoding for reads (FR-003).

/// Text targets in order of preference.
pub const PREFERENCE: [&str; 4] = ["UTF8_STRING", "text/plain;charset=utf-8", "STRING", "TEXT"];

/// The best text target among the owner's offered `types`.
#[must_use]
pub fn pick_target(_types: &[String]) -> Option<&'static str> {
    None
}

/// ISO 8859-1 bytes as UTF-8 text.
#[must_use]
pub fn latin1_to_utf8(_bytes: &[u8]) -> String {
    String::new()
}

/// UTF-8 bytes with invalid sequences replaced by U+FFFD, and the number of replacements.
#[must_use]
pub fn utf8_lossy(_bytes: Vec<u8>) -> (Vec<u8>, usize) {
    (Vec::new(), 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| (*n).to_string()).collect()
    }

    #[test]
    fn prefers_utf8_then_plain_then_string_then_text() {
        assert_eq!(
            pick_target(&t(&["STRING", "UTF8_STRING", "TEXT"])),
            Some("UTF8_STRING")
        );
        assert_eq!(
            pick_target(&t(&["STRING", "text/plain;charset=utf-8"])),
            Some("text/plain;charset=utf-8")
        );
        assert_eq!(pick_target(&t(&["TEXT", "STRING"])), Some("STRING"));
        assert_eq!(pick_target(&t(&["TEXT"])), Some("TEXT"));
    }

    #[test]
    fn no_text_target_means_none() {
        assert_eq!(pick_target(&t(&["image/png", "TARGETS"])), None);
        assert_eq!(pick_target(&[]), None);
    }

    #[test]
    fn latin1_maps_each_byte_to_its_code_point() {
        assert_eq!(latin1_to_utf8(b"caf\xe9 \xff"), "caf\u{e9} \u{ff}");
        assert_eq!(latin1_to_utf8(b""), "");
    }

    #[test]
    fn invalid_utf8_is_replaced_and_counted() {
        assert_eq!(
            utf8_lossy(b"ab\xffcd\xc3".to_vec()),
            ("ab\u{fffd}cd\u{fffd}".as_bytes().to_vec(), 2)
        );
        assert_eq!(
            utf8_lossy("ok \u{e9}".as_bytes().to_vec()),
            ("ok \u{e9}".as_bytes().to_vec(), 0)
        );
    }
}
