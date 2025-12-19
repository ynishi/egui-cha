//! Badge atom

use egui::{Color32, RichText, Ui};

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
        let is_dark = ui.ctx().style().visuals.dark_mode;
        let (bg_color, text_color) = self.variant_colors(is_dark);

        egui::Frame::new()
            .fill(bg_color)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(8, 2))
            .show(ui, |ui| {
                ui.label(RichText::new(self.text).color(text_color).small());
            });
    }

    fn variant_colors(&self, is_dark: bool) -> (Color32, Color32) {
        match self.variant {
            BadgeVariant::Default => {
                if is_dark {
                    (
                        Color32::from_rgb(55, 65, 81),
                        Color32::from_rgb(209, 213, 219),
                    )
                } else {
                    (
                        Color32::from_rgb(229, 231, 235),
                        Color32::from_rgb(55, 65, 81),
                    )
                }
            }
            BadgeVariant::Success => {
                if is_dark {
                    (
                        Color32::from_rgb(22, 101, 52),
                        Color32::from_rgb(187, 247, 208),
                    )
                } else {
                    (
                        Color32::from_rgb(220, 252, 231),
                        Color32::from_rgb(22, 101, 52),
                    )
                }
            }
            BadgeVariant::Warning => {
                if is_dark {
                    (
                        Color32::from_rgb(133, 77, 14),
                        Color32::from_rgb(254, 240, 138),
                    )
                } else {
                    (
                        Color32::from_rgb(254, 249, 195),
                        Color32::from_rgb(133, 77, 14),
                    )
                }
            }
            BadgeVariant::Error => {
                if is_dark {
                    (
                        Color32::from_rgb(153, 27, 27),
                        Color32::from_rgb(254, 202, 202),
                    )
                } else {
                    (
                        Color32::from_rgb(254, 226, 226),
                        Color32::from_rgb(153, 27, 27),
                    )
                }
            }
            BadgeVariant::Info => {
                if is_dark {
                    (
                        Color32::from_rgb(30, 64, 175),
                        Color32::from_rgb(191, 219, 254),
                    )
                } else {
                    (
                        Color32::from_rgb(219, 234, 254),
                        Color32::from_rgb(30, 64, 175),
                    )
                }
            }
        }
    }
}
