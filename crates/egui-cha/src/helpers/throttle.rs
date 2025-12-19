//! Throttle helper for TEA applications
//!
//! Throttling limits action frequency to at most once per interval.
//! Useful for scroll events, resize handlers, rate-limited APIs, etc.

use super::clock::Clock;
use crate::Cmd;
use std::time::{Duration, Instant};

/// Throttler - limits action frequency
///
/// # How it works
/// The first call to `run()` executes immediately. Subsequent calls within
/// the throttle interval are ignored. After the interval passes, the next
/// call will execute again.
///
/// # Example
/// ```ignore
/// use egui_cha::helpers::Throttler;
/// use std::time::Duration;
///
/// struct Model {
///     scroll_pos: f32,
///     scroll_throttle: Throttler,
/// }
///
/// enum Msg {
///     Scroll(f32),
///     UpdateVisibleItems,
/// }
///
/// fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
///     match msg {
///         Msg::Scroll(pos) => {
///             model.scroll_pos = pos;
///             // Only fires at most once per 100ms
///             model.scroll_throttle.run(
///                 Duration::from_millis(100),
///                 || Cmd::msg(Msg::UpdateVisibleItems),
///             )
///         }
///         Msg::UpdateVisibleItems => {
///             // Expensive computation, throttled
///             Cmd::none()
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Throttler {
    last_run: Option<Instant>,
}

impl Default for Throttler {
    fn default() -> Self {
        Self::new()
    }
}

impl Throttler {
    /// Create a new throttler
    pub fn new() -> Self {
        Self { last_run: None }
    }

    /// Run an action if not throttled
    ///
    /// If the throttle interval has passed since the last run (or this is
    /// the first call), executes the action and returns its result.
    /// Otherwise, returns `Cmd::none()`.
    ///
    /// # Arguments
    /// - `interval`: Minimum time between executions
    /// - `action`: Closure that returns a `Cmd<Msg>`
    pub fn run<Msg, F>(&mut self, interval: Duration, action: F) -> Cmd<Msg>
    where
        F: FnOnce() -> Cmd<Msg>,
    {
        let now = Instant::now();

        let should_run = match self.last_run {
            None => true,
            Some(last) => now.duration_since(last) >= interval,
        };

        if should_run {
            self.last_run = Some(now);
            action()
        } else {
            Cmd::none()
        }
    }

    /// Run with a message (convenience method)
    ///
    /// Emits the message if not throttled.
    pub fn run_msg<Msg>(&mut self, interval: Duration, msg: Msg) -> Cmd<Msg>
    where
        Msg: Clone,
    {
        self.run(interval, || Cmd::Msg(msg))
    }

    /// Check if an action would be throttled (without running)
    ///
    /// Returns `true` if calling `run()` now would be throttled.
    pub fn is_throttled(&self, interval: Duration) -> bool {
        match self.last_run {
            None => false,
            Some(last) => Instant::now().duration_since(last) < interval,
        }
    }

    /// Time remaining until throttle expires
    ///
    /// Returns `Some(duration)` if throttled, `None` if ready to run.
    pub fn time_remaining(&self, interval: Duration) -> Option<Duration> {
        self.last_run.and_then(|last| {
            let elapsed = Instant::now().duration_since(last);
            if elapsed < interval {
                Some(interval - elapsed)
            } else {
                None
            }
        })
    }

    /// Reset the throttler state
    ///
    /// The next `run()` call will execute immediately.
    pub fn reset(&mut self) {
        self.last_run = None;
    }

    /// Force the next run to be throttled for the given duration
    ///
    /// Useful when you want to prevent immediate execution after
    /// some external event.
    pub fn suppress(&mut self) {
        self.last_run = Some(Instant::now());
    }
}

// ============================================
// ThrottlerWithClock - testable version
// ============================================

/// A throttler with pluggable clock for testing
///
/// Like [`Throttler`], but uses a [`Clock`] trait for time access,
/// enabling deterministic testing with [`FakeClock`](crate::testing::FakeClock).
///
/// # Example
/// ```ignore
/// use egui_cha::testing::FakeClock;
/// use egui_cha::helpers::ThrottlerWithClock;
/// use std::time::Duration;
///
/// let clock = FakeClock::new();
/// let mut throttler = ThrottlerWithClock::new(clock.clone());
///
/// // First call executes
/// let cmd1 = throttler.run(Duration::from_millis(100), || Cmd::Msg(1));
/// assert!(cmd1.is_msg());
///
/// // Immediate second call is throttled
/// let cmd2 = throttler.run(Duration::from_millis(100), || Cmd::Msg(2));
/// assert!(cmd2.is_none());
///
/// // Advance time past throttle interval
/// clock.advance(Duration::from_millis(150));
/// let cmd3 = throttler.run(Duration::from_millis(100), || Cmd::Msg(3));
/// assert!(cmd3.is_msg());
/// ```
#[derive(Debug, Clone)]
pub struct ThrottlerWithClock<C: Clock> {
    clock: C,
    last_run: Option<Duration>,
}

impl<C: Clock> ThrottlerWithClock<C> {
    /// Create a new throttler with the given clock
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            last_run: None,
        }
    }

    /// Run an action if not throttled
    pub fn run<Msg, F>(&mut self, interval: Duration, action: F) -> Cmd<Msg>
    where
        F: FnOnce() -> Cmd<Msg>,
    {
        let now = self.clock.now();

        let should_run = match self.last_run {
            None => true,
            Some(last) => now >= last + interval,
        };

        if should_run {
            self.last_run = Some(now);
            action()
        } else {
            Cmd::none()
        }
    }

    /// Run with a message (convenience method)
    pub fn run_msg<Msg>(&mut self, interval: Duration, msg: Msg) -> Cmd<Msg>
    where
        Msg: Clone,
    {
        self.run(interval, || Cmd::Msg(msg))
    }

    /// Check if an action would be throttled
    pub fn is_throttled(&self, interval: Duration) -> bool {
        match self.last_run {
            None => false,
            Some(last) => self.clock.now() < last + interval,
        }
    }

    /// Time remaining until throttle expires
    pub fn time_remaining(&self, interval: Duration) -> Option<Duration> {
        self.last_run.and_then(|last| {
            let now = self.clock.now();
            let expires_at = last + interval;
            if now < expires_at {
                Some(expires_at - now)
            } else {
                None
            }
        })
    }

    /// Reset the throttler state
    pub fn reset(&mut self) {
        self.last_run = None;
    }

    /// Force the next run to be throttled
    pub fn suppress(&mut self) {
        self.last_run = Some(self.clock.now());
    }
}

/// Throttler with trailing edge execution
///
/// Like `Throttler`, but also fires once after the throttle period
/// if there were any suppressed calls.
///
/// # Example
/// ```ignore
/// struct Model {
///     throttle: TrailingThrottler,
/// }
///
/// fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
///     match msg {
///         Msg::Resize(size) => {
///             model.throttle.run(
///                 Duration::from_millis(100),
///                 Msg::DoResize(size),
///                 Msg::TrailingResize,
///             )
///         }
///         Msg::TrailingResize => {
///             if model.throttle.should_fire_trailing() {
///                 // Handle trailing edge
///             }
///             Cmd::none()
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TrailingThrottler {
    last_run: Option<Instant>,
    has_pending: bool,
    trailing_scheduled: bool,
}

impl Default for TrailingThrottler {
    fn default() -> Self {
        Self::new()
    }
}

impl TrailingThrottler {
    /// Create a new trailing throttler
    pub fn new() -> Self {
        Self {
            last_run: None,
            has_pending: false,
            trailing_scheduled: false,
        }
    }

    /// Run with both immediate and trailing edge handling
    ///
    /// - If not throttled: executes immediately with `msg`
    /// - If throttled: schedules trailing edge with `trailing_msg`
    ///
    /// When `trailing_msg` arrives, call `should_fire_trailing()` to check.
    ///
    /// Requires the `tokio` feature.
    #[cfg(feature = "tokio")]
    pub fn run<Msg>(&mut self, interval: Duration, msg: Msg, trailing_msg: Msg) -> Cmd<Msg>
    where
        Msg: Clone + Send + 'static,
    {
        let now = Instant::now();

        let should_run = match self.last_run {
            None => true,
            Some(last) => now.duration_since(last) >= interval,
        };

        if should_run {
            self.last_run = Some(now);
            self.has_pending = false;
            self.trailing_scheduled = false;
            Cmd::Msg(msg)
        } else {
            self.has_pending = true;

            // Schedule trailing if not already scheduled
            if !self.trailing_scheduled {
                self.trailing_scheduled = true;
                let remaining = self
                    .last_run
                    .map(|last| interval.saturating_sub(now.duration_since(last)))
                    .unwrap_or(interval);
                Cmd::delay(remaining, trailing_msg)
            } else {
                Cmd::none()
            }
        }
    }

    /// Mark a run without returning a Cmd (non-tokio version)
    ///
    /// Use this when you want to manage the delay yourself.
    /// Returns `Some(remaining_duration)` if trailing should be scheduled,
    /// `None` if executed immediately or trailing already scheduled.
    pub fn mark_run(&mut self, interval: Duration) -> Option<Duration> {
        let now = Instant::now();

        let should_run = match self.last_run {
            None => true,
            Some(last) => now.duration_since(last) >= interval,
        };

        if should_run {
            self.last_run = Some(now);
            self.has_pending = false;
            self.trailing_scheduled = false;
            None
        } else {
            self.has_pending = true;

            if !self.trailing_scheduled {
                self.trailing_scheduled = true;
                let remaining = self
                    .last_run
                    .map(|last| interval.saturating_sub(now.duration_since(last)))
                    .unwrap_or(interval);
                Some(remaining)
            } else {
                None
            }
        }
    }

    /// Check if trailing edge should fire
    ///
    /// Call this when the trailing message arrives.
    /// Returns `true` if there were suppressed calls during throttle.
    pub fn should_fire_trailing(&mut self) -> bool {
        if self.has_pending {
            self.has_pending = false;
            self.trailing_scheduled = false;
            self.last_run = Some(Instant::now());
            true
        } else {
            self.trailing_scheduled = false;
            false
        }
    }

    /// Reset the throttler state
    pub fn reset(&mut self) {
        self.last_run = None;
        self.has_pending = false;
        self.trailing_scheduled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_throttler_first_call() {
        let mut throttler = Throttler::new();

        // First call should not be throttled
        assert!(!throttler.is_throttled(Duration::from_millis(100)));

        let cmd = throttler.run(Duration::from_millis(100), || Cmd::Msg(42));
        assert!(cmd.is_msg());
    }

    #[test]
    fn test_throttler_blocks_rapid_calls() {
        let mut throttler = Throttler::new();
        let interval = Duration::from_millis(50);

        // First call executes
        let cmd1 = throttler.run(interval, || Cmd::Msg(1));
        assert!(cmd1.is_msg());

        // Immediate second call is throttled
        let cmd2 = throttler.run(interval, || Cmd::Msg(2));
        assert!(cmd2.is_none());

        // Still throttled
        thread::sleep(Duration::from_millis(20));
        let cmd3 = throttler.run(interval, || Cmd::Msg(3));
        assert!(cmd3.is_none());

        // After interval, executes again
        thread::sleep(Duration::from_millis(35));
        let cmd4 = throttler.run(interval, || Cmd::Msg(4));
        assert!(cmd4.is_msg());
    }

    #[test]
    fn test_throttler_reset() {
        let mut throttler = Throttler::new();
        let interval = Duration::from_millis(100);

        // First call
        let _ = throttler.run(interval, || Cmd::Msg(1));

        // Reset
        throttler.reset();

        // Next call should execute immediately
        let cmd = throttler.run(interval, || Cmd::Msg(2));
        assert!(cmd.is_msg());
    }

    #[test]
    fn test_throttler_time_remaining() {
        let mut throttler = Throttler::new();
        let interval = Duration::from_millis(100);

        // Before first run
        assert!(throttler.time_remaining(interval).is_none());

        // After first run
        let _ = throttler.run(interval, || Cmd::Msg(1));
        let remaining = throttler.time_remaining(interval);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= interval);
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_trailing_throttler_basic() {
        let mut throttler = TrailingThrottler::new();
        let interval = Duration::from_millis(50);

        // First call executes immediately
        let cmd1 = throttler.run(interval, 1, 100);
        assert!(cmd1.is_msg_eq(&1));

        // Second call schedules trailing
        let cmd2 = throttler.run(interval, 2, 100);
        assert!(!cmd2.is_none()); // Trailing scheduled

        // Third call doesn't schedule another trailing
        let cmd3 = throttler.run(interval, 3, 100);
        assert!(cmd3.is_none());

        // Wait and check trailing
        thread::sleep(Duration::from_millis(55));
        assert!(throttler.should_fire_trailing());
    }
}
