//! Waveform atom - Audio waveform visualization for EDM/VJ applications
//!
//! Displays audio sample data as a waveform. Optimized for real-time visualization
//! at 60fps.
//!
//! # Features
//! - Line or filled waveform styles
//! - Configurable height and color
//! - Theme-aware styling
//! - Supports mono and stereo display
//!
//! # Example
//! ```ignore
//! // Basic waveform
//! Waveform::new(&audio_samples)
//!     .show(ctx);
//!
//! // With custom styling
//! Waveform::new(&samples)
//!     .height(80.0)
//!     .filled(true)
//!     .color(theme.primary)
//!     .show(ctx);
//!
//! // Stereo waveform
//! Waveform::stereo(&left_samples, &right_samples)
//!     .show(ctx);
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Waveform display style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaveformStyle {
    /// Line connecting sample points
    #[default]
    Line,
    /// Filled area from center line
    Filled,
    /// Bars for each sample
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
            let painter = ui.painter();

            // Colors
            let primary_color = self.color.unwrap_or(theme.primary);
            let secondary_color = self.color_right.unwrap_or(theme.secondary);
            let bg_color = theme.bg_secondary;
            let grid_color = theme.border;
            let center_line_color = theme.text_muted;

            // Background
            painter.rect_filled(rect, theme.radius_sm, bg_color);

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

                self.draw_waveform(
                    painter,
                    left_rect,
                    self.samples,
                    primary_color,
                    grid_color,
                    center_line_color,
                    &theme,
                );

                if let Some(right_samples) = self.samples_right {
                    self.draw_waveform(
                        painter,
                        right_rect,
                        right_samples,
                        secondary_color,
                        grid_color,
                        center_line_color,
                        &theme,
                    );
                }
            } else {
                // Mono: single waveform
                self.draw_waveform(
                    painter,
                    rect,
                    self.samples,
                    primary_color,
                    grid_color,
                    center_line_color,
                    &theme,
                );
            }

            // Border
            painter.rect_stroke(rect, theme.radius_sm, Stroke::new(theme.border_width, theme.border), egui::StrokeKind::Outside);
        }

        response
    }

    fn draw_waveform(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        grid_color: Color32,
        center_line_color: Color32,
        theme: &Theme,
    ) {
        if samples.is_empty() {
            return;
        }

        let center_y = rect.center().y;
        let half_height = rect.height() / 2.0 - theme.spacing_xs;

        // Grid
        if self.show_grid {
            let grid_stroke = Stroke::new(theme.stroke_width * 0.5, grid_color);
            // Horizontal grid lines at 25%, 50%, 75%
            for i in 1..4 {
                let y = rect.min.y + rect.height() * (i as f32 / 4.0);
                painter.line_segment(
                    [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                    grid_stroke,
                );
            }
            // Vertical grid lines
            let num_v_lines = 8;
            for i in 1..num_v_lines {
                let x = rect.min.x + rect.width() * (i as f32 / num_v_lines as f32);
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    grid_stroke,
                );
            }
        }

        // Center line
        if self.show_center_line {
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, center_y),
                    Pos2::new(rect.max.x, center_y),
                ],
                Stroke::new(theme.stroke_width * 0.5, center_line_color),
            );
        }

        match self.style {
            WaveformStyle::Line => {
                self.draw_line_waveform(painter, rect, samples, color, center_y, half_height, theme);
            }
            WaveformStyle::Filled => {
                self.draw_filled_waveform(painter, rect, samples, color, center_y, half_height, theme);
            }
            WaveformStyle::Bars => {
                self.draw_bars_waveform(painter, rect, samples, color, center_y, half_height, theme);
            }
        }
    }

    fn draw_line_waveform(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        center_y: f32,
        half_height: f32,
        theme: &Theme,
    ) {
        let stroke = Stroke::new(theme.stroke_width * 1.5, color);
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

        // Draw lines between consecutive points
        for window in points.windows(2) {
            painter.line_segment([window[0], window[1]], stroke);
        }
    }

    fn draw_filled_waveform(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        center_y: f32,
        half_height: f32,
        theme: &Theme,
    ) {
        let step = rect.width() / samples.len().max(1) as f32;
        let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 100);
        let stroke = Stroke::new(theme.stroke_width, color);

        for (i, &sample) in samples.iter().enumerate() {
            let x = rect.min.x + step * i as f32;
            let sample_clamped = sample.clamp(-1.0, 1.0);
            let y = center_y - sample_clamped * half_height;

            // Fill from center to sample
            let fill_rect = if sample_clamped >= 0.0 {
                Rect::from_min_max(Pos2::new(x, y), Pos2::new(x + step, center_y))
            } else {
                Rect::from_min_max(Pos2::new(x, center_y), Pos2::new(x + step, y))
            };
            painter.rect_filled(fill_rect, 0.0, fill_color);

            // Top line
            painter.line_segment(
                [Pos2::new(x, y), Pos2::new(x + step, y)],
                stroke,
            );
        }
    }

    fn draw_bars_waveform(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        samples: &[f32],
        color: Color32,
        center_y: f32,
        half_height: f32,
        theme: &Theme,
    ) {
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
    }
}

impl Widget for Waveform<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
