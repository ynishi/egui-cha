//! Arc Slider atom - Modern arc-shaped slider for EDM/VJ applications
//!
//! A sleek arc slider commonly seen in modern synthesizers and audio plugins.
//! Similar to a knob but with a more visible arc indicator.
//!
//! # Example
//! ```ignore
//! // Basic arc slider
//! ArcSlider::new(0.0..=100.0)
//!     .show_with(ctx, model.value, Msg::SetValue);
//!
//! // With label
//! ArcSlider::new(0.0..=1.0)
//!     .label("Dry/Wet")
//!     .show_with(ctx, model.mix, Msg::SetMix);
//! ```

use crate::Theme;
use egui::{Response, Sense, Ui, Vec2};
use egui_cha::ViewCtx;
use std::f32::consts::PI;
use std::ops::RangeInclusive;

/// Arc slider size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArcSliderSize {
    /// Small arc slider (~48px)
    Small,
    /// Medium arc slider (~64px)
    #[default]
    Medium,
    /// Large arc slider (~80px)
    Large,
}

/// Arc style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArcStyle {
    /// Standard arc (bottom gap)
    #[default]
    Standard,
    /// Full circle (no gap)
    Full,
    /// Half circle (180°)
    Half,
    /// Quarter circle (90°)
    Quarter,
}

/// A modern arc slider control
pub struct ArcSlider<'a> {
    range: RangeInclusive<f64>,
    label: Option<&'a str>,
    size: ArcSliderSize,
    style: ArcStyle,
    show_value: bool,
    value_suffix: Option<&'a str>,
    disabled: bool,
    /// Arc thickness multiplier (1.0 = default)
    thickness: f32,
}

impl<'a> ArcSlider<'a> {
    /// Create a new arc slider with the given value range
    pub fn new(range: RangeInclusive<f64>) -> Self {
        Self {
            range,
            label: None,
            size: ArcSliderSize::default(),
            style: ArcStyle::default(),
            show_value: true,
            value_suffix: None,
            disabled: false,
            thickness: 1.0,
        }
    }

    /// Set the label displayed below the arc
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the size variant
    pub fn size(mut self, size: ArcSliderSize) -> Self {
        self.size = size;
        self
    }

    /// Use small size
    pub fn small(mut self) -> Self {
        self.size = ArcSliderSize::Small;
        self
    }

    /// Use large size
    pub fn large(mut self) -> Self {
        self.size = ArcSliderSize::Large;
        self
    }

    /// Set the arc style
    pub fn style(mut self, style: ArcStyle) -> Self {
        self.style = style;
        self
    }

    /// Show/hide the value display
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Set a suffix for the value display (e.g., "%", "dB")
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.value_suffix = Some(suffix);
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set arc thickness multiplier
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Get arc angles based on style
    fn get_arc_angles(&self) -> (f32, f32) {
        match self.style {
            ArcStyle::Standard => (0.75 * PI, 2.25 * PI), // 270° arc with bottom gap
            ArcStyle::Full => (0.0, 2.0 * PI),            // Full circle
            ArcStyle::Half => (PI, 2.0 * PI),             // Bottom half
            ArcStyle::Quarter => (1.25 * PI, 1.75 * PI),  // Bottom quarter
        }
    }

    /// TEA-style: Show arc slider with immutable value, emit Msg on change
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

    /// Show arc slider (modifies value in place)
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> Response {
        self.show_internal(ui, value)
    }

    fn show_internal(self, ui: &mut Ui, value: &mut f64) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate dimensions based on size
        let diameter = match self.size {
            ArcSliderSize::Small => theme.spacing_xl + theme.spacing_md, // ~48px
            ArcSliderSize::Medium => theme.spacing_xl * 2.0,             // ~64px
            ArcSliderSize::Large => theme.spacing_xl * 2.0 + theme.spacing_lg, // ~80px
        };

        // Total height including label
        let label_height = if self.label.is_some() {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };
        let total_height = diameter + label_height;

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
            // Horizontal drag also works: right = increase, left = decrease
            let sensitivity = 0.003 * (self.range.end() - self.range.start());
            let change = (-delta.y + delta.x * 0.5) as f64 * sensitivity;
            *value = (*value + change).clamp(*self.range.start(), *self.range.end());
            response.mark_changed();
        }

        // Double-click to reset to center
        if response.double_clicked() && !self.disabled {
            *value = (*self.range.start() + *self.range.end()) / 2.0;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center_top() + Vec2::new(0.0, diameter / 2.0);
            let radius = diameter / 2.0 - theme.spacing_xs;

            // Colors
            let (track_color, arc_color, text_color) = if self.disabled {
                (theme.border, theme.text_muted, theme.text_muted)
            } else if response.hovered() || response.dragged() {
                (theme.border, theme.primary_hover, theme.text_primary)
            } else {
                (theme.border, theme.primary, theme.text_primary)
            };

            let (arc_start, arc_end) = self.get_arc_angles();
            let stroke_width = theme.stroke_width * 4.0 * self.thickness;

            // Background arc (track)
            self.draw_arc(
                painter,
                center,
                radius - stroke_width / 2.0,
                arc_start,
                arc_end,
                egui::Stroke::new(stroke_width, track_color),
            );

            // Value arc (foreground)
            let normalized =
                (*value - *self.range.start()) / (*self.range.end() - *self.range.start());
            let value_angle = arc_start + (arc_end - arc_start) * normalized as f32;

            if normalized > 0.001 {
                self.draw_arc(
                    painter,
                    center,
                    radius - stroke_width / 2.0,
                    arc_start,
                    value_angle,
                    egui::Stroke::new(stroke_width, arc_color),
                );
            }

            // End cap (small circle at current position)
            let cap_radius = stroke_width / 2.0 + 1.0;
            let cap_pos = center
                + Vec2::new(
                    value_angle.sin() * (radius - stroke_width / 2.0),
                    -value_angle.cos() * (radius - stroke_width / 2.0),
                );
            painter.circle_filled(cap_pos, cap_radius, arc_color);

            // Value text (center)
            if self.show_value {
                let value_text = if *self.range.end() - *self.range.start() > 10.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{:.2}", value)
                };
                let display_text = if let Some(suffix) = self.value_suffix {
                    format!("{}{}", value_text, suffix)
                } else {
                    value_text
                };

                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    &display_text,
                    egui::FontId::proportional(theme.font_size_sm),
                    text_color,
                );
            }

            // Label (below)
            if let Some(label) = self.label {
                let label_pos = rect.center_bottom() - Vec2::new(0.0, theme.spacing_xs);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_BOTTOM,
                    label,
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_secondary,
                );
            }
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

        painter.add(egui::Shape::line(points, stroke));
    }
}
