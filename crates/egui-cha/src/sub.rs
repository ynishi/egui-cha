//! Subscription type for continuous effects (intervals, timers, etc.)
//!
//! Unlike `Cmd` which represents one-shot effects, `Sub` represents
//! continuous subscriptions that persist across frames.
//!
//! # Example
//! ```ignore
//! use egui_cha::prelude::*;
//! use std::time::Duration;
//!
//! fn subscriptions(model: &Model) -> Sub<Msg> {
//!     if model.auto_refresh {
//!         Sub::interval("refresh", Duration::from_secs(30), Msg::Refresh)
//!     } else {
//!         Sub::none()
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::time::Duration;

/// A subscription representing a continuous effect
///
/// Subscriptions are declared each frame based on model state.
/// The runtime manages starting/stopping subscriptions as they
/// appear or disappear from the returned `Sub`.
#[derive(Clone)]
pub enum Sub<Msg> {
    /// No subscription
    None,

    /// Multiple subscriptions
    Batch(Vec<Sub<Msg>>),

    /// Periodic interval timer
    ///
    /// Emits the message at regular intervals.
    /// The `id` must be unique - subscriptions with the same ID
    /// are considered identical and won't be restarted.
    Interval {
        /// Unique identifier for this interval
        id: &'static str,
        /// Time between emissions
        duration: Duration,
        /// Message to emit
        msg: Msg,
    },
}

impl<Msg> Default for Sub<Msg> {
    fn default() -> Self {
        Sub::None
    }
}

impl<Msg: Clone> Sub<Msg> {
    /// Create an empty subscription
    #[inline]
    pub fn none() -> Self {
        Sub::None
    }

    /// Create a batch of subscriptions
    pub fn batch(subs: impl IntoIterator<Item = Sub<Msg>>) -> Self {
        let subs: Vec<_> = subs.into_iter().collect();
        if subs.is_empty() {
            Sub::None
        } else if subs.len() == 1 {
            subs.into_iter().next().unwrap()
        } else {
            Sub::Batch(subs)
        }
    }

    /// Create a periodic interval subscription
    ///
    /// The message will be emitted every `duration` after the subscription starts.
    ///
    /// # Arguments
    /// - `id`: Unique identifier. Intervals with the same ID won't restart.
    /// - `duration`: Time between message emissions
    /// - `msg`: Message to emit each interval
    ///
    /// # Example
    /// ```ignore
    /// // Auto-save every 30 seconds when enabled
    /// fn subscriptions(model: &Model) -> Sub<Msg> {
    ///     if model.auto_save_enabled {
    ///         Sub::interval("auto_save", Duration::from_secs(30), Msg::AutoSave)
    ///     } else {
    ///         Sub::none()
    ///     }
    /// }
    /// ```
    pub fn interval(id: &'static str, duration: Duration, msg: Msg) -> Self {
        Sub::Interval { id, duration, msg }
    }

    /// Collect all interval IDs in this subscription tree
    pub(crate) fn collect_interval_ids(&self, ids: &mut HashSet<&'static str>) {
        match self {
            Sub::None => {}
            Sub::Batch(subs) => {
                for sub in subs {
                    sub.collect_interval_ids(ids);
                }
            }
            Sub::Interval { id, .. } => {
                ids.insert(id);
            }
        }
    }

    /// Iterate over all intervals in this subscription
    pub(crate) fn intervals(&self) -> Vec<(&'static str, Duration, Msg)> {
        let mut result = Vec::new();
        self.collect_intervals(&mut result);
        result
    }

    fn collect_intervals(&self, result: &mut Vec<(&'static str, Duration, Msg)>) {
        match self {
            Sub::None => {}
            Sub::Batch(subs) => {
                for sub in subs {
                    sub.collect_intervals(result);
                }
            }
            Sub::Interval { id, duration, msg } => {
                result.push((id, *duration, msg.clone()));
            }
        }
    }
}

// ============================================================
// Test helpers
// ============================================================

impl<Msg> Sub<Msg> {
    /// Check if this is Sub::None
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Sub::None)
    }

    /// Check if this is Sub::Interval
    #[inline]
    pub fn is_interval(&self) -> bool {
        matches!(self, Sub::Interval { .. })
    }

    /// Check if this is Sub::Batch
    #[inline]
    pub fn is_batch(&self) -> bool {
        matches!(self, Sub::Batch(_))
    }

    /// Get the number of subscriptions (1 for single, n for batch, 0 for none)
    pub fn len(&self) -> usize {
        match self {
            Sub::None => 0,
            Sub::Batch(subs) => subs.iter().map(|s| s.len()).sum(),
            Sub::Interval { .. } => 1,
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_none() {
        let sub: Sub<i32> = Sub::none();
        assert!(sub.is_none());
        assert_eq!(sub.len(), 0);
    }

    #[test]
    fn test_sub_interval() {
        let sub = Sub::interval("test", Duration::from_secs(1), 42);
        assert!(sub.is_interval());
        assert_eq!(sub.len(), 1);
    }

    #[test]
    fn test_sub_batch() {
        let sub = Sub::batch([
            Sub::interval("a", Duration::from_secs(1), 1),
            Sub::interval("b", Duration::from_secs(2), 2),
        ]);
        assert!(sub.is_batch());
        assert_eq!(sub.len(), 2);
    }

    #[test]
    fn test_collect_interval_ids() {
        let sub = Sub::batch([
            Sub::interval("a", Duration::from_secs(1), 1),
            Sub::interval("b", Duration::from_secs(2), 2),
            Sub::none(),
        ]);

        let mut ids = HashSet::new();
        sub.collect_interval_ids(&mut ids);

        assert!(ids.contains("a"));
        assert!(ids.contains("b"));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_batch_flattening() {
        // Empty batch becomes None
        let empty: Sub<i32> = Sub::batch([]);
        assert!(empty.is_none());

        // Single-item batch unwraps
        let single = Sub::batch([Sub::interval("x", Duration::from_secs(1), 1)]);
        assert!(single.is_interval());
    }

    #[test]
    fn test_intervals_iterator() {
        let sub = Sub::batch([
            Sub::interval("a", Duration::from_secs(1), 10),
            Sub::interval("b", Duration::from_secs(2), 20),
        ]);

        let intervals = sub.intervals();
        assert_eq!(intervals.len(), 2);
        assert_eq!(intervals[0], ("a", Duration::from_secs(1), 10));
        assert_eq!(intervals[1], ("b", Duration::from_secs(2), 20));
    }
}
