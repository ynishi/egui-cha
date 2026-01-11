//! Capacity gauge for resource utilization visualization
//!
//! A progress bar that automatically changes color based on utilization thresholds.
//! Useful for showing resource capacity, worker pools, memory usage, etc.
//!
//! # Example
//! ```ignore
//! // Simple usage with automatic coloring
//! CapacityGauge::new(0.85)
//!     .show(ui);
//!
//! // With custom thresholds and fraction display
//! CapacityGauge::new(799.0 / 1000.0)
//!     .thresholds(0.7, 0.9)  // warning at 70%, danger at 90%
//!     .label("Workers")
//!     .show_fraction(799, 1000)
//!     .show(ui);
//!
//! // TEA style
//! CapacityGauge::new(model.cpu_usage)
//!     .show_percentage()
//!     .show_with(ctx, || Msg::CapacityClicked);
//! ```

use crate::Theme;
use egui::{Response, Ui, Widget};
use egui_cha::ViewCtx;

/// Capacity gauge with threshold-based coloring
pub struct CapacityGauge {
    /// Current value (0.0 - 1.0)
    progress: f32,
    /// Warning threshold (default: 0.7)
    warning_threshold: f32,
    /// Danger threshold (default: 0.9)
    danger_threshold: f32,
    /// Optional label text
    label: Option<String>,
    /// Display mode for the value
    display: DisplayMode,
    /// Desired width (None = fill available)
    width: Option<f32>,
    /// Desired height (None = theme default)
    height: Option<f32>,
    /// Whether to animate when not full
    animate: bool,
}

/// How to display the value on the gauge
#[derive(Debug, Clone)]
enum DisplayMode {
    /// No text display
    None,
    /// Show percentage (e.g., "85%")
    Percentage,
    /// Show fraction (e.g., "799 / 1000")
    Fraction { current: u64, total: u64 },
    /// Custom text
    Custom(String),
}

impl CapacityGauge {
    /// Create a new capacity gauge with the given progress (0.0 - 1.0)
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            warning_threshold: 0.7,
            danger_threshold: 0.9,
            label: None,
            display: DisplayMode::None,
            width: None,
            height: None,
            animate: false,
        }
    }

    /// Create from current/total values
    pub fn from_fraction(current: u64, total: u64) -> Self {
        let progress = if total == 0 {
            0.0
        } else {
            current as f32 / total as f32
        };
        Self::new(progress).show_fraction(current, total)
    }

    /// Set warning and danger thresholds (0.0 - 1.0)
    ///
    /// - Below warning: success color (green)
    /// - Between warning and danger: warning color (yellow/orange)
    /// - Above danger: danger color (red)
    pub fn thresholds(mut self, warning: f32, danger: f32) -> Self {
        self.warning_threshold = warning.clamp(0.0, 1.0);
        self.danger_threshold = danger.clamp(0.0, 1.0);
        self
    }

    /// Set a label to display before the gauge
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Show percentage on the gauge (e.g., "85%")
    pub fn show_percentage(mut self) -> Self {
        self.display = DisplayMode::Percentage;
        self
    }

    /// Show fraction on the gauge (e.g., "799 / 1000")
    pub fn show_fraction(mut self, current: u64, total: u64) -> Self {
        self.display = DisplayMode::Fraction { current, total };
        self
    }

    /// Show custom text on the gauge
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.display = DisplayMode::Custom(text.into());
        self
    }

    /// Set the width of the gauge (default: fill available space)
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height of the gauge (default: theme.spacing_md)
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Enable animation when progress < 1.0
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Show the gauge (egui style)
    pub fn show(self, ui: &mut Ui) -> Response {
        self.show_internal(ui)
    }

    /// Show the gauge with click handler (TEA style)
    pub fn show_with<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, on_click: impl FnOnce() -> Msg) {
        let response = self.show_internal(ctx.ui);
        if response.clicked() {
            ctx.emit(on_click());
        }
    }

    fn show_internal(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());

        // Determine color based on thresholds
        let fill_color = if self.progress >= self.danger_threshold {
            theme.state_danger
        } else if self.progress >= self.warning_threshold {
            theme.state_warning
        } else {
            theme.state_success
        };

        // Build display text
        let text = match self.display {
            DisplayMode::None => None,
            DisplayMode::Percentage => Some(format!("{:.0}%", self.progress * 100.0)),
            DisplayMode::Fraction { current, total } => Some(format!("{} / {}", current, total)),
            DisplayMode::Custom(s) => Some(s),
        };

        // Get height from theme if not specified
        let height = self.height.unwrap_or(theme.spacing_md);

        // Build the progress bar
        let mut bar = egui::ProgressBar::new(self.progress)
            .fill(fill_color)
            .desired_height(height)
            .animate(self.animate);

        if let Some(w) = self.width {
            bar = bar.desired_width(w);
        }

        if let Some(t) = text {
            bar = bar.text(t);
        }

        // Build the gauge with optional label
        if let Some(label_text) = self.label {
            ui.horizontal(|ui| {
                ui.label(&label_text);
                ui.add(bar)
            })
            .inner
        } else {
            ui.add(bar)
        }
    }
}

impl Widget for CapacityGauge {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fraction() {
        let gauge = CapacityGauge::from_fraction(750, 1000);
        assert!((gauge.progress - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_from_fraction_zero_total() {
        let gauge = CapacityGauge::from_fraction(100, 0);
        assert_eq!(gauge.progress, 0.0);
    }

    #[test]
    fn test_progress_clamping() {
        let gauge = CapacityGauge::new(1.5);
        assert_eq!(gauge.progress, 1.0);

        let gauge = CapacityGauge::new(-0.5);
        assert_eq!(gauge.progress, 0.0);
    }
}
