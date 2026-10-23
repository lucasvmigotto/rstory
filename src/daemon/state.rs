//! Daemon state view, status read model, clock and reconnect policy.

use std::time::Duration;

use time::OffsetDateTime;
use tokio::time::Instant;

use crate::backend::{BackendKind, Mode};
use crate::domain::pause::RecordingState;
use crate::ipc::protocol::{State, StateKind};

/// What the daemon knows about its backend, for `status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendInfo {
    /// Which backend (or `None`).
    pub kind: BackendKind,
    /// How it reaches the clipboard.
    pub mode: Mode,
    /// Why there is no backend (selection reason), when there is none.
    pub reason: Option<String>,
}

impl BackendInfo {
    /// An active backend.
    #[must_use]
    pub fn active(kind: BackendKind, mode: Mode) -> Self {
        Self {
            kind,
            mode,
            reason: None,
        }
    }

    /// No backend, for `reason`.
    #[must_use]
    pub fn none(reason: &str) -> Self {
        Self {
            kind: BackendKind::None,
            mode: Mode::None,
            reason: Some(reason.to_string()),
        }
    }

    /// `status.mode`.
    #[must_use]
    pub fn mode_name(&self) -> Option<String> {
        match self.mode {
            Mode::None => None,
            m => Some(m.as_str().to_string()),
        }
    }
}

/// The state clients see: `inactive` wins over a persisted pause.
#[must_use]
pub fn view(rec: &RecordingState, active: bool, reason: Option<&str>) -> State {
    if !active {
        return State {
            state: StateKind::Inactive,
            paused_until: None,
            reason: Some(reason.unwrap_or("no_backend").to_string()),
        };
    }
    State {
        state: if rec.is_paused() {
            StateKind::Paused
        } else {
            StateKind::Recording
        },
        paused_until: rec.until(),
        reason: None,
    }
}

/// Wall clock that follows tokio's clock, so tests with paused time move it too.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    wall: OffsetDateTime,
    at: Instant,
}

impl Clock {
    /// A clock that reads `wall` now.
    #[must_use]
    pub fn at(wall: OffsetDateTime) -> Self {
        Self {
            wall,
            at: Instant::now(),
        }
    }

    /// The system wall clock.
    #[must_use]
    pub fn system() -> Self {
        Self::at(OffsetDateTime::now_utc())
    }

    /// The current time.
    #[must_use]
    pub fn now(&self) -> OffsetDateTime {
        self.wall + self.at.elapsed()
    }

    /// The tokio instant at which the clock reads `t` (now when `t` has passed).
    #[must_use]
    pub fn instant_at(&self, t: OffsetDateTime) -> Instant {
        let ahead = (t - self.now()).max(time::Duration::ZERO);
        Instant::now() + Duration::try_from(ahead).unwrap_or(Duration::ZERO)
    }
}

/// Reconnect delays: 100 ms doubling to 5 s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    next: Duration,
}

impl Backoff {
    /// First delay.
    pub const START: Duration = Duration::from_millis(100);
    /// Longest delay.
    pub const MAX: Duration = Duration::from_secs(5);

    /// A fresh policy.
    #[must_use]
    pub fn new() -> Self {
        Self { next: Self::START }
    }

    /// The delay to wait now; the following one doubles.
    pub fn next_delay(&mut self) -> Duration {
        let d = self.next;
        self.next = (self.next * 2).min(Self::MAX);
        d
    }

    /// Back to the first delay after a successful connection.
    pub fn reset(&mut self) {
        self.next = Self::START;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entry::PauseState;
    use time::macros::datetime;

    const T0: OffsetDateTime = datetime!(2026-10-21 14:00:00 UTC);

    #[test]
    fn backoff_doubles_from_100ms_to_5s_and_resets() {
        let mut b = Backoff::new();
        let ms: Vec<u128> = (0..8).map(|_| b.next_delay().as_millis()).collect();
        assert_eq!(ms, [100, 200, 400, 800, 1600, 3200, 5000, 5000]);
        b.reset();
        assert_eq!(b.next_delay(), Backoff::START);
    }

    #[test]
    fn inactive_wins_over_a_persisted_pause() {
        let (rec, _) = RecordingState::load(PauseState::Paused { until: None }, T0);
        let v = view(&rec, false, Some("no_session"));
        assert_eq!(v.state, StateKind::Inactive);
        assert_eq!(v.reason.as_deref(), Some("no_session"));
        assert_eq!(view(&rec, true, None).state, StateKind::Paused);
    }

    #[tokio::test(start_paused = true)]
    async fn the_clock_follows_tokio_time() {
        let c = Clock::at(T0);
        assert_eq!(c.now(), T0);
        tokio::time::advance(Duration::from_secs(90)).await;
        assert_eq!(c.now(), T0 + Duration::from_secs(90));
        let target = T0 + Duration::from_secs(100);
        let i = c.instant_at(target);
        assert_eq!(i - Instant::now(), Duration::from_secs(10));
        assert_eq!(c.instant_at(T0), Instant::now(), "the past is now");
    }
}
