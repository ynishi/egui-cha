//! Debounce helper for TEA applications
//!
//! Debouncing delays action until input stops for a specified duration.
//! Useful for search inputs, form validation, auto-save, etc.

use super::clock::Clock;
use crate::Cmd;
use std::time::{Duration, Instant};

/// Debouncer - delays action until input stops
///
/// # How it works
/// Each call to `trigger()` resets the timer. The message is only
/// delivered after the specified delay has passed without any new triggers.
///
/// # Example
/// ```ignore
/// use egui_cha::helpers::Debouncer;
/// use std::time::Duration;
///
/// struct Model {
///     search_query: String,
///     search_debouncer: Debouncer,
/// }
///
/// enum Msg {
///     SearchInput(String),
///     DoSearch,           // Debounced trigger
///     SearchComplete,     // Actual search execution
/// }
///
/// fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
///     match msg {
///         Msg::SearchInput(text) => {
///             model.search_query = text;
///             // Returns Cmd::delay - resets on each keystroke
///             model.search_debouncer.trigger(
///                 Duration::from_millis(300),
///                 Msg::DoSearch,
///             )
///         }
///         Msg::DoSearch => {
///             // Only fire if this is the latest trigger
///             if model.search_debouncer.should_fire() {
///                 // Perform actual search
///                 Cmd::task(async { Msg::SearchComplete })
///             } else {
///                 Cmd::none()
///             }
///         }
///         _ => Cmd::none()
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Debouncer {
    pending_until: Option<Instant>,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new()
    }
}

impl Debouncer {
    /// Create a new debouncer
    pub fn new() -> Self {
        Self {
            pending_until: None,
        }
    }

    /// Trigger a debounced action
    ///
    /// Returns `Cmd::delay` that will deliver the message after the specified delay.
    /// Each call resets the internal timer, so rapid calls will keep delaying
    /// until input stops.
    ///
    /// When the delayed message arrives in `update()`, call `should_fire()`
    /// to check if this is the latest trigger.
    pub fn trigger<Msg>(&mut self, delay: Duration, msg: Msg) -> Cmd<Msg>
    where
        Msg: Clone + Send + 'static,
    {
        let fire_at = Instant::now() + delay;
        self.pending_until = Some(fire_at);
        Cmd::delay(delay, msg)
    }

    /// Check if the debounced action should fire
    ///
    /// Call this when the delayed message arrives to verify it's the latest trigger.
    /// Returns `true` if enough time has passed since the last `trigger()` call.
    ///
    /// This also clears the pending state if firing.
    pub fn should_fire(&mut self) -> bool {
        match self.pending_until {
            Some(until) if Instant::now() >= until => {
                self.pending_until = None;
                true
            }
            Some(_) => false, // Not yet time (newer trigger exists)
            None => false,    // No pending trigger
        }
    }

    /// Check if there's a pending debounce (without firing)
    pub fn is_pending(&self) -> bool {
        self.pending_until.is_some()
    }

    /// Cancel any pending debounced action
    ///
    /// The next delayed message will be ignored by `should_fire()`.
    pub fn cancel(&mut self) {
        self.pending_until = None;
    }

    /// Reset the debouncer state
    ///
    /// Same as `cancel()`, but semantically for cleanup.
    pub fn reset(&mut self) {
        self.pending_until = None;
    }
}

// ============================================
// DebouncerWithClock - testable version
// ============================================

/// A debouncer with pluggable clock for testing
///
/// Like [`Debouncer`], but uses a [`Clock`] trait for time access,
/// enabling deterministic testing with [`FakeClock`](crate::testing::FakeClock).
///
/// # Example
/// ```ignore
/// use egui_cha::testing::FakeClock;
/// use egui_cha::helpers::DebouncerWithClock;
/// use std::time::Duration;
///
/// let clock = FakeClock::new();
/// let mut debouncer = DebouncerWithClock::new(clock.clone());
///
/// debouncer.trigger(Duration::from_millis(500), Msg::Search);
/// assert!(!debouncer.should_fire()); // Not yet
///
/// clock.advance(Duration::from_millis(600));
/// assert!(debouncer.should_fire()); // Now it fires
/// ```
#[derive(Debug, Clone)]
pub struct DebouncerWithClock<C: Clock> {
    clock: C,
    pending_until: Option<Duration>,
}

impl<C: Clock> DebouncerWithClock<C> {
    /// Create a new debouncer with the given clock
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            pending_until: None,
        }
    }

    /// Trigger a debounced action
    ///
    /// Returns `Cmd::delay` that will deliver the message after the specified delay.
    /// Each call resets the internal timer.
    pub fn trigger<Msg>(&mut self, delay: Duration, msg: Msg) -> Cmd<Msg>
    where
        Msg: Clone + Send + 'static,
    {
        let fire_at = self.clock.now() + delay;
        self.pending_until = Some(fire_at);
        Cmd::delay(delay, msg)
    }

    /// Check if the debounced action should fire
    ///
    /// Returns `true` if enough time has passed since the last `trigger()` call.
    pub fn should_fire(&mut self) -> bool {
        match self.pending_until {
            Some(until) if self.clock.now() >= until => {
                self.pending_until = None;
                true
            }
            Some(_) => false,
            None => false,
        }
    }

    /// Check if there's a pending debounce
    pub fn is_pending(&self) -> bool {
        self.pending_until.is_some()
    }

    /// Cancel any pending debounced action
    pub fn cancel(&mut self) {
        self.pending_until = None;
    }

    /// Reset the debouncer state
    pub fn reset(&mut self) {
        self.pending_until = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_debouncer_basic() {
        let mut debouncer = Debouncer::new();

        // Trigger with short delay
        let _cmd = debouncer.trigger::<()>(Duration::from_millis(10), ());
        assert!(debouncer.is_pending());

        // Wait for delay
        thread::sleep(Duration::from_millis(15));
        assert!(debouncer.should_fire());
        assert!(!debouncer.is_pending());
    }

    #[test]
    fn test_debouncer_reset_on_trigger() {
        let mut debouncer = Debouncer::new();

        // First trigger
        let _cmd = debouncer.trigger::<()>(Duration::from_millis(50), ());

        // Wait partial time
        thread::sleep(Duration::from_millis(30));
        assert!(!debouncer.should_fire()); // Not yet

        // Trigger again (resets timer)
        let _cmd = debouncer.trigger::<()>(Duration::from_millis(50), ());

        // Wait partial time again
        thread::sleep(Duration::from_millis(30));
        assert!(!debouncer.should_fire()); // Still not yet (timer was reset)

        // Wait remaining time
        thread::sleep(Duration::from_millis(25));
        assert!(debouncer.should_fire()); // Now it fires
    }

    #[test]
    fn test_debouncer_cancel() {
        let mut debouncer = Debouncer::new();

        let _cmd = debouncer.trigger::<()>(Duration::from_millis(10), ());
        debouncer.cancel();

        thread::sleep(Duration::from_millis(15));
        assert!(!debouncer.should_fire()); // Cancelled, won't fire
    }

    #[test]
    fn test_debouncer_double_fire_protection() {
        let mut debouncer = Debouncer::new();

        let _cmd = debouncer.trigger::<()>(Duration::from_millis(10), ());
        thread::sleep(Duration::from_millis(15));

        assert!(debouncer.should_fire()); // First call fires
        assert!(!debouncer.should_fire()); // Second call doesn't
    }
}
