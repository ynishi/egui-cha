//! Tooltip extension for egui Response

use egui::Response;

use crate::Theme;

/// Extension trait for adding themed tooltips to any Response
pub trait ResponseExt {
    /// Add a themed text tooltip that appears on hover (uses default delay ~0.5s)
    fn with_tooltip(self, text: impl Into<String>) -> Self;

    /// Add a themed tooltip that appears immediately on hover (no delay)
    fn with_tooltip_immediate(self, text: impl Into<String>) -> Self;

    /// Add a themed tooltip with custom delay in seconds
    fn with_tooltip_delayed(self, text: impl Into<String>, delay_secs: f32) -> Self;
}

impl ResponseExt for Response {
    fn with_tooltip(self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.on_hover_ui(|ui| {
            let theme = Theme::current(ui.ctx());
            ui.label(egui::RichText::new(&text).color(theme.text_primary));
        })
    }

    fn with_tooltip_immediate(self, text: impl Into<String>) -> Self {
        self.with_tooltip_delayed(text, 0.0)
    }

    fn with_tooltip_delayed(self, text: impl Into<String>, delay_secs: f32) -> Self {
        if !self.hovered() {
            return self;
        }

        let ctx = self.ctx.clone();
        let text = text.into();

        // Set custom delay before showing tooltip
        let old_delay = ctx.style().interaction.tooltip_delay;
        ctx.style_mut(|style| {
            style.interaction.tooltip_delay = delay_secs;
        });

        let result = self.on_hover_ui(|ui| {
            let theme = Theme::current(ui.ctx());
            ui.label(egui::RichText::new(&text).color(theme.text_primary));
        });

        // Restore original delay
        ctx.style_mut(|style| {
            style.interaction.tooltip_delay = old_delay;
        });

        result
    }
}
