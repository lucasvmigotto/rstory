//! Shared fixtures for the session tests: private X servers and a minimal X11 test client.
//! Every launcher gives each test its own display and sockets so tests can run in parallel.
#![allow(dead_code)]

pub mod mutter;
pub mod x11;

use std::time::{Duration, Instant};

/// Poll `check` every few milliseconds until it returns `Some`, or give up at `timeout`.
/// Readiness is always observed, never slept for.
pub fn wait_for<T>(timeout: Duration, mut check: impl FnMut() -> Option<T>) -> Option<T> {
    let end = Instant::now() + timeout;
    loop {
        if let Some(v) = check() {
            return Some(v);
        }
        if Instant::now() >= end {
            return None;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
