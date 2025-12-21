//! Fader atom - Vertical fader control for EDM/VJ applications
//!
//! A vertical fader/slider commonly used in mixers, DAWs, and audio software.
//!
//! # Features
//! - Vertical drag control
//! - Optional label and value display
//! - dB scale support
//! - Multiple size variants
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! // Basic fader
//! Fader::new(0.0..=1.0)
//!     .show_with(ctx, model.volume, Msg::SetVolume);
//!
//! // With label and dB scale
//! Fader::new(-60.0..=6.0)
//!     .label("Master")
//!     .db_scale(true)
//!     .show_with(ctx, model.master_db, Msg::SetMaster);
//!
//! // Compact for channel strips
//! Fader::new(0.0..=1.0)
//!     .compact()
//!     .show_with(ctx, model.ch1, Msg::SetCh1);
//! ```

use crate::Theme;
use egui::{Response, Sense, Ui, Vec2, Widget};
use egui_cha::ViewCtx;
use std::ops::RangeInclusive;

/// Size variant for Fader
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaderSize {
    /// Compact size (narrow, short)
    Compact,
    /// Default size
    #[default]
    Medium,
    /// Large size (wide, tall)
    Large,
}

/// A vertical fader control
pub struct Fader<'a> {
    range: RangeInclusive<f64>,
    label: Option<&'a str>,
    size: FaderSize,
    show_value: bool,
    db_scale: bool,
    disabled: bool,
}

impl<'a> Fader<'a> {
    /// Create a new fader with the given range
    pub fn new(range: RangeInclusive<f64>) -> Self {
        Self {
            range,
            label: None,
            size: FaderSize::default(),
            show_value: true,
            db_scale: false,
            disabled: false,
        }
    }

    /// Set a label displayed below the fader
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set compact size
    pub fn compact(mut self) -> Self {
        self.size = FaderSize::Compact;
        self
    }

    /// Set size variant
    pub fn size(mut self, size: FaderSize) -> Self {
        self.size = size;
        self
    }

    /// Show or hide the value display
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Display value as dB (adds "dB" suffix)
    pub fn db_scale(mut self, enabled: bool) -> Self {
        self.db_scale = enabled;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show fader with immutable value, emit Msg on change
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

    /// Show fader (modifies value in place)
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> Response {
        self.show_internal(ui, value)
    }

    fn show_internal(self, ui: &mut Ui, value: &mut f64) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate dimensions based on size variant
        let (width, height) = match self.size {
            FaderSize::Compact => (theme.spacing_lg, theme.spacing_xl * 4.0),
            FaderSize::Medium => (theme.spacing_xl, theme.spacing_xl * 5.0),
            FaderSize::Large => (theme.spacing_xl + theme.spacing_md, theme.spacing_xl * 6.0),
        };

        // Additional space for label and value
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
        let total_height = height + label_height + value_height;

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(width, total_height),
            if self.disabled {
                Sense::hover()
            } else {
                Sense::click_and_drag()
            },
        );

        // Track area (the draggable part)
        let track_rect = egui::Rect::from_min_size(
            rect.min + Vec2::new(0.0, value_height),
            Vec2::new(width, height),
        );

        // Handle drag
        if response.dragged() && !self.disabled {
            let delta = response.drag_delta();
            // Vertical drag: up = increase, down = decrease
            let range_size = *self.range.end() - *self.range.start();
            let sensitivity = range_size / (height - theme.spacing_md) as f64;
            *value = (*value - delta.y as f64 * sensitivity)
                .clamp(*self.range.start(), *self.range.end());
        }

        // Click to set value directly
        if response.clicked() && !self.disabled {
            if let Some(pos) = response.interact_pointer_pos() {
                let relative_y = (track_rect.max.y - pos.y) / (height - theme.spacing_md);
                let range_size = *self.range.end() - *self.range.start();
                *value = (*self.range.start() + relative_y as f64 * range_size)
                    .clamp(*self.range.start(), *self.range.end());
            }
        }

        // Double-click to reset
        if response.double_clicked() && !self.disabled {
            // Reset to 0 for dB scale, or center for linear
            if self.db_scale {
                *value = 0.0_f64.clamp(*self.range.start(), *self.range.end());
            } else {
                *value = (*self.range.start() + *self.range.end()) / 2.0;
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Colors
            let (track_color, thumb_color, fill_color) = if self.disabled {
                (theme.bg_tertiary, theme.text_muted, theme.text_muted)
            } else if response.hovered() || response.dragged() {
                (theme.bg_tertiary, theme.primary_hover, theme.primary_hover)
            } else {
                (theme.bg_secondary, theme.primary, theme.primary)
            };

            // Track background
            let track_inner = track_rect.shrink(theme.spacing_xs);
            painter.rect_filled(track_inner, theme.radius_sm, track_color);

            // Track border
            painter.rect_stroke(
                track_inner,
                theme.radius_sm,
                egui::Stroke::new(theme.border_width, theme.border),
                egui::StrokeKind::Outside,
            );

            // Fill (from bottom to current value)
            let normalized =
                (*value - *self.range.start()) / (*self.range.end() - *self.range.start());
            let fill_height = normalized as f32 * (track_inner.height() - theme.spacing_sm);
            let fill_rect = egui::Rect::from_min_max(
                egui::Pos2::new(
                    track_inner.min.x + theme.spacing_xs,
                    track_inner.max.y - theme.spacing_xs / 2.0 - fill_height,
                ),
                egui::Pos2::new(
                    track_inner.max.x - theme.spacing_xs,
                    track_inner.max.y - theme.spacing_xs / 2.0,
                ),
            );
            if fill_height > 0.0 {
                let fill_alpha = if self.disabled { 80 } else { 150 };
                let fill_color_alpha = egui::Color32::from_rgba_unmultiplied(
                    fill_color.r(),
                    fill_color.g(),
                    fill_color.b(),
                    fill_alpha,
                );
                painter.rect_filled(fill_rect, theme.radius_sm * 0.5, fill_color_alpha);
            }

            // Thumb (horizontal line at current position)
            let thumb_y = track_inner.max.y - theme.spacing_xs / 2.0 - fill_height;
            let thumb_height = theme.spacing_sm;
            let thumb_rect = egui::Rect::from_min_max(
                egui::Pos2::new(track_inner.min.x, thumb_y - thumb_height / 2.0),
                egui::Pos2::new(track_inner.max.x, thumb_y + thumb_height / 2.0),
            );
            painter.rect_filled(thumb_rect, theme.radius_sm * 0.5, thumb_color);

            // Thumb grip lines
            let grip_color = if self.disabled {
                theme.bg_tertiary
            } else {
                theme.bg_primary
            };
            for i in [-1, 0, 1] {
                let y = thumb_y + i as f32 * 2.0;
                painter.line_segment(
                    [
                        egui::Pos2::new(thumb_rect.min.x + theme.spacing_xs, y),
                        egui::Pos2::new(thumb_rect.max.x - theme.spacing_xs, y),
                    ],
                    egui::Stroke::new(theme.stroke_width * 0.5, grip_color),
                );
            }

            // Value display (above track)
            if self.show_value {
                let value_text = if self.db_scale {
                    if *value <= *self.range.start() + 0.1 {
                        "-∞".to_string()
                    } else {
                        format!("{:.1}dB", value)
                    }
                } else if *self.range.end() - *self.range.start() > 10.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{:.2}", value)
                };

                let value_pos = rect.center_top() + Vec2::new(0.0, theme.font_size_xs / 2.0);
                painter.text(
                    value_pos,
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

            // Label (below track)
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
        if response.dragged() || response.clicked() || response.double_clicked() {
            response.mark_changed();
        }
        response
    }
}

impl Widget for Fader<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut dummy = 0.5;
        self.show_internal(ui, &mut dummy)
    }
}
