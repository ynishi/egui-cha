//! Link/Hyperlink atom

use egui::{Color32, RichText, Ui};
use egui_cha::ViewCtx;

/// A hyperlink component
pub struct Link<'a> {
    text: &'a str,
    url: Option<&'a str>,
}

impl<'a> Link<'a> {
    /// Create a link that opens a URL
    pub fn new(text: &'a str, url: &'a str) -> Self {
        Self {
            text,
            url: Some(url),
        }
    }

    /// Create a clickable link (for internal navigation)
    pub fn clickable(text: &'a str) -> Self {
        Self { text, url: None }
    }

    /// Show as external hyperlink (opens in browser)
    pub fn show(self, ui: &mut Ui) -> bool {
        let is_dark = ui.ctx().style().visuals.dark_mode;
        let color = if is_dark {
            Color32::from_rgb(96, 165, 250)
        } else {
            Color32::from_rgb(59, 130, 246)
        };

        if let Some(url) = self.url {
            ui.hyperlink_to(RichText::new(self.text).color(color), url)
                .clicked()
        } else {
            let response = ui.link(RichText::new(self.text).color(color));
            response.clicked()
        }
    }

    /// Show link and emit Msg on click (for internal navigation)
    pub fn on_click<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) -> bool {
        let is_dark = ctx.ui.ctx().style().visuals.dark_mode;
        let color = if is_dark {
            Color32::from_rgb(96, 165, 250)
        } else {
            Color32::from_rgb(59, 130, 246)
        };

        let response = ctx.ui.link(RichText::new(self.text).color(color));
        if response.clicked() {
            ctx.emit(msg);
            true
        } else {
            false
        }
    }
}
