//! Badge atom

use egui::{Color32, RichText, Ui};

use crate::Theme;

/// Badge variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

/// A badge/tag component
pub struct Badge<'a> {
    text: &'a str,
    variant: BadgeVariant,
}

impl<'a> Badge<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            variant: BadgeVariant::Default,
        }
    }

    pub fn success(text: &'a str) -> Self {
        Self::new(text).variant(BadgeVariant::Success)
    }

    pub fn warning(text: &'a str) -> Self {
        Self::new(text).variant(BadgeVariant::Warning)
    }

    pub fn error(text: &'a str) -> Self {
        Self::new(text).variant(BadgeVariant::Error)
    }

    pub fn info(text: &'a str) -> Self {
        Self::new(text).variant(BadgeVariant::Info)
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let (bg_color, text_color) = self.variant_colors(&theme);

        egui::Frame::new()
            .fill(bg_color)
            .corner_radius(theme.radius_sm)
            .inner_margin(egui::Margin::symmetric(
                theme.spacing_sm as i8,
                (theme.spacing_xs / 2.0) as i8,
            ))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.text)
                        .color(text_color)
                        .size(theme.font_size_xs),
                );
            });
    }

    /// Get badge colors from theme
    /// Note: Badge uses "subtle" style (light bg, dark text) unlike buttons
    fn variant_colors(&self, theme: &Theme) -> (Color32, Color32) {
        let is_dark = theme.variant == crate::ThemeVariant::Dark;

        match self.variant {
            BadgeVariant::Default => {
                (theme.bg_tertiary, theme.text_secondary)
            }
            BadgeVariant::Success => {
                // Subtle style: light green bg, dark green text
                if is_dark {
                    (darken(theme.state_success, 0.3), lighten(theme.state_success, 0.7))
                } else {
                    (lighten(theme.state_success, 0.85), darken(theme.state_success, 0.3))
                }
            }
            BadgeVariant::Warning => {
                if is_dark {
                    (darken(theme.state_warning, 0.3), lighten(theme.state_warning, 0.7))
                } else {
                    (lighten(theme.state_warning, 0.85), darken(theme.state_warning, 0.4))
                }
            }
            BadgeVariant::Error => {
                if is_dark {
                    (darken(theme.state_danger, 0.3), lighten(theme.state_danger, 0.7))
                } else {
                    (lighten(theme.state_danger, 0.85), darken(theme.state_danger, 0.3))
                }
            }
            BadgeVariant::Info => {
                if is_dark {
                    (darken(theme.state_info, 0.3), lighten(theme.state_info, 0.7))
                } else {
                    (lighten(theme.state_info, 0.85), darken(theme.state_info, 0.3))
                }
            }
        }
    }
}

/// Lighten a color by mixing with white
fn lighten(color: Color32, amount: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        (r as f32 + (255.0 - r as f32) * amount) as u8,
        (g as f32 + (255.0 - g as f32) * amount) as u8,
        (b as f32 + (255.0 - b as f32) * amount) as u8,
        a,
    )
}

/// Darken a color by mixing with black
fn darken(color: Color32, amount: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        (r as f32 * (1.0 - amount)) as u8,
        (g as f32 * (1.0 - amount)) as u8,
        (b as f32 * (1.0 - amount)) as u8,
        a,
    )
}
