//! Waveform atom - Audio waveform visualization for EDM/VJ applications
//!
//! Displays audio sample data as a waveform. Uses egui_plot internally for
//! smooth, anti-aliased rendering at 60fps.
//!
//! # Features
//! - Line or filled waveform styles (smooth via egui_plot)
//! - Bars style for discrete visualization
//! - Configurable height and color
//! - Theme-aware styling
//! - Supports mono and stereo display
//!
//! # Example
//! ```ignore
//! // Basic waveform
//! Waveform::new(&audio_samples)
//!     .show(ui);
//!
//! // With custom styling
//! Waveform::new(&samples)
//!     .height(80.0)
//!     .filled()
//!     .color(theme.primary)
//!     .show(ui);
//!
//! // Stereo waveform
//! Waveform::stereo(&left_samples, &right_samples)
//!     .show(ui);
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

#[cfg(feature = "plot")]
use egui_plot::{Line, Plot, PlotPoints};

/// Waveform display style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaveformStyle {
    /// Line connecting sample points (smooth, uses egui_plot)
    #[default]
    Line,
    /// Filled area from center line (smooth, uses egui_plot)
    Filled,
    /// Bars for each sample (discrete, custom drawing)
    Bars,
}

/// A waveform visualization component
pub struct Waveform<'a> {
    samples: &'a [f32],
    samples_right: Option<&'a [f32]>,
    height: Option<f32>,
    style: WaveformStyle,
    color: Option<Color32>,
    color_right: Option<Color32>,
    show_center_line: bool,
    show_grid: bool,
    line_width: f32,
}

impl<'a> Waveform<'a> {
    /// Create a new waveform from sample data
    ///
    /// Samples should be normalized to -1.0..1.0 range
    pub fn new(samples: &'a [f32]) -> Self {
        Self {
            samples,
            samples_right: None,
            height: None,
            style: WaveformStyle::default(),
            color: None,
            color_right: None,
            show_center_line: true,
            show_grid: false,
            line_width: 1.5,
        }
    }

    /// Create a stereo waveform (top: left, bottom: right)
    pub fn stereo(left: &'a [f32], right: &'a [f32]) -> Self {
        Self {
            samples: left,
            samples_right: Some(right),
            height: None,
            style: WaveformStyle::default(),
            color: None,
            color_right: None,
            show_center_line: true,
            show_grid: false,
            line_width: 1.5,
        }
    }

    /// Set the height (default: uses theme spacing)
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set display style
    pub fn style(mut self, style: WaveformStyle) -> Self {
        self.style = style;
        self
    }

    /// Use filled style
    pub fn filled(mut self) -> Self {
        self.style = WaveformStyle::Filled;
        self
    }

    /// Use bars style
    pub fn bars(mut self) -> Self {
        self.style = WaveformStyle::Bars;
        self
    }

    /// Set waveform color (default: theme.primary)
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Set right channel color for stereo (default: theme.secondary)
    pub fn color_right(mut self, color: Color32) -> Self {
        self.color_right = Some(color);
        self
    }

    /// Show/hide center line
    pub fn center_line(mut self, show: bool) -> Self {
        self.show_center_line = show;
        self
    }

    /// Show/hide background grid
    pub fn grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Set line width (default: 1.5)
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Show the waveform
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate dimensions
        let height = self.height.unwrap_or(theme.spacing_xl * 2.0);
        let total_height = if self.samples_right.is_some() {
            height * 2.0 + theme.spacing_xs
        } else {
            height
        };
        let width = ui.available_width();

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, total_height),
            Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            // Colors
            let primary_color = self.color.unwrap_or(theme.primary);
            let secondary_color = self.color_right.unwrap_or(theme.secondary);

            if self.samples_right.is_some() {
                // Stereo: draw two waveforms
                let left_rect = Rect::from_min_size(
                    rect.min,
                    Vec2::new(width, height),
                );
                let right_rect = Rect::from_min_size(
                    rect.min + Vec2::new(0.0, height + theme.spacing_xs),
                    Vec2::new(width, height),
                );

                self.draw_waveform_in_rect(ui, left_rect, self.samples, primary_color, &theme);

                if let Some(right_samples) = self.samples_right {
                    self.draw_waveform_in_rect(ui, right_rect, right_samples, secondary_color, &theme);
                }
            } else {
                // Mono: single waveform
                self.draw_waveform_in_rect(ui, rect, self.samples, primary_color, &theme);
            }
        }

        response
    }

    fn draw_waveform_in_rect(
        &self,
        ui: &mut Ui,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        theme: &Theme,
    ) {
        if samples.is_empty() {
            return;
        }

        // For Bars style, use custom drawing
        if self.style == WaveformStyle::Bars {
            self.draw_bars_waveform_custom(ui, rect, samples, color, theme);
            return;
        }

        // For Line and Filled styles, use egui_plot
        #[cfg(feature = "plot")]
        {
            self.draw_plot_waveform(ui, rect, samples, color, theme);
        }

        #[cfg(not(feature = "plot"))]
        {
            // Fallback to custom drawing if plot feature is not enabled
            self.draw_line_waveform_custom(ui, rect, samples, color, theme);
        }
    }

    #[cfg(feature = "plot")]
    fn draw_plot_waveform(
        &self,
        ui: &mut Ui,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        theme: &Theme,
    ) {
        // Draw background first
        ui.painter().rect_filled(rect, theme.radius_sm, theme.bg_secondary);

        // Convert samples to plot points
        let plot_points: PlotPoints = samples
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let x = i as f64 / samples.len().max(1) as f64;
                let y = sample.clamp(-1.0, 1.0) as f64;
                [x, y]
            })
            .collect();

        // Use unique ID based on rect position
        let plot_id_str = format!("waveform_{}_{}", rect.min.x as i32, rect.min.y as i32);

        let mut line = Line::new(&plot_id_str, plot_points)
            .color(color)
            .width(self.line_width);

        if self.style == WaveformStyle::Filled {
            line = line.fill(0.0);
        }

        // Clone values needed in closure
        let show_center_line = self.show_center_line;
        let center_line_color = theme.text_muted;

        // Create a child UI positioned at rect
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        Plot::new(&plot_id_str)
            .height(rect.height())
            .width(rect.width())
            .show_axes(false)
            .show_grid(self.show_grid)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .show_background(false)
            .include_y(-1.0)
            .include_y(1.0)
            .include_x(0.0)
            .include_x(1.0)
            .show(&mut child_ui, |plot_ui| {
                // Center line
                if show_center_line {
                    let center_line_id = format!("{}_center", plot_id_str);
                    let center_line = Line::new(
                        center_line_id,
                        PlotPoints::from_iter([[0.0, 0.0], [1.0, 0.0]]),
                    )
                    .color(center_line_color)
                    .width(0.5);
                    plot_ui.line(center_line);
                }

                plot_ui.line(line);
            });

        // Border (drawn after plot)
        ui.painter().rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Outside,
        );
    }

    #[cfg(not(feature = "plot"))]
    fn draw_line_waveform_custom(
        &self,
        ui: &mut Ui,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        theme: &Theme,
    ) {
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_sm, theme.bg_secondary);

        let center_y = rect.center().y;
        let half_height = rect.height() / 2.0 - theme.spacing_xs;

        // Center line
        if self.show_center_line {
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, center_y),
                    Pos2::new(rect.max.x, center_y),
                ],
                Stroke::new(theme.stroke_width * 0.5, theme.text_muted),
            );
        }

        // Draw waveform
        let stroke = Stroke::new(self.line_width, color);
        let step = rect.width() / samples.len().max(1) as f32;

        let points: Vec<Pos2> = samples
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let x = rect.min.x + step * i as f32 + step / 2.0;
                let y = center_y - sample.clamp(-1.0, 1.0) * half_height;
                Pos2::new(x, y)
            })
            .collect();

        for window in points.windows(2) {
            painter.line_segment([window[0], window[1]], stroke);
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Outside,
        );
    }

    fn draw_bars_waveform_custom(
        &self,
        ui: &mut Ui,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        theme: &Theme,
    ) {
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_sm, theme.bg_secondary);

        let center_y = rect.center().y;
        let half_height = rect.height() / 2.0 - theme.spacing_xs;

        // Center line
        if self.show_center_line {
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, center_y),
                    Pos2::new(rect.max.x, center_y),
                ],
                Stroke::new(theme.stroke_width * 0.5, theme.text_muted),
            );
        }

        let num_bars = samples.len().min(64); // Max 64 bars
        let samples_per_bar = samples.len() / num_bars.max(1);
        let bar_width = rect.width() / num_bars as f32;
        let gap = theme.spacing_xs * 0.5;

        for i in 0..num_bars {
            // Average samples for this bar
            let start = i * samples_per_bar;
            let end = ((i + 1) * samples_per_bar).min(samples.len());
            let slice = &samples[start..end];

            if slice.is_empty() {
                continue;
            }

            // Use RMS for better visual representation
            let rms: f32 = (slice.iter().map(|s| s * s).sum::<f32>() / slice.len() as f32).sqrt();
            let bar_height = rms.clamp(0.0, 1.0) * half_height;

            let x = rect.min.x + bar_width * i as f32 + gap / 2.0;
            let bar_rect = Rect::from_min_max(
                Pos2::new(x, center_y - bar_height),
                Pos2::new(x + bar_width - gap, center_y + bar_height),
            );

            painter.rect_filled(bar_rect, theme.radius_sm * 0.5, color);
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Outside,
        );
    }
}

impl Widget for Waveform<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
