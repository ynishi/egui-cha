//! BPM Display atom - Large numeric display for tempo/BPM
//!
//! A prominent numeric display commonly seen in DAWs and DJ software.
//! Shows BPM or other numeric values in a large, easy-to-read format.
//!
//! # Example
//! ```ignore
//! // Basic BPM display
//! BpmDisplay::new()
//!     .show(ui, 128.0);
//!
//! // With label and tap tempo
//! BpmDisplay::new()
//!     .label("BPM")
//!     .show_with(ctx, model.bpm, Msg::TapTempo);
//! ```

use crate::Theme;
use egui::{Color32, Response, Sense, Ui, Vec2};
use egui_cha::ViewCtx;

/// BPM display size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplaySize {
    /// Compact display
    Compact,
    /// Medium display (default)
    #[default]
    Medium,
    /// Large display
    Large,
}

/// BPM display style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayStyle {
    /// Modern flat style (default)
    #[default]
    Modern,
    /// LED segment style
    Segment,
    /// Minimal style
    Minimal,
}

/// Large numeric display for BPM/tempo
pub struct BpmDisplay<'a> {
    label: Option<&'a str>,
    size: DisplaySize,
    style: DisplayStyle,
    decimals: usize,
    min_digits: usize,
    blinking: bool,
    color: Option<Color32>,
}

impl<'a> Default for BpmDisplay<'a> {
    fn default() -> Self {
        Self {
            label: None,
            size: DisplaySize::default(),
            style: DisplayStyle::default(),
            decimals: 1,
            min_digits: 3,
            blinking: false,
            color: None,
        }
    }
}

impl<'a> BpmDisplay<'a> {
    /// Create a new BPM display
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the label (displayed above the value)
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the size variant
    pub fn size(mut self, size: DisplaySize) -> Self {
        self.size = size;
        self
    }

    /// Use compact size
    pub fn compact(mut self) -> Self {
        self.size = DisplaySize::Compact;
        self
    }

    /// Use large size
    pub fn large(mut self) -> Self {
        self.size = DisplaySize::Large;
        self
    }

    /// Set the display style
    pub fn style(mut self, style: DisplayStyle) -> Self {
        self.style = style;
        self
    }

    /// Use LED segment style
    pub fn segment(mut self) -> Self {
        self.style = DisplayStyle::Segment;
        self
    }

    /// Use minimal style
    pub fn minimal(mut self) -> Self {
        self.style = DisplayStyle::Minimal;
        self
    }

    /// Set number of decimal places
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Set minimum number of digits (pads with leading spaces)
    pub fn min_digits(mut self, digits: usize) -> Self {
        self.min_digits = digits;
        self
    }

    /// Enable blinking effect (for sync indicator)
    pub fn blinking(mut self, enabled: bool) -> Self {
        self.blinking = enabled;
        self
    }

    /// Set custom color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// TEA-style: Show display, emit Msg on tap (for tap tempo)
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: f64,
        on_tap: impl FnOnce() -> Msg,
    ) {
        let response = self.show_internal(ctx.ui, value);
        if response.clicked() {
            ctx.emit(on_tap());
        }
    }

    /// Display the BPM value
    pub fn show(self, ui: &mut Ui, value: f64) -> Response {
        self.show_internal(ui, value)
    }

    fn show_internal(self, ui: &mut Ui, value: f64) -> Response {
        let theme = Theme::current(ui.ctx());

        // Calculate dimensions based on size
        let (font_size, label_size, padding) = match self.size {
            DisplaySize::Compact => (theme.font_size_2xl, theme.font_size_xs, theme.spacing_sm),
            DisplaySize::Medium => (
                theme.font_size_3xl * 1.5,
                theme.font_size_sm,
                theme.spacing_md,
            ),
            DisplaySize::Large => (
                theme.font_size_3xl * 2.0,
                theme.font_size_md,
                theme.spacing_lg,
            ),
        };

        // Format the value
        let value_text = if self.decimals > 0 {
            format!(
                "{:>width$.prec$}",
                value,
                width = self.min_digits + 1 + self.decimals,
                prec = self.decimals
            )
        } else {
            format!("{:>width$.0}", value, width = self.min_digits)
        };

        // Calculate total height
        let label_height = if self.label.is_some() {
            label_size + theme.spacing_xs
        } else {
            0.0
        };

        // Measure text width
        let text_width = ui.fonts_mut(|f| {
            f.glyph_width(&egui::FontId::monospace(font_size), '0') * value_text.len() as f32
        });

        let total_width = text_width + padding * 2.0;
        let total_height = font_size + label_height + padding * 2.0;

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background and style-specific rendering
            let (bg_color, text_color) = match self.style {
                DisplayStyle::Modern => {
                    let bg = theme.bg_secondary;
                    let fg = self.color.unwrap_or(theme.primary);
                    (bg, fg)
                }
                DisplayStyle::Segment => {
                    let bg = Color32::from_rgb(20, 25, 20);
                    let fg = self.color.unwrap_or(Color32::from_rgb(0, 255, 100));
                    (bg, fg)
                }
                DisplayStyle::Minimal => {
                    let bg = Color32::TRANSPARENT;
                    let fg = self.color.unwrap_or(theme.text_primary);
                    (bg, fg)
                }
            };

            // Apply blinking effect
            let text_color = if self.blinking {
                let time = ui.input(|i| i.time);
                let blink = (time * 2.0).sin() > 0.0;
                if blink {
                    text_color
                } else {
                    Color32::from_rgba_unmultiplied(
                        text_color.r(),
                        text_color.g(),
                        text_color.b(),
                        100,
                    )
                }
            } else {
                text_color
            };

            // Draw background
            if bg_color != Color32::TRANSPARENT {
                painter.rect_filled(rect, theme.radius_md, bg_color);
            }

            // Draw segment-style shadow (dim segments)
            if matches!(self.style, DisplayStyle::Segment) {
                let shadow_color = Color32::from_rgba_unmultiplied(
                    text_color.r(),
                    text_color.g(),
                    text_color.b(),
                    25,
                );
                let shadow_text = "8".repeat(value_text.len());
                let text_pos = egui::pos2(rect.center().x, rect.max.y - padding - font_size / 2.0);
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    &shadow_text,
                    egui::FontId::monospace(font_size),
                    shadow_color,
                );
            }

            // Draw label
            if let Some(label) = self.label {
                let label_pos =
                    egui::pos2(rect.center().x, rect.min.y + padding + label_size / 2.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(label_size),
                    theme.text_secondary,
                );
            }

            // Draw value
            let value_y = if self.label.is_some() {
                rect.max.y - padding - font_size / 2.0
            } else {
                rect.center().y
            };
            let value_pos = egui::pos2(rect.center().x, value_y);

            painter.text(
                value_pos,
                egui::Align2::CENTER_CENTER,
                &value_text,
                egui::FontId::monospace(font_size),
                text_color,
            );

            // Hover effect
            if response.hovered() {
                painter.rect_stroke(
                    rect,
                    theme.radius_md,
                    egui::Stroke::new(theme.border_width, theme.primary),
                    egui::StrokeKind::Inside,
                );
            }
        }

        // Request repaint if blinking
        if self.blinking {
            ui.ctx().request_repaint();
        }

        response
    }
}
