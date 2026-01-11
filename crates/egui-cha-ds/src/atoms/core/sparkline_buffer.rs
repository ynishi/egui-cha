//! Ring buffer for time-series data
//!
//! A fixed-capacity buffer that automatically removes oldest values when full.
//! Useful for storing streaming data like CPU usage, latency, throughput.
//!
//! # Example
//! ```ignore
//! let mut buffer = SparklineBuffer::new(100);
//!
//! // Push values (oldest auto-removed when full)
//! buffer.push(45.0);
//! buffer.push(52.0);
//!
//! // Use with Sparkline
//! Sparkline::new(&buffer.as_vec()).show(ui);
//! ```

use std::collections::VecDeque;

/// Fixed-capacity ring buffer for time-series data
#[derive(Debug, Clone)]
pub struct SparklineBuffer {
    data: VecDeque<f32>,
    capacity: usize,
}

impl SparklineBuffer {
    /// Create a new buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Create a buffer pre-filled with a default value
    pub fn filled(capacity: usize, value: f32) -> Self {
        let mut data = VecDeque::with_capacity(capacity);
        data.resize(capacity, value);
        Self { data, capacity }
    }

    /// Push a new value, removing the oldest if at capacity
    pub fn push(&mut self, value: f32) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Push multiple values at once
    pub fn extend(&mut self, values: impl IntoIterator<Item = f32>) {
        for value in values {
            self.push(value);
        }
    }

    /// Get a contiguous Vec copy (for APIs that need &[f32])
    pub fn as_vec(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }

    /// Get slices (may be split due to ring buffer)
    pub fn as_slices(&self) -> (&[f32], &[f32]) {
        self.data.as_slices()
    }

    /// Number of values currently stored
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Maximum capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all values
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the most recent value
    pub fn last(&self) -> Option<f32> {
        self.data.back().copied()
    }

    /// Get the oldest value
    pub fn first(&self) -> Option<f32> {
        self.data.front().copied()
    }

    /// Get min and max values (useful for auto-scaling)
    pub fn min_max(&self) -> Option<(f32, f32)> {
        if self.data.is_empty() {
            return None;
        }
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for &v in &self.data {
            min = min.min(v);
            max = max.max(v);
        }
        Some((min, max))
    }

    /// Calculate average of all values
    pub fn average(&self) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f32 = self.data.iter().sum();
        Some(sum / self.data.len() as f32)
    }

    /// Iterate over values (oldest to newest)
    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.data.iter().copied()
    }
}

impl Default for SparklineBuffer {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_capacity() {
        let mut buf = SparklineBuffer::new(3);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        assert_eq!(buf.as_vec(), vec![1.0, 2.0, 3.0]);

        // Should remove oldest
        buf.push(4.0);
        assert_eq!(buf.as_vec(), vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_min_max() {
        let mut buf = SparklineBuffer::new(5);
        buf.extend([10.0, 5.0, 20.0, 15.0]);
        assert_eq!(buf.min_max(), Some((5.0, 20.0)));
    }

    #[test]
    fn test_average() {
        let mut buf = SparklineBuffer::new(4);
        buf.extend([10.0, 20.0, 30.0, 40.0]);
        assert_eq!(buf.average(), Some(25.0));
    }
}
