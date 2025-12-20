//! Code/CodeBlock atom

use egui::{Color32, FontId, RichText, Ui};

/// Inline code styling
pub struct Code<'a> {
    text: &'a str,
}

impl<'a> Code<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn show(self, ui: &mut Ui) {
        let is_dark = ui.ctx().style().visuals.dark_mode;

        let bg = if is_dark {
            Color32::from_rgb(55, 65, 81)
        } else {
            Color32::from_rgb(243, 244, 246)
        };

        let fg = if is_dark {
            Color32::from_rgb(248, 113, 113) // reddish for code
        } else {
            Color32::from_rgb(220, 38, 38)
        };

        egui::Frame::new()
            .fill(bg)
            .corner_radius(3.0)
            .inner_margin(egui::Margin::symmetric(4, 1))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.text)
                        .color(fg)
                        .font(FontId::monospace(13.0)),
                );
            });
    }
}

/// Code block for multi-line code
pub struct CodeBlock<'a> {
    code: &'a str,
    language: Option<&'a str>,
}

impl<'a> CodeBlock<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            code,
            language: None,
        }
    }

    pub fn language(mut self, lang: &'a str) -> Self {
        self.language = Some(lang);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let is_dark = ui.ctx().style().visuals.dark_mode;

        let bg = if is_dark {
            Color32::from_rgb(31, 41, 55)
        } else {
            Color32::from_rgb(249, 250, 251)
        };

        let fg = if is_dark {
            Color32::from_rgb(229, 231, 235)
        } else {
            Color32::from_rgb(31, 41, 55)
        };

        let border = if is_dark {
            Color32::from_rgb(55, 65, 81)
        } else {
            Color32::from_rgb(229, 231, 235)
        };

        egui::Frame::new()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, border))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                // Language label
                if let Some(lang) = self.language {
                    ui.label(RichText::new(lang).small().color(if is_dark {
                        Color32::from_rgb(156, 163, 175)
                    } else {
                        Color32::from_rgb(107, 114, 128)
                    }));
                    ui.add_space(4.0);
                }

                // Code content
                ui.label(
                    RichText::new(self.code)
                        .color(fg)
                        .font(FontId::monospace(13.0)),
                );
            });
    }
}
