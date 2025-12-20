//! Tooltip extension for egui Response

use egui::Response;

use crate::Theme;

/// Extension trait for adding themed tooltips to any Response
pub trait ResponseExt {
    /// Add a themed text tooltip that appears on hover
    fn with_tooltip(self, text: impl Into<String>) -> Self;
}

impl ResponseExt for Response {
    fn with_tooltip(self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.on_hover_ui(|ui| {
            let theme = Theme::current(ui.ctx());
            ui.label(egui::RichText::new(&text).color(theme.text_primary));
        })
    }
}
