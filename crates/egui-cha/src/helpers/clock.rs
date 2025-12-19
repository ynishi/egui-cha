//! Clock abstraction for time-based helpers
//!
//! This module provides a `Clock` trait that abstracts time access,
//! enabling testable time-dependent code with `FakeClock`.

use std::time::{Duration, Instant};

/// A clock that provides the current time
///
/// This trait abstracts time access, allowing for:
/// - Normal operation with `SystemClock`
/// - Testing with `FakeClock` (see `testing` module)
pub trait Clock: Clone {
    /// Get the current time as a duration since the clock's epoch
    fn now(&self) -> Duration;
}

/// System clock using real time
///
/// Uses `std::time::Instant` internally, with the epoch being
/// when the clock was created.
#[derive(Clone)]
pub struct SystemClock {
    start: Instant,
}

impl SystemClock {
    /// Create a new system clock
    ///
    /// The current time becomes the epoch (time zero).
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }
}
