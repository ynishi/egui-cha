//! CrossFader - A/B mix fader for DJ/VJ transitions
//!
//! A horizontal fader for mixing between two sources (A/B).
//! Common in DJ mixers and VJ software for transitions.
//!
//! # Example
//! ```ignore
//! CrossFader::new()
//!     .value(model.mix)  // -1.0 = A, 0.0 = center, 1.0 = B
//!     .labels("Deck A", "Deck B")
//!     .curve(CrossfaderCurve::EqualPower)
//!     .show_with(ctx, |value| Msg::SetMix(value));
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Crossfader curve types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossfaderCurve {
    /// Linear crossfade
    #[default]
    Linear,
    /// Equal power (constant loudness)
    EqualPower,
    /// Fast cut (DJ scratch style)
    FastCut,
    /// Slow transition
    Smooth,
}

impl CrossfaderCurve {
    /// Apply curve to get A/B mix values
    /// Returns (a_level, b_level) where each is 0.0-1.0
    pub fn apply(&self, value: f32) -> (f32, f32) {
        // value: -1.0 (full A) to 1.0 (full B)
        let normalized = (value + 1.0) / 2.0; // 0.0 to 1.0

        match self {
            CrossfaderCurve::Linear => {
                (1.0 - normalized, normalized)
            }
            CrossfaderCurve::EqualPower => {
                let angle = normalized * std::f32::consts::FRAC_PI_2;
                (angle.cos(), angle.sin())
            }
            CrossfaderCurve::FastCut => {
                // Sharp cut near edges
                let a = if normalized < 0.1 {
                    1.0
                } else if normalized > 0.9 {
                    0.0
                } else {
                    1.0 - (normalized - 0.1) / 0.8
                };
                let b = if normalized > 0.9 {
                    1.0
                } else if normalized < 0.1 {
                    0.0
                } else {
                    (normalized - 0.1) / 0.8
                };
                (a, b)
            }
            CrossfaderCurve::Smooth => {
                // S-curve for smooth transitions
                let t = normalized;
                let smooth = t * t * (3.0 - 2.0 * t);
                (1.0 - smooth, smooth)
            }
        }
    }
}

/// Crossfader orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossfaderOrientation {
    /// Horizontal (A left, B right)
    #[default]
    Horizontal,
    /// Vertical (A bottom, B top)
    Vertical,
}

/// A/B Crossfader component
pub struct CrossFader<'a> {
    value: f32,
    label_a: Option<&'a str>,
    label_b: Option<&'a str>,
    curve: CrossfaderCurve,
    orientation: CrossfaderOrientation,
    width: f32,
    height: f32,
    show_levels: bool,
    show_center_detent: bool,
    color_a: Option<Color32>,
    color_b: Option<Color32>,
}

impl<'a> CrossFader<'a> {
    /// Create a new crossfader
    pub fn new() -> Self {
        Self {
            value: 0.0,
            label_a: None,
            label_b: None,
            curve: CrossfaderCurve::default(),
            orientation: CrossfaderOrientation::default(),
            width: 200.0,
            height: 40.0,
            show_levels: true,
            show_center_detent: true,
            color_a: None,
            color_b: None,
        }
    }

    /// Set current value (-1.0 = A, 0.0 = center, 1.0 = B)
    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(-1.0, 1.0);
        self
    }

    /// Set labels for A and B
    pub fn labels(mut self, a: &'a str, b: &'a str) -> Self {
        self.label_a = Some(a);
        self.label_b = Some(b);
        self
    }

    /// Set crossfader curve
    pub fn curve(mut self, curve: CrossfaderCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Set orientation
    pub fn orientation(mut self, orientation: CrossfaderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Show/hide level indicators
    pub fn show_levels(mut self, show: bool) -> Self {
        self.show_levels = show;
        self
    }

    /// Show/hide center detent marker
    pub fn show_center_detent(mut self, show: bool) -> Self {
        self.show_center_detent = show;
        self
    }

    /// Set color for A side
    pub fn color_a(mut self, color: Color32) -> Self {
        self.color_a = Some(color);
        self
    }

    /// Set color for B side
    pub fn color_b(mut self, color: Color32) -> Self {
        self.color_b = Some(color);
        self
    }

    /// TEA-style: Show crossfader and emit value changes
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_change: impl FnOnce(f32) -> Msg,
    ) {
        if let Some(new_value) = self.render(ctx.ui) {
            ctx.emit(on_change(new_value));
        }
    }

    /// Show crossfader, returns new value if changed
    pub fn show(self, ui: &mut Ui) -> Option<f32> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<f32> {
        let theme = Theme::current(ui.ctx());
        let mut new_value = None;

        let is_horizontal = self.orientation == CrossfaderOrientation::Horizontal;
        let (total_width, total_height) = if is_horizontal {
            (self.width, self.height)
        } else {
            (self.height, self.width)
        };

        // Add space for labels
        let label_height = if self.label_a.is_some() || self.label_b.is_some() {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };

        let level_height = if self.show_levels { 6.0 } else { 0.0 };

        let full_height = total_height + label_height + level_height;

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(total_width, full_height),
            Sense::click_and_drag(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Handle drag
        if response.dragged() {
            let delta = if is_horizontal {
                response.drag_delta().x
            } else {
                -response.drag_delta().y
            };
            let range = if is_horizontal { total_width } else { total_height };
            let delta_normalized = delta / (range * 0.5);
            let new_val = (self.value + delta_normalized).clamp(-1.0, 1.0);
            new_value = Some(new_val);
        }

        // Handle click to position
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let track_rect = Rect::from_min_size(
                    rect.min + Vec2::new(0.0, label_height),
                    Vec2::new(total_width, total_height - level_height),
                );
                let normalized = if is_horizontal {
                    (pos.x - track_rect.min.x) / track_rect.width()
                } else {
                    1.0 - (pos.y - track_rect.min.y) / track_rect.height()
                };
                let new_val = (normalized * 2.0 - 1.0).clamp(-1.0, 1.0);
                new_value = Some(new_val);
            }
        }

        let current_value = new_value.unwrap_or(self.value);
        let (a_level, b_level) = self.curve.apply(current_value);

        let painter = ui.painter();
        let color_a = self.color_a.unwrap_or(theme.primary);
        let color_b = self.color_b.unwrap_or(theme.state_warning);

        // Track area
        let track_rect = Rect::from_min_size(
            rect.min + Vec2::new(0.0, label_height),
            Vec2::new(total_width, total_height - level_height - label_height),
        );

        // Draw labels
        if let Some(label_a) = self.label_a {
            painter.text(
                egui::pos2(rect.min.x + 4.0, rect.min.y + label_height / 2.0),
                egui::Align2::LEFT_CENTER,
                label_a,
                egui::FontId::proportional(theme.font_size_xs),
                color_a,
            );
        }
        if let Some(label_b) = self.label_b {
            painter.text(
                egui::pos2(rect.max.x - 4.0, rect.min.y + label_height / 2.0),
                egui::Align2::RIGHT_CENTER,
                label_b,
                egui::FontId::proportional(theme.font_size_xs),
                color_b,
            );
        }

        // Draw track background
        painter.rect_filled(track_rect, theme.radius_sm, theme.bg_secondary);

        // Draw A/B gradient zones
        let center_x = track_rect.center().x;
        let thumb_pos = (current_value + 1.0) / 2.0;
        let thumb_x = track_rect.min.x + thumb_pos * track_rect.width();

        // A side fill
        let a_rect = Rect::from_min_max(
            track_rect.min,
            egui::pos2(center_x, track_rect.max.y),
        );
        let a_fill = Color32::from_rgba_unmultiplied(
            color_a.r(),
            color_a.g(),
            color_a.b(),
            (a_level * 100.0) as u8,
        );
        painter.rect_filled(a_rect, theme.radius_sm, a_fill);

        // B side fill
        let b_rect = Rect::from_min_max(
            egui::pos2(center_x, track_rect.min.y),
            track_rect.max,
        );
        let b_fill = Color32::from_rgba_unmultiplied(
            color_b.r(),
            color_b.g(),
            color_b.b(),
            (b_level * 100.0) as u8,
        );
        painter.rect_filled(b_rect, theme.radius_sm, b_fill);

        // Draw center detent
        if self.show_center_detent {
            painter.line_segment(
                [
                    egui::pos2(center_x, track_rect.min.y),
                    egui::pos2(center_x, track_rect.max.y),
                ],
                Stroke::new(1.0, theme.text_muted),
            );
        }

        // Draw thumb
        let thumb_width = 16.0;
        let thumb_rect = Rect::from_center_size(
            egui::pos2(thumb_x, track_rect.center().y),
            Vec2::new(thumb_width, track_rect.height() - 4.0),
        );

        let thumb_color = if response.hovered() || response.dragged() {
            theme.text_primary
        } else {
            theme.text_secondary
        };

        painter.rect_filled(thumb_rect, theme.radius_sm * 0.5, thumb_color);

        // Thumb grip lines
        for i in -1..=1 {
            let line_x = thumb_rect.center().x + i as f32 * 3.0;
            painter.line_segment(
                [
                    egui::pos2(line_x, thumb_rect.min.y + 4.0),
                    egui::pos2(line_x, thumb_rect.max.y - 4.0),
                ],
                Stroke::new(1.0, theme.bg_primary),
            );
        }

        // Draw level indicators
        if self.show_levels {
            let level_y = track_rect.max.y + 2.0;
            let level_rect_height = level_height - 2.0;

            // A level
            let a_level_width = (center_x - track_rect.min.x) * a_level;
            let a_level_rect = Rect::from_min_size(
                egui::pos2(track_rect.min.x, level_y),
                Vec2::new(a_level_width, level_rect_height),
            );
            painter.rect_filled(a_level_rect, 2.0, color_a);

            // B level
            let b_level_width = (track_rect.max.x - center_x) * b_level;
            let b_level_rect = Rect::from_min_size(
                egui::pos2(track_rect.max.x - b_level_width, level_y),
                Vec2::new(b_level_width, level_rect_height),
            );
            painter.rect_filled(b_level_rect, 2.0, color_b);
        }

        // Draw border
        painter.rect_stroke(
            track_rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        new_value
    }
}

impl Default for CrossFader<'_> {
    fn default() -> Self {
        Self::new()
    }
}
