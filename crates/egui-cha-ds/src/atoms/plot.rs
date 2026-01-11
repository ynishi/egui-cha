//! Plot wrappers for egui_plot with theme integration
//!
//! Provides themed plot components for music/audio applications.
//!
//! # Example
//! ```ignore
//! // Simple line plot
//! LinePlot::new("waveform", &samples)
//!     .show(ctx.ui);
//!
//! // Envelope plot (ADSR)
//! EnvelopePlot::new(&envelope_points)
//!     .show(ctx.ui);
//! ```

use crate::Theme;
use egui::{Color32, Ui, Vec2};
use egui_plot::{Line, Plot, PlotPoints};

/// A simple line plot with theme integration
pub struct LinePlot<'a> {
    id: &'a str,
    points: &'a [f64],
    size: Vec2,
    color: Option<Color32>,
    fill: bool,
    show_axes: bool,
    show_grid: bool,
}

impl<'a> LinePlot<'a> {
    /// Create a new line plot
    pub fn new(id: &'a str, points: &'a [f64]) -> Self {
        Self {
            id,
            points,
            size: Vec2::new(200.0, 100.0),
            color: None,
            fill: false,
            show_axes: false,
            show_grid: true,
        }
    }

    /// Set the plot size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set custom color (defaults to theme primary)
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Fill area under the line
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Show/hide axes labels
    pub fn show_axes(mut self, show: bool) -> Self {
        self.show_axes = show;
        self
    }

    /// Show/hide grid
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Show the plot
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let line_color = self.color.unwrap_or(theme.primary);

        let plot_points: PlotPoints = self
            .points
            .iter()
            .enumerate()
            .map(|(i, &y)| [i as f64, y])
            .collect();

        let mut line = Line::new(self.id, plot_points).color(line_color).width(1.5);

        if self.fill {
            line = line.fill(0.0);
        }

        Plot::new(self.id)
            .height(self.size.y)
            .width(self.size.x)
            .show_axes(self.show_axes)
            .show_grid(self.show_grid)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_background(false)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }
}

/// Envelope plot for ADSR and similar curves
pub struct EnvelopePlot<'a> {
    id: &'a str,
    /// Points as (time, value) pairs, normalized 0-1
    points: &'a [(f64, f64)],
    size: Vec2,
    color: Option<Color32>,
    editable: bool,
}

impl<'a> EnvelopePlot<'a> {
    /// Create a new envelope plot
    pub fn new(id: &'a str, points: &'a [(f64, f64)]) -> Self {
        Self {
            id,
            points,
            size: Vec2::new(200.0, 80.0),
            color: None,
            editable: false,
        }
    }

    /// Set the plot size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set custom color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Make the envelope editable (future feature)
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Show the plot
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let line_color = self.color.unwrap_or(theme.state_success);

        let plot_points: PlotPoints = self.points.iter().map(|&(x, y)| [x, y]).collect();

        let line = Line::new(self.id, plot_points)
            .color(line_color)
            .width(2.0)
            .fill(0.0);

        Plot::new(self.id)
            .height(self.size.y)
            .width(self.size.x)
            .show_axes(false)
            .show_grid(false)
            .allow_zoom(false)
            .allow_drag(self.editable)
            .allow_scroll(false)
            .show_background(false)
            .include_y(0.0)
            .include_y(1.0)
            .include_x(0.0)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }
}

/// Automation curve plot for DAW-style parameter automation
pub struct AutomationPlot<'a> {
    id: &'a str,
    /// Points as (time, value) pairs
    points: &'a [(f64, f64)],
    size: Vec2,
    color: Option<Color32>,
    range: (f64, f64),
    show_points: bool,
}

impl<'a> AutomationPlot<'a> {
    /// Create a new automation plot
    pub fn new(id: &'a str, points: &'a [(f64, f64)]) -> Self {
        Self {
            id,
            points,
            size: Vec2::new(300.0, 60.0),
            color: None,
            range: (0.0, 1.0),
            show_points: true,
        }
    }

    /// Set the plot size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set custom color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Set value range
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = (min, max);
        self
    }

    /// Show/hide control points
    pub fn show_points(mut self, show: bool) -> Self {
        self.show_points = show;
        self
    }

    /// Show the plot
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let line_color = self.color.unwrap_or(theme.state_warning);

        let plot_points: PlotPoints = self.points.iter().map(|&(x, y)| [x, y]).collect();

        let line = Line::new(self.id, plot_points).color(line_color).width(1.5);

        Plot::new(self.id)
            .height(self.size.y)
            .width(self.size.x)
            .show_axes(false)
            .show_grid(true)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_background(false)
            .include_y(self.range.0)
            .include_y(self.range.1)
            .show(ui, |plot_ui| {
                plot_ui.line(line);

                if self.show_points {
                    let point_data: PlotPoints = self.points.iter().map(|&(x, y)| [x, y]).collect();
                    let points = egui_plot::Points::new(format!("{}_points", self.id), point_data)
                        .radius(4.0)
                        .color(line_color);
                    plot_ui.points(points);
                }
            });
    }
}

/// Frequency response plot for EQ curves
pub struct FrequencyPlot<'a> {
    id: &'a str,
    /// Points as (frequency_hz, gain_db) pairs
    points: &'a [(f64, f64)],
    size: Vec2,
    color: Option<Color32>,
    log_scale: bool,
    db_range: (f64, f64),
}

impl<'a> FrequencyPlot<'a> {
    /// Create a new frequency response plot
    pub fn new(id: &'a str, points: &'a [(f64, f64)]) -> Self {
        Self {
            id,
            points,
            size: Vec2::new(300.0, 120.0),
            color: None,
            log_scale: true,
            db_range: (-24.0, 24.0),
        }
    }

    /// Set the plot size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set custom color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Use logarithmic frequency scale (default: true)
    pub fn log_scale(mut self, log: bool) -> Self {
        self.log_scale = log;
        self
    }

    /// Set dB range
    pub fn db_range(mut self, min: f64, max: f64) -> Self {
        self.db_range = (min, max);
        self
    }

    /// Show the plot
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let line_color = self.color.unwrap_or(theme.primary);

        // Convert to log scale if needed
        let plot_points: PlotPoints = if self.log_scale {
            self.points
                .iter()
                .map(|&(freq, db)| [freq.log10(), db])
                .collect()
        } else {
            self.points.iter().map(|&(x, y)| [x, y]).collect()
        };

        let line = Line::new(self.id, plot_points)
            .color(line_color)
            .width(2.0)
            .fill(0.0);

        // Draw 0dB reference line
        let zero_line = Line::new(
            format!("{}_zero", self.id),
            PlotPoints::from_iter([[1.0_f64.log10(), 0.0], [20000.0_f64.log10(), 0.0]]),
        )
        .color(theme.border)
        .width(1.0);

        Plot::new(self.id)
            .height(self.size.y)
            .width(self.size.x)
            .show_axes(false)
            .show_grid(true)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_background(false)
            .include_y(self.db_range.0)
            .include_y(self.db_range.1)
            .show(ui, |plot_ui| {
                plot_ui.line(zero_line);
                plot_ui.line(line);
            });
    }
}

/// Bar chart plot
pub struct BarPlot<'a> {
    id: &'a str,
    values: &'a [f64],
    size: Vec2,
    color: Option<Color32>,
    bar_width: f64,
}

impl<'a> BarPlot<'a> {
    /// Create a new bar plot
    pub fn new(id: &'a str, values: &'a [f64]) -> Self {
        Self {
            id,
            values,
            size: Vec2::new(200.0, 100.0),
            color: None,
            bar_width: 0.8,
        }
    }

    /// Set the plot size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set custom color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Set bar width (0.0 - 1.0)
    pub fn bar_width(mut self, width: f64) -> Self {
        self.bar_width = width;
        self
    }

    /// Show the plot
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let bar_color = self.color.unwrap_or(theme.primary);

        let bars: Vec<egui_plot::Bar> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                egui_plot::Bar::new(i as f64, v)
                    .width(self.bar_width)
                    .fill(bar_color)
            })
            .collect();

        let chart = egui_plot::BarChart::new(self.id, bars);

        Plot::new(self.id)
            .height(self.size.y)
            .width(self.size.x)
            .show_axes(false)
            .show_grid(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_background(false)
            .include_y(0.0)
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
            });
    }
}

/// Compact inline chart for time-series data
///
/// Sparklines are word-sized graphics showing trends without axes or labels.
/// Ideal for dashboards, tables, and inline metrics display.
///
/// # Example
/// ```ignore
/// // Simple usage
/// Sparkline::new(&cpu_history).height(24.0).show(ui);
///
/// // With options
/// Sparkline::new(&latency_data)
///     .height(32.0)
///     .color(theme.state_warning)
///     .fill(true)
///     .bounds(0.0, 100.0)
///     .show_current(true)
///     .show(ui);
/// ```
pub struct Sparkline<'a> {
    data: &'a [f32],
    height: Option<f32>,
    width: Option<f32>,
    color: Option<Color32>,
    fill: bool,
    bounds: Option<(f32, f32)>,
    show_current: bool,
    highlight_extremes: bool,
}

impl<'a> Sparkline<'a> {
    /// Create a new sparkline from data
    pub fn new(data: &'a [f32]) -> Self {
        Self {
            data,
            height: None,
            width: None,
            color: None,
            fill: false,
            bounds: None,
            show_current: false,
            highlight_extremes: false,
        }
    }

    /// Set sparkline height (default: theme.spacing_md + theme.spacing_xs)
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set fixed width (default: fill available space)
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set line color (default: theme.primary)
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Fill area under the line (area chart)
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Set fixed Y-axis bounds (default: auto-scale to data)
    pub fn bounds(mut self, min: f32, max: f32) -> Self {
        self.bounds = Some((min, max));
        self
    }

    /// Show current (latest) value as label
    pub fn show_current(mut self, show: bool) -> Self {
        self.show_current = show;
        self
    }

    /// Highlight min/max points
    pub fn highlight_extremes(mut self, show: bool) -> Self {
        self.highlight_extremes = show;
        self
    }

    /// Show the sparkline
    pub fn show(self, ui: &mut Ui) {
        if self.data.is_empty() {
            return;
        }

        let theme = Theme::current(ui.ctx());
        let height = self.height.unwrap_or(theme.spacing_md + theme.spacing_xs);
        let line_color = self.color.unwrap_or(theme.primary);

        // Calculate bounds
        let (min_y, max_y) = if let Some((min, max)) = self.bounds {
            (min as f64, max as f64)
        } else {
            let min = self.data.iter().cloned().fold(f32::MAX, f32::min) as f64;
            let max = self.data.iter().cloned().fold(f32::MIN, f32::max) as f64;
            let padding = (max - min) * 0.1;
            (min - padding, max + padding)
        };

        // Convert to plot points
        let plot_points: PlotPoints = self
            .data
            .iter()
            .enumerate()
            .map(|(i, &y)| [i as f64, y as f64])
            .collect();

        let mut line = Line::new("sparkline", plot_points)
            .color(line_color)
            .width(1.5);

        if self.fill {
            line = line.fill(min_y as f32);
        }

        // Create plot
        let mut plot = Plot::new(ui.auto_id_with("sparkline"))
            .height(height)
            .show_axes(false)
            .show_grid(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_background(false)
            .include_y(min_y)
            .include_y(max_y);

        if let Some(w) = self.width {
            plot = plot.width(w);
        }

        let response = plot.show(ui, |plot_ui| {
            plot_ui.line(line);

            // Highlight extremes
            if self.highlight_extremes && self.data.len() > 1 {
                let (min_idx, min_val) = self
                    .data
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap();
                let (max_idx, max_val) = self
                    .data
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap();

                let min_point = egui_plot::Points::new(
                    "sparkline_min",
                    PlotPoints::from_iter([[min_idx as f64, *min_val as f64]]),
                )
                .radius(3.0)
                .color(theme.state_danger);

                let max_point = egui_plot::Points::new(
                    "sparkline_max",
                    PlotPoints::from_iter([[max_idx as f64, *max_val as f64]]),
                )
                .radius(3.0)
                .color(theme.state_success);

                plot_ui.points(min_point);
                plot_ui.points(max_point);
            }
        });

        // Show current value label
        if self.show_current {
            if let Some(&current) = self.data.last() {
                let text = if current.abs() >= 100.0 {
                    format!("{:.0}", current)
                } else if current.abs() >= 10.0 {
                    format!("{:.1}", current)
                } else {
                    format!("{:.2}", current)
                };
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(text)
                            .size(theme.font_size_xs)
                            .color(theme.text_secondary),
                    );
                });
            }
        }

        // Enable hover tooltip showing value at position
        if let Some(pos) = response.response.hover_pos() {
            let plot_rect = response.response.rect;
            let x_ratio = (pos.x - plot_rect.left()) / plot_rect.width();
            let idx = ((x_ratio * self.data.len() as f32) as usize)
                .min(self.data.len().saturating_sub(1));
            let value = self.data[idx];
            response.response.on_hover_ui_at_pointer(|ui| {
                ui.label(format!("{:.2}", value));
            });
        }
    }
}

/// Re-export egui_plot for advanced usage
pub mod raw {
    pub use egui_plot::*;
}
