//! The one deadline timer: `min(next expiry, paused_until, reconnect_at)` (research R5).
//! There is no periodic tick; the loop re-arms after every change.

use time::OffsetDateTime;

/// The earliest of the given instants, if any.
#[must_use]
pub fn next_deadline(
    expiry: Option<OffsetDateTime>,
    pause_end: Option<OffsetDateTime>,
    reconnect: Option<OffsetDateTime>,
) -> Option<OffsetDateTime> {
    [expiry, pause_end, reconnect].into_iter().flatten().min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn the_earliest_wins_and_none_means_no_timer() {
        let a = datetime!(2026-10-21 14:00:00 UTC);
        let b = datetime!(2026-10-21 15:00:00 UTC);
        let c = datetime!(2026-10-21 16:00:00 UTC);
        assert_eq!(next_deadline(None, None, None), None);
        assert_eq!(next_deadline(Some(b), None, None), Some(b));
        assert_eq!(next_deadline(Some(c), Some(a), Some(b)), Some(a));
        assert_eq!(next_deadline(Some(c), None, Some(b)), Some(b));
    }
}
