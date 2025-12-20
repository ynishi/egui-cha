//! XYPad atom - 2D pad control for EDM/VJ applications
//!
//! A two-dimensional touch/drag pad for controlling X/Y parameters simultaneously.
//! Commonly used for filter cutoff/resonance, effects, or any 2D parameter space.
//!
//! # Features
//! - 2D drag control with visual feedback
//! - Optional axis labels
//! - Crosshair cursor display
//! - Optional grid overlay
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! // Basic XY pad
//! XYPad::new()
//!     .show_with(ctx, (model.x, model.y), |(x, y)| Msg::SetXY(x, y));
//!
//! // With labels and grid
//! XYPad::new()
//!     .label_x("Cutoff")
//!     .label_y("Resonance")
//!     .grid(true)
//!     .show_with(ctx, (model.cutoff, model.reso), Msg::SetFilter);
//!
//! // Custom size
//! XYPad::new()
//!     .size(200.0, 200.0)
//!     .show_with(ctx, (model.x, model.y), Msg::SetXY);
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use egui_cha::ViewCtx;

/// A 2D pad control for X/Y parameter adjustment
pub struct XYPad<'a> {
    width: Option<f32>,
    height: Option<f32>,
    label_x: Option<&'a str>,
    label_y: Option<&'a str>,
    show_grid: bool,
    show_crosshair: bool,
    disabled: bool,
}

impl<'a> XYPad<'a> {
    /// Create a new XY pad
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            label_x: None,
            label_y: None,
            show_grid: false,
            show_crosshair: true,
            disabled: false,
        }
    }

    /// Set custom size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set X-axis label (displayed at bottom)
    pub fn label_x(mut self, label: &'a str) -> Self {
        self.label_x = Some(label);
        self
    }

    /// Set Y-axis label (displayed at left, rotated)
    pub fn label_y(mut self, label: &'a str) -> Self {
        self.label_y = Some(label);
        self
    }

    /// Show/hide grid overlay
    pub fn grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Show/hide crosshair cursor
    pub fn crosshair(mut self, show: bool) -> Self {
        self.show_crosshair = show;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show pad with immutable values, emit Msg on change
    ///
    /// Values are normalized to 0.0..1.0 range
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: (f64, f64),
        on_change: impl FnOnce((f64, f64)) -> Msg,
    ) {
        let mut current = value;
        let response = self.show_internal(ctx.ui, &mut current);

        if response.changed() {
            ctx.emit(on_change(current));
        }
    }

    /// Show pad (modifies values in place)
    pub fn show(self, ui: &mut Ui, value: &mut (f64, f64)) -> Response {
        self.show_internal(ui, value)
    }

    fn show_internal(self, ui: &mut Ui, value: &mut (f64, f64)) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate dimensions
        let pad_width = self.width.unwrap_or(theme.spacing_xl * 5.0);
        let pad_height = self.height.unwrap_or(theme.spacing_xl * 5.0);

        // Additional space for labels
        let label_x_height = if self.label_x.is_some() {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };
        let label_y_width = if self.label_y.is_some() {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };

        let total_width = pad_width + label_y_width;
        let total_height = pad_height + label_x_height;

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(total_width, total_height),
            if self.disabled { Sense::hover() } else { Sense::click_and_drag() },
        );

        // Pad area (excluding labels)
        let pad_rect = Rect::from_min_size(
            rect.min + Vec2::new(label_y_width, 0.0),
            Vec2::new(pad_width, pad_height),
        );

        // Handle interaction
        if (response.dragged() || response.clicked()) && !self.disabled {
            if let Some(pos) = response.interact_pointer_pos() {
                // Clamp to pad area
                let clamped_pos = Pos2::new(
                    pos.x.clamp(pad_rect.min.x, pad_rect.max.x),
                    pos.y.clamp(pad_rect.min.y, pad_rect.max.y),
                );

                // Convert to 0..1 range
                let x = ((clamped_pos.x - pad_rect.min.x) / pad_width) as f64;
                let y = (1.0 - (clamped_pos.y - pad_rect.min.y) / pad_height) as f64; // Invert Y

                value.0 = x.clamp(0.0, 1.0);
                value.1 = y.clamp(0.0, 1.0);
            }
        }

        // Double-click to reset to center
        if response.double_clicked() && !self.disabled {
            value.0 = 0.5;
            value.1 = 0.5;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Colors
            let (bg_color, border_color, cursor_color, grid_color) = if self.disabled {
                (theme.bg_tertiary, theme.border, theme.text_muted, theme.border)
            } else if response.hovered() || response.dragged() {
                (theme.bg_secondary, theme.primary, theme.primary, theme.border)
            } else {
                (theme.bg_secondary, theme.border, theme.primary, theme.border)
            };

            // Background
            painter.rect_filled(pad_rect, theme.radius_md, bg_color);

            // Grid
            if self.show_grid {
                let grid_stroke = Stroke::new(theme.stroke_width * 0.5, grid_color);
                let grid_divisions = 4;

                // Vertical lines
                for i in 1..grid_divisions {
                    let x = pad_rect.min.x + pad_width * (i as f32 / grid_divisions as f32);
                    painter.line_segment(
                        [Pos2::new(x, pad_rect.min.y), Pos2::new(x, pad_rect.max.y)],
                        grid_stroke,
                    );
                }

                // Horizontal lines
                for i in 1..grid_divisions {
                    let y = pad_rect.min.y + pad_height * (i as f32 / grid_divisions as f32);
                    painter.line_segment(
                        [Pos2::new(pad_rect.min.x, y), Pos2::new(pad_rect.max.x, y)],
                        grid_stroke,
                    );
                }
            }

            // Center cross (light)
            let center_stroke = Stroke::new(theme.stroke_width * 0.5, theme.text_muted);
            painter.line_segment(
                [
                    Pos2::new(pad_rect.center().x, pad_rect.min.y),
                    Pos2::new(pad_rect.center().x, pad_rect.max.y),
                ],
                center_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(pad_rect.min.x, pad_rect.center().y),
                    Pos2::new(pad_rect.max.x, pad_rect.center().y),
                ],
                center_stroke,
            );

            // Current position
            let cursor_x = pad_rect.min.x + value.0 as f32 * pad_width;
            let cursor_y = pad_rect.max.y - value.1 as f32 * pad_height; // Invert Y
            let cursor_pos = Pos2::new(cursor_x, cursor_y);

            // Crosshair lines
            if self.show_crosshair {
                let crosshair_stroke = Stroke::new(theme.stroke_width, cursor_color);
                // Horizontal line
                painter.line_segment(
                    [
                        Pos2::new(pad_rect.min.x, cursor_y),
                        Pos2::new(pad_rect.max.x, cursor_y),
                    ],
                    crosshair_stroke,
                );
                // Vertical line
                painter.line_segment(
                    [
                        Pos2::new(cursor_x, pad_rect.min.y),
                        Pos2::new(cursor_x, pad_rect.max.y),
                    ],
                    crosshair_stroke,
                );
            }

            // Cursor dot
            let dot_radius = theme.spacing_sm;
            painter.circle_filled(cursor_pos, dot_radius, cursor_color);
            painter.circle_stroke(
                cursor_pos,
                dot_radius,
                Stroke::new(theme.stroke_width, theme.bg_primary),
            );

            // Border
            painter.rect_stroke(
                pad_rect,
                theme.radius_md,
                Stroke::new(theme.border_width, border_color),
                egui::StrokeKind::Outside,
            );

            // X-axis label
            if let Some(label) = self.label_x {
                let label_pos = Pos2::new(
                    pad_rect.center().x,
                    rect.max.y - theme.font_size_xs / 2.0,
                );
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(theme.font_size_xs),
                    if self.disabled { theme.text_muted } else { theme.text_secondary },
                );
            }

            // Y-axis label (rotated text simulation - just draw at left)
            if let Some(label) = self.label_y {
                // For simplicity, draw vertically by placing each char
                // In a real app, you'd use proper text rotation
                let label_x = rect.min.x + theme.font_size_xs / 2.0;
                let label_y = pad_rect.center().y;

                // Draw label horizontally for now (egui doesn't easily support rotation)
                painter.text(
                    Pos2::new(label_x, label_y),
                    egui::Align2::CENTER_CENTER,
                    label.chars().next().unwrap_or(' '), // First char as indicator
                    egui::FontId::proportional(theme.font_size_xs),
                    if self.disabled { theme.text_muted } else { theme.text_secondary },
                );
            }

            // Value display (top right corner)
            let value_text = format!("({:.2}, {:.2})", value.0, value.1);
            painter.text(
                Pos2::new(pad_rect.max.x - theme.spacing_xs, pad_rect.min.y + theme.spacing_xs),
                egui::Align2::RIGHT_TOP,
                &value_text,
                egui::FontId::proportional(theme.font_size_xs),
                Color32::from_rgba_unmultiplied(
                    theme.text_secondary.r(),
                    theme.text_secondary.g(),
                    theme.text_secondary.b(),
                    if self.disabled { 80 } else { 180 },
                ),
            );
        }

        // Mark as changed if interacted
        if response.dragged() || response.clicked() || response.double_clicked() {
            response.mark_changed();
        }
        response
    }
}

impl Default for XYPad<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for XYPad<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut dummy = (0.5, 0.5);
        self.show_internal(ui, &mut dummy)
    }
}
