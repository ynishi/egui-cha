//! Link/Hyperlink atom

use crate::theme::Theme;
use egui::{RichText, Ui};
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
        let color = Theme::current(ui.ctx()).primary;

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
        let color = Theme::current(ctx.ui.ctx()).primary;

        let response = ctx.ui.link(RichText::new(self.text).color(color));
        if response.clicked() {
            ctx.emit(msg);
            true
        } else {
            false
        }
    }
}
