//! Card molecule

use egui::Ui;
use egui_cha::ViewCtx;

use crate::Theme;

/// A card container with optional header
pub struct Card<'a> {
    title: Option<&'a str>,
    padding: Option<f32>,
}

impl<'a> Card<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            padding: None,
        }
    }

    pub fn titled(title: &'a str) -> Self {
        Self {
            title: Some(title),
            padding: None,
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Show card with content (ViewCtx version)
    pub fn show_ctx<Msg, R>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        content: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        let theme = Theme::current(ctx.ui.ctx());
        ctx.group(|ctx| {
            if let Some(title) = self.title {
                ctx.ui.heading(title);
                ctx.ui.separator();
                ctx.ui.add_space(theme.spacing_sm);
            }
            content(ctx)
        })
    }

    /// Show card with content (Ui version)
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let theme = Theme::current(ui.ctx());
        let padding = self.padding.unwrap_or(theme.spacing_md);
        egui::Frame::group(ui.style())
            .inner_margin(padding)
            .show(ui, |ui| {
                if let Some(title) = self.title {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(theme.spacing_sm);
                }
                content(ui)
            })
            .inner
    }
}

impl<'a> Default for Card<'a> {
    fn default() -> Self {
        Self::new()
    }
}
