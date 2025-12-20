//! Spectrum atom - Frequency spectrum visualization for EDM/VJ applications
//!
//! Displays FFT frequency bin data as vertical bars. Commonly used for
//! audio spectrum analyzers in DAWs and VJ software.
//!
//! # Features
//! - Vertical bar display with configurable band count
//! - Peak hold indicators
//! - Multiple color modes (solid, gradient, rainbow)
//! - Mirrored mode for symmetric display
//!
//! # Example
//! ```ignore
//! // Basic spectrum
//! Spectrum::new(&fft_bins)
//!     .show(ctx.ui);
//!
//! // With peak hold and gradient
//! Spectrum::new(&fft_bins)
//!     .bands(32)
//!     .peak_hold(true)
//!     .gradient(true)
//!     .show(ctx.ui);
//!
//! // Mirrored (symmetric) display
//! Spectrum::new(&fft_bins)
//!     .mirrored(true)
//!     .show(ctx.ui);
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Color mode for spectrum bars
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectrumColorMode {
    /// Single color from theme
    #[default]
    Solid,
    /// Gradient from bottom to top
    Gradient,
    /// Rainbow across frequency bands
    Rainbow,
}

/// A frequency spectrum visualization component
pub struct Spectrum<'a> {
    bins: &'a [f32],
    bands: usize,
    height: Option<f32>,
    color_mode: SpectrumColorMode,
    peak_hold: bool,
    peaks: Option<&'a [f32]>,
    mirrored: bool,
    bar_gap: f32,
}

impl<'a> Spectrum<'a> {
    /// Create a new spectrum from FFT bin data
    ///
    /// Bins should be normalized to 0.0..1.0 range
    pub fn new(bins: &'a [f32]) -> Self {
        Self {
            bins,
            bands: 32,
            height: None,
            color_mode: SpectrumColorMode::default(),
            peak_hold: false,
            peaks: None,
            mirrored: false,
            bar_gap: 2.0,
        }
    }

    /// Set number of bands to display (default: 32)
    pub fn bands(mut self, bands: usize) -> Self {
        self.bands = bands.max(1);
        self
    }

    /// Set display height (default: uses theme spacing)
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set color mode
    pub fn color_mode(mut self, mode: SpectrumColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Use gradient color mode
    pub fn gradient(mut self) -> Self {
        self.color_mode = SpectrumColorMode::Gradient;
        self
    }

    /// Use rainbow color mode
    pub fn rainbow(mut self) -> Self {
        self.color_mode = SpectrumColorMode::Rainbow;
        self
    }

    /// Enable peak hold indicators
    pub fn peak_hold(mut self, enabled: bool) -> Self {
        self.peak_hold = enabled;
        self
    }

    /// Provide external peak values (for smooth decay)
    pub fn peaks(mut self, peaks: &'a [f32]) -> Self {
        self.peaks = Some(peaks);
        self.peak_hold = true;
        self
    }

    /// Enable mirrored (symmetric) display
    pub fn mirrored(mut self, enabled: bool) -> Self {
        self.mirrored = enabled;
        self
    }

    /// Set gap between bars (default: 2.0)
    pub fn bar_gap(mut self, gap: f32) -> Self {
        self.bar_gap = gap;
        self
    }

    /// Show the spectrum
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate dimensions
        let height = self.height.unwrap_or(theme.spacing_xl * 3.0);
        let width = ui.available_width();

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, theme.radius_sm, theme.bg_secondary);

            // Calculate bands
            let display_bands = if self.mirrored {
                self.bands / 2
            } else {
                self.bands
            };

            let total_gap = self.bar_gap * (display_bands.saturating_sub(1)) as f32;
            let bar_width = if self.mirrored {
                (width - total_gap) / display_bands as f32 / 2.0 - self.bar_gap / 2.0
            } else {
                (width - total_gap) / display_bands as f32
            };

            let bins_per_band = self.bins.len() / display_bands.max(1);

            // Draw bars
            for i in 0..display_bands {
                // Average bins for this band
                let start = i * bins_per_band;
                let end = ((i + 1) * bins_per_band).min(self.bins.len());
                let slice = &self.bins[start..end];

                let value = if slice.is_empty() {
                    0.0
                } else {
                    // Use max for more responsive display
                    slice.iter().cloned().fold(0.0_f32, f32::max)
                };

                let bar_height = value.clamp(0.0, 1.0) * (height - theme.spacing_xs * 2.0);

                // Get color for this band
                let color = self.get_bar_color(i, display_bands, value, &theme);

                if self.mirrored {
                    // Right side
                    let x_right = rect.center().x + (i as f32 * (bar_width + self.bar_gap)) + self.bar_gap / 2.0;
                    let bar_rect = Rect::from_min_max(
                        Pos2::new(x_right, rect.max.y - theme.spacing_xs - bar_height),
                        Pos2::new(x_right + bar_width, rect.max.y - theme.spacing_xs),
                    );
                    painter.rect_filled(bar_rect, theme.radius_sm * 0.5, color);

                    // Left side (mirror)
                    let x_left = rect.center().x - (i as f32 * (bar_width + self.bar_gap)) - bar_width - self.bar_gap / 2.0;
                    let bar_rect_left = Rect::from_min_max(
                        Pos2::new(x_left, rect.max.y - theme.spacing_xs - bar_height),
                        Pos2::new(x_left + bar_width, rect.max.y - theme.spacing_xs),
                    );
                    painter.rect_filled(bar_rect_left, theme.radius_sm * 0.5, color);

                    // Peak indicators
                    if self.peak_hold {
                        let peak_value = self.peaks
                            .and_then(|p| p.get(i).cloned())
                            .unwrap_or(value);
                        let peak_y = rect.max.y - theme.spacing_xs - peak_value.clamp(0.0, 1.0) * (height - theme.spacing_xs * 2.0);

                        // Right peak
                        painter.line_segment(
                            [Pos2::new(x_right, peak_y), Pos2::new(x_right + bar_width, peak_y)],
                            Stroke::new(theme.stroke_width * 2.0, theme.primary),
                        );
                        // Left peak
                        painter.line_segment(
                            [Pos2::new(x_left, peak_y), Pos2::new(x_left + bar_width, peak_y)],
                            Stroke::new(theme.stroke_width * 2.0, theme.primary),
                        );
                    }
                } else {
                    // Normal (non-mirrored)
                    let x = rect.min.x + (i as f32 * (bar_width + self.bar_gap));
                    let bar_rect = Rect::from_min_max(
                        Pos2::new(x, rect.max.y - theme.spacing_xs - bar_height),
                        Pos2::new(x + bar_width, rect.max.y - theme.spacing_xs),
                    );
                    painter.rect_filled(bar_rect, theme.radius_sm * 0.5, color);

                    // Peak indicator
                    if self.peak_hold {
                        let peak_value = self.peaks
                            .and_then(|p| p.get(i).cloned())
                            .unwrap_or(value);
                        let peak_y = rect.max.y - theme.spacing_xs - peak_value.clamp(0.0, 1.0) * (height - theme.spacing_xs * 2.0);

                        painter.line_segment(
                            [Pos2::new(x, peak_y), Pos2::new(x + bar_width, peak_y)],
                            Stroke::new(theme.stroke_width * 2.0, theme.primary),
                        );
                    }
                }
            }

            // Border
            painter.rect_stroke(
                rect,
                theme.radius_sm,
                Stroke::new(theme.border_width, theme.border),
                egui::StrokeKind::Outside,
            );
        }

        response
    }

    fn get_bar_color(&self, index: usize, total: usize, value: f32, theme: &Theme) -> Color32 {
        match self.color_mode {
            SpectrumColorMode::Solid => theme.primary,
            SpectrumColorMode::Gradient => {
                // Gradient from secondary (low) to primary (high)
                let t = value.clamp(0.0, 1.0);
                Color32::from_rgb(
                    lerp_u8(theme.secondary.r(), theme.primary.r(), t),
                    lerp_u8(theme.secondary.g(), theme.primary.g(), t),
                    lerp_u8(theme.secondary.b(), theme.primary.b(), t),
                )
            }
            SpectrumColorMode::Rainbow => {
                // HSV rainbow across frequency bands
                let hue = (index as f32 / total as f32) * 360.0;
                hsv_to_rgb(hue, 0.8, 0.9)
            }
        }
    }
}

impl Widget for Spectrum<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// Helper functions
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h / 60.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
