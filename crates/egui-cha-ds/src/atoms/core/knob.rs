//! Knob atom - Rotary knob control for EDM/VJ applications
//!
//! A rotary knob component for parameter adjustment, common in DAWs and synthesizers.
//!
//! # Features
//! - Circular knob with arc indicator
//! - Drag to adjust value
//! - Optional label and value display
//! - Theme-aware styling
//! - Multiple size variants
//!
//! # Example
//! ```ignore
//! // Basic knob
//! Knob::new(0.0..=1.0)
//!     .show_with(ctx, model.cutoff, |v| Msg::SetCutoff(v));
//!
//! // With label
//! Knob::new(0.0..=100.0)
//!     .label("Volume")
//!     .show_with(ctx, model.volume, Msg::SetVolume);
//!
//! // Compact size
//! Knob::new(0.0..=1.0)
//!     .compact()
//!     .show_with(ctx, model.pan, Msg::SetPan);
//! ```

use crate::Theme;
use egui::{Response, Sense, Ui, Vec2, Widget};
use egui_cha::ViewCtx;
use std::f32::consts::PI;
use std::ops::RangeInclusive;

/// Size variant for Knob
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KnobSize {
    /// Compact size (~32px)
    Compact,
    /// Default size (~48px)
    #[default]
    Medium,
    /// Large size (~64px)
    Large,
}

/// A rotary knob control
pub struct Knob<'a> {
    range: RangeInclusive<f64>,
    label: Option<&'a str>,
    size: KnobSize,
    show_value: bool,
    disabled: bool,
    /// Arc start angle (radians from bottom, clockwise)
    arc_start: f32,
    /// Arc end angle (radians from bottom, clockwise)
    arc_end: f32,
}

impl<'a> Knob<'a> {
    /// Create a new knob with the given range
    pub fn new(range: RangeInclusive<f64>) -> Self {
        Self {
            range,
            label: None,
            size: KnobSize::default(),
            show_value: true,
            disabled: false,
            // Default arc: 270 degrees, starting from bottom-left
            arc_start: -0.75 * PI, // -135 degrees
            arc_end: 0.75 * PI,    // +135 degrees
        }
    }

    /// Set a label displayed below the knob
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set compact size
    pub fn compact(mut self) -> Self {
        self.size = KnobSize::Compact;
        self
    }

    /// Set size variant
    pub fn size(mut self, size: KnobSize) -> Self {
        self.size = size;
        self
    }

    /// Show or hide the value display
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show knob with immutable value, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: f64,
        on_change: impl FnOnce(f64) -> Msg,
    ) {
        let mut current = value;
        let response = self.show_internal(ctx.ui, &mut current);

        if response.changed() {
            ctx.emit(on_change(current));
        }
    }

    /// Show knob (modifies value in place)
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> Response {
        self.show_internal(ui, value)
    }

    fn show_internal(self, ui: &mut Ui, value: &mut f64) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate size based on variant
        let diameter = match self.size {
            KnobSize::Compact => theme.spacing_lg + theme.spacing_md, // ~40px
            KnobSize::Medium => theme.spacing_xl + theme.spacing_md,  // ~48px
            KnobSize::Large => theme.spacing_xl + theme.spacing_lg,   // ~56px
        };

        // Total height including label
        let label_height = if self.label.is_some() {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };
        let value_height = if self.show_value {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };
        let total_height = diameter + label_height + value_height;

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(diameter, total_height),
            if self.disabled {
                Sense::hover()
            } else {
                Sense::click_and_drag()
            },
        );

        // Handle drag
        if response.dragged() && !self.disabled {
            let delta = response.drag_delta();
            // Vertical drag: up = increase, down = decrease
            let sensitivity = 0.005 * (self.range.end() - self.range.start());
            *value = (*value - delta.y as f64 * sensitivity)
                .clamp(*self.range.start(), *self.range.end());
        }

        // Double-click to reset to center
        if response.double_clicked() && !self.disabled {
            *value = (*self.range.start() + *self.range.end()) / 2.0;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let knob_center = rect.center_top() + Vec2::new(0.0, diameter / 2.0);
            let radius = diameter / 2.0 - theme.spacing_xs;

            // Colors
            let (bg_color, track_color, arc_color) = if self.disabled {
                (theme.bg_tertiary, theme.border, theme.text_muted)
            } else if response.hovered() || response.dragged() {
                (theme.bg_tertiary, theme.border, theme.primary_hover)
            } else {
                (theme.bg_secondary, theme.border, theme.primary)
            };

            // Background circle
            painter.circle_filled(knob_center, radius, bg_color);

            // Track arc (background)
            let stroke_width = theme.stroke_width * 3.0;
            self.draw_arc(
                painter,
                knob_center,
                radius - stroke_width,
                self.arc_start,
                self.arc_end,
                egui::Stroke::new(stroke_width, track_color),
            );

            // Value arc (foreground)
            let normalized =
                (*value - *self.range.start()) / (*self.range.end() - *self.range.start());
            let value_angle = self.arc_start + (self.arc_end - self.arc_start) * normalized as f32;

            if normalized > 0.001 {
                self.draw_arc(
                    painter,
                    knob_center,
                    radius - stroke_width,
                    self.arc_start,
                    value_angle,
                    egui::Stroke::new(stroke_width, arc_color),
                );
            }

            // Indicator dot
            let dot_radius = theme.spacing_xs / 2.0;
            let dot_distance = radius - stroke_width * 2.5;
            let dot_pos = knob_center
                + Vec2::new(
                    value_angle.sin() * dot_distance,
                    -value_angle.cos() * dot_distance,
                );
            painter.circle_filled(dot_pos, dot_radius, arc_color);

            // Value text (center of knob)
            if self.show_value {
                let value_text = if *self.range.end() - *self.range.start() > 10.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{:.2}", value)
                };
                painter.text(
                    knob_center,
                    egui::Align2::CENTER_CENTER,
                    &value_text,
                    egui::FontId::proportional(theme.font_size_xs),
                    if self.disabled {
                        theme.text_muted
                    } else {
                        theme.text_primary
                    },
                );
            }

            // Label below knob
            if let Some(label) = self.label {
                let label_pos = rect.center_bottom() - Vec2::new(0.0, theme.font_size_xs / 2.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(theme.font_size_xs),
                    if self.disabled {
                        theme.text_muted
                    } else {
                        theme.text_secondary
                    },
                );
            }
        }

        // Mark response as changed if value was modified
        if response.dragged() || response.double_clicked() {
            response.mark_changed();
        }
        response
    }

    /// Draw an arc using line segments
    fn draw_arc(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        stroke: egui::Stroke,
    ) {
        let segments = 32;
        let angle_step = (end_angle - start_angle) / segments as f32;

        let points: Vec<egui::Pos2> = (0..=segments)
            .map(|i| {
                let angle = start_angle + angle_step * i as f32;
                center + Vec2::new(angle.sin() * radius, -angle.cos() * radius)
            })
            .collect();

        for i in 0..points.len() - 1 {
            painter.line_segment([points[i], points[i + 1]], stroke);
        }
    }
}

impl Widget for Knob<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut dummy = 0.5; // Default middle value
        self.show_internal(ui, &mut dummy)
    }
}
