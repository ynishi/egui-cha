//! Code/CodeBlock atom
//!
//! Theme-aware code display components with proper scaling support.

use crate::theme::Theme;
use egui::{FontId, RichText, Ui};

/// Inline code styling
///
/// Uses theme for colors, spacing, and font sizing.
pub struct Code<'a> {
    text: &'a str,
}

impl<'a> Code<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());

        // Use tertiary background for inline code
        let bg = theme.bg_tertiary;
        // Use state_danger for code highlight (reddish)
        let fg = theme.state_danger;

        egui::Frame::new()
            .fill(bg)
            .corner_radius(theme.radius_sm)
            .inner_margin(egui::Margin::symmetric(
                (theme.spacing_xs * 0.75) as i8,
                1,
            ))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.text)
                        .color(fg)
                        .font(FontId::monospace(theme.font_size_sm)),
                );
            });
    }
}

/// Code block for multi-line code
///
/// Uses theme for colors, spacing, borders, and font sizing.
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
        let theme = Theme::current(ui.ctx());

        // Use secondary background for code blocks
        let bg = theme.bg_secondary;
        // Use primary text for code content
        let fg = theme.text_primary;
        // Use border color from theme
        let border = theme.border;

        egui::Frame::new()
            .fill(bg)
            .stroke(egui::Stroke::new(theme.border_width, border))
            .corner_radius(theme.radius_md)
            .inner_margin(egui::Margin::same(theme.spacing_sm as i8))
            .show(ui, |ui| {
                // Language label
                if let Some(lang) = self.language {
                    ui.label(
                        RichText::new(lang)
                            .size(theme.font_size_xs)
                            .color(theme.text_muted),
                    );
                    ui.add_space(theme.spacing_xs);
                }

                // Code content
                ui.label(
                    RichText::new(self.code)
                        .color(fg)
                        .font(FontId::monospace(theme.font_size_sm)),
                );
            });
    }
}
