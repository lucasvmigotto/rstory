//! `RecordingState`: pause, timed pause, resume (domain model section 7, rules R1 to R4).
//! Pure: the caller supplies `now` and persists the result.

use std::time::Duration;

use thiserror::Error;
use time::OffsetDateTime;

use super::entry::PauseState;

/// Shortest and longest timed pause (R4).
pub const PAUSE_RANGE: (Duration, Duration) =
    (Duration::from_secs(1), Duration::from_secs(24 * 60 * 60));

/// Why a pause was refused.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum PauseError {
    /// The duration is outside 1 s to 24 h.
    #[error("a timed pause lasts from 1 s to 24 h")]
    OutOfRange,
}

/// The recording control singleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordingState {
    state: PauseState,
}

impl RecordingState {
    /// Loads the persisted state; a timed pause whose end has passed resumes (R3). The flag says
    /// whether the state changed and must be persisted again.
    #[must_use]
    pub fn load(stored: PauseState, now: OffsetDateTime) -> (Self, bool) {
        let mut s = Self { state: stored };
        let changed = s.tick(now);
        (s, changed)
    }

    /// The state to persist.
    #[must_use]
    pub fn state(&self) -> PauseState {
        self.state
    }

    /// Whether copies must be skipped (R1).
    #[must_use]
    pub fn is_paused(&self) -> bool {
        matches!(self.state, PauseState::Paused { .. })
    }

    /// End of a timed pause.
    #[must_use]
    pub fn until(&self) -> Option<OffsetDateTime> {
        match self.state {
            PauseState::Paused { until } => until,
            PauseState::Recording => None,
        }
    }

    /// `PauseRecording{until?}`: indefinite without a duration, else ending at `now + d`.
    /// Pausing again replaces the end time. Returns whether the state changed.
    ///
    /// # Errors
    /// [`PauseError::OutOfRange`] (R4); nothing changes.
    pub fn pause(&mut self, now: OffsetDateTime, d: Option<Duration>) -> Result<bool, PauseError> {
        let until = match d {
            None => None,
            Some(d) if d < PAUSE_RANGE.0 || d > PAUSE_RANGE.1 => {
                return Err(PauseError::OutOfRange);
            }
            Some(d) => Some(now + d),
        };
        let next = PauseState::Paused { until };
        let changed = next != self.state;
        self.state = next;
        Ok(changed)
    }

    /// `ResumeRecording`; a no-op success while recording. Returns whether the state changed.
    pub fn resume(&mut self) -> bool {
        std::mem::replace(&mut self.state, PauseState::Recording) != PauseState::Recording
    }

    /// `Tick(now)`: ends a timed pause that has reached its end (R2). Returns whether it did.
    pub fn tick(&mut self, now: OffsetDateTime) -> bool {
        match self.state {
            PauseState::Paused { until: Some(t) } if now >= t => {
                self.state = PauseState::Recording;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    const T0: OffsetDateTime = datetime!(2026-10-21 14:00:00 UTC);
    const fn secs(n: u64) -> Option<Duration> {
        Some(Duration::from_secs(n))
    }

    #[test]
    fn indefinite_pause_and_resume() {
        let mut r = RecordingState::load(PauseState::Recording, T0).0;
        assert!(!r.is_paused());
        assert_eq!(r.pause(T0, None), Ok(true));
        assert!(r.is_paused() && r.until().is_none());
        assert_eq!(r.pause(T0, None), Ok(false), "same pause is idempotent");
        assert!(r.resume());
        assert!(!r.resume(), "resume while recording is a no-op");
    }

    #[test]
    fn timed_pause_ends_at_its_instant_and_not_before() {
        let mut r = RecordingState::load(PauseState::Recording, T0).0;
        r.pause(T0, secs(50)).unwrap();
        assert_eq!(r.until(), Some(T0 + Duration::from_secs(50)));
        assert!(!r.tick(T0 + Duration::from_secs(49)));
        assert!(r.is_paused());
        assert!(r.tick(T0 + Duration::from_secs(50)));
        assert!(!r.is_paused());
        assert!(
            !r.tick(T0 + Duration::from_secs(60)),
            "tick while recording is ignored"
        );
    }

    #[test]
    fn a_new_pause_replaces_the_end_time() {
        let mut r = RecordingState::load(PauseState::Recording, T0).0;
        r.pause(T0, secs(50)).unwrap();
        assert_eq!(r.pause(T0, secs(60)), Ok(true));
        assert_eq!(r.until(), Some(T0 + Duration::from_secs(60)));
        r.pause(T0, None).unwrap();
        assert_eq!(r.until(), None);
        assert!(r.is_paused());
    }

    #[test]
    fn the_duration_must_be_within_one_second_and_a_day() {
        let mut r = RecordingState::load(PauseState::Recording, T0).0;
        assert_eq!(r.pause(T0, secs(0)), Err(PauseError::OutOfRange));
        assert_eq!(r.pause(T0, secs(86_401)), Err(PauseError::OutOfRange));
        assert!(!r.is_paused(), "a refused pause changes nothing");
        assert!(r.pause(T0, secs(1)).is_ok());
        assert!(r.pause(T0, secs(86_400)).is_ok());
    }

    #[test]
    fn load_keeps_a_pause_in_force_and_resumes_an_expired_one() {
        let timed = PauseState::Paused {
            until: Some(T0 + Duration::from_secs(50)),
        };
        let (r, changed) = RecordingState::load(timed, T0);
        assert!(r.is_paused() && !changed);
        let (r, changed) = RecordingState::load(timed, T0 + Duration::from_secs(50));
        assert!(!r.is_paused() && changed);
        let (r, changed) = RecordingState::load(PauseState::Paused { until: None }, T0);
        assert!(r.is_paused() && !changed);
    }
}
