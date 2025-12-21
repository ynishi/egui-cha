//! Level Meter atom - VU/Peak meter for audio level visualization
//!
//! # Example
//! ```ignore
//! // Mono meter
//! LevelMeter::new()
//!     .show(ui, -12.0);
//!
//! // Stereo with peak hold
//! LevelMeter::new()
//!     .stereo(true)
//!     .show_stereo_with_peak(ui, left_db, right_db, left_peak, right_peak);
//! ```

use crate::Theme;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Level meter display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterMode {
    /// VU-style meter with slower response
    #[default]
    VU,
    /// Peak meter with fast attack
    Peak,
    /// RMS meter showing average level
    RMS,
}

/// Level meter orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterOrientation {
    /// Vertical meter (bottom to top)
    #[default]
    Vertical,
    /// Horizontal meter (left to right)
    Horizontal,
}

/// Level meter widget for audio visualization
///
/// Displays audio levels with optional peak hold, stereo mode,
/// and configurable color segments (green/yellow/red).
#[derive(Debug, Clone)]
pub struct LevelMeter {
    width: f32,
    height: f32,
    mode: MeterMode,
    orientation: MeterOrientation,
    stereo: bool,
    peak_hold: bool,
    show_scale: bool,
    min_db: f32,
    max_db: f32,
    /// Threshold for yellow zone (dB)
    yellow_threshold: f32,
    /// Threshold for red zone (dB)
    red_threshold: f32,
}

impl Default for LevelMeter {
    fn default() -> Self {
        Self {
            width: 24.0,
            height: 200.0,
            mode: MeterMode::default(),
            orientation: MeterOrientation::default(),
            stereo: false,
            peak_hold: true,
            show_scale: true,
            min_db: -60.0,
            max_db: 6.0,
            yellow_threshold: -12.0,
            red_threshold: -3.0,
        }
    }
}

impl LevelMeter {
    /// Create a new level meter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the meter dimensions
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the meter mode (VU, Peak, RMS)
    pub fn mode(mut self, mode: MeterMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the meter orientation
    pub fn orientation(mut self, orientation: MeterOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Enable stereo mode (two channels side by side)
    pub fn stereo(mut self, stereo: bool) -> Self {
        self.stereo = stereo;
        self
    }

    /// Enable peak hold indicator
    pub fn peak_hold(mut self, enabled: bool) -> Self {
        self.peak_hold = enabled;
        self
    }

    /// Show dB scale markings
    pub fn show_scale(mut self, show: bool) -> Self {
        self.show_scale = show;
        self
    }

    /// Set the dB range
    pub fn range(mut self, min_db: f32, max_db: f32) -> Self {
        self.min_db = min_db;
        self.max_db = max_db;
        self
    }

    /// Set the yellow zone threshold (dB)
    pub fn yellow_at(mut self, db: f32) -> Self {
        self.yellow_threshold = db;
        self
    }

    /// Set the red zone threshold (dB)
    pub fn red_at(mut self, db: f32) -> Self {
        self.red_threshold = db;
        self
    }

    /// Convert dB to normalized 0.0-1.0 value
    fn db_to_normalized(&self, db: f32) -> f32 {
        let clamped = db.clamp(self.min_db, self.max_db);
        (clamped - self.min_db) / (self.max_db - self.min_db)
    }

    /// Get color for a given dB level
    fn get_level_color(&self, db: f32, theme: &Theme) -> Color32 {
        if db >= self.red_threshold {
            theme.state_danger // Red - clipping danger
        } else if db >= self.yellow_threshold {
            theme.state_warning // Yellow - caution
        } else {
            theme.state_success // Green - safe
        }
    }

    /// TEA-style: Show meter with current level
    pub fn show_with<Msg>(&self, ctx: &mut ViewCtx<'_, Msg>, level_db: f32) {
        self.show_internal(ctx.ui, level_db, level_db, None);
    }

    /// TEA-style: Show stereo meter
    pub fn show_stereo_with<Msg>(&self, ctx: &mut ViewCtx<'_, Msg>, left_db: f32, right_db: f32) {
        self.show_internal(ctx.ui, left_db, right_db, None);
    }

    /// Display the level meter (mono mode)
    pub fn show(&self, ui: &mut Ui, level_db: f32) -> Response {
        self.show_internal(ui, level_db, level_db, None)
    }

    /// Display the level meter with peak hold (mono mode)
    pub fn show_with_peak(&self, ui: &mut Ui, level_db: f32, peak_db: f32) -> Response {
        self.show_internal(ui, level_db, level_db, Some((peak_db, peak_db)))
    }

    /// Display the level meter (stereo mode)
    pub fn show_stereo(&self, ui: &mut Ui, left_db: f32, right_db: f32) -> Response {
        self.show_internal(ui, left_db, right_db, None)
    }

    /// Display the level meter with peak hold (stereo mode)
    pub fn show_stereo_with_peak(
        &self,
        ui: &mut Ui,
        left_db: f32,
        right_db: f32,
        left_peak: f32,
        right_peak: f32,
    ) -> Response {
        self.show_internal(ui, left_db, right_db, Some((left_peak, right_peak)))
    }

    fn show_internal(
        &self,
        ui: &mut Ui,
        left_db: f32,
        right_db: f32,
        peaks: Option<(f32, f32)>,
    ) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate total size including scale
        let scale_width = if self.show_scale { 24.0 } else { 0.0 };
        let total_width = self.width + scale_width;

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(total_width, self.height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter();

        // Calculate meter area (excluding scale)
        let meter_rect = if self.show_scale {
            Rect::from_min_size(
                rect.min + Vec2::new(scale_width, 0.0),
                Vec2::new(self.width, self.height),
            )
        } else {
            rect
        };

        // Background
        painter.rect_filled(meter_rect, 2.0, theme.bg_tertiary);
        painter.rect_stroke(
            meter_rect,
            2.0,
            Stroke::new(1.0, theme.border),
            egui::StrokeKind::Outside,
        );

        // Draw scale if enabled
        if self.show_scale {
            self.draw_scale(ui, rect, &theme);
        }

        // Draw meter bars
        let bar_padding = 2.0;
        let inner_rect = meter_rect.shrink(bar_padding);

        if self.stereo {
            let bar_width = (inner_rect.width() - 2.0) / 2.0;

            // Left channel
            let left_rect =
                Rect::from_min_size(inner_rect.min, Vec2::new(bar_width, inner_rect.height()));
            self.draw_meter_bar(ui, left_rect, left_db, peaks.map(|(l, _)| l), &theme);

            // Right channel
            let right_rect = Rect::from_min_size(
                inner_rect.min + Vec2::new(bar_width + 2.0, 0.0),
                Vec2::new(bar_width, inner_rect.height()),
            );
            self.draw_meter_bar(ui, right_rect, right_db, peaks.map(|(_, r)| r), &theme);
        } else {
            self.draw_meter_bar(ui, inner_rect, left_db, peaks.map(|(l, _)| l), &theme);
        }

        response
    }

    fn draw_meter_bar(
        &self,
        ui: &mut Ui,
        rect: Rect,
        level_db: f32,
        peak_db: Option<f32>,
        theme: &Theme,
    ) {
        let painter = ui.painter();
        let normalized = self.db_to_normalized(level_db);

        match self.orientation {
            MeterOrientation::Vertical => {
                // Draw segmented meter (bottom to top)
                let segments = 30;
                let segment_height = rect.height() / segments as f32;
                let segment_gap = 1.0;

                for i in 0..segments {
                    let segment_normalized = (i as f32 + 0.5) / segments as f32;
                    let segment_db = self.min_db + segment_normalized * (self.max_db - self.min_db);

                    let y = rect.max.y - (i as f32 + 1.0) * segment_height;
                    let segment_rect = Rect::from_min_size(
                        egui::pos2(rect.min.x, y + segment_gap / 2.0),
                        Vec2::new(rect.width(), segment_height - segment_gap),
                    );

                    if segment_normalized <= normalized {
                        let color = self.get_level_color(segment_db, theme);
                        painter.rect_filled(segment_rect, 1.0, color);
                    } else {
                        // Dim segments above level
                        painter.rect_filled(segment_rect, 1.0, theme.bg_secondary);
                    }
                }

                // Peak hold indicator
                if self.peak_hold {
                    if let Some(peak) = peak_db {
                        let peak_normalized = self.db_to_normalized(peak);
                        let peak_y = rect.max.y - peak_normalized * rect.height();
                        let peak_color = self.get_level_color(peak, theme);
                        painter.hline(
                            rect.min.x..=rect.max.x,
                            peak_y,
                            Stroke::new(2.0, peak_color),
                        );
                    }
                }
            }
            MeterOrientation::Horizontal => {
                // Draw segmented meter (left to right)
                let segments = 30;
                let segment_width = rect.width() / segments as f32;
                let segment_gap = 1.0;

                for i in 0..segments {
                    let segment_normalized = (i as f32 + 0.5) / segments as f32;
                    let segment_db = self.min_db + segment_normalized * (self.max_db - self.min_db);

                    let x = rect.min.x + i as f32 * segment_width;
                    let segment_rect = Rect::from_min_size(
                        egui::pos2(x + segment_gap / 2.0, rect.min.y),
                        Vec2::new(segment_width - segment_gap, rect.height()),
                    );

                    if segment_normalized <= normalized {
                        let color = self.get_level_color(segment_db, theme);
                        painter.rect_filled(segment_rect, 1.0, color);
                    } else {
                        painter.rect_filled(segment_rect, 1.0, theme.bg_secondary);
                    }
                }

                // Peak hold indicator
                if self.peak_hold {
                    if let Some(peak) = peak_db {
                        let peak_normalized = self.db_to_normalized(peak);
                        let peak_x = rect.min.x + peak_normalized * rect.width();
                        let peak_color = self.get_level_color(peak, theme);
                        painter.vline(
                            peak_x,
                            rect.min.y..=rect.max.y,
                            Stroke::new(2.0, peak_color),
                        );
                    }
                }
            }
        }
    }

    fn draw_scale(&self, ui: &mut Ui, rect: Rect, theme: &Theme) {
        let painter = ui.painter();
        let scale_marks = [-60, -48, -36, -24, -18, -12, -6, -3, 0, 3, 6];

        for &db in &scale_marks {
            let db_f32 = db as f32;
            if db_f32 < self.min_db || db_f32 > self.max_db {
                continue;
            }

            let normalized = self.db_to_normalized(db_f32);
            let y = rect.max.y - normalized * self.height;

            // Draw tick mark
            painter.hline(
                rect.min.x + 16.0..=rect.min.x + 22.0,
                y,
                Stroke::new(1.0, theme.text_secondary),
            );

            // Draw label
            let label = if db == 0 {
                "0".to_string()
            } else {
                format!("{}", db)
            };

            painter.text(
                egui::pos2(rect.min.x + 14.0, y),
                egui::Align2::RIGHT_CENTER,
                label,
                egui::FontId::proportional(9.0),
                theme.text_secondary,
            );
        }
    }
}
