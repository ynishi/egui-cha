//! Card molecule

use egui::Ui;
use egui_cha::ViewCtx;

/// A card container with optional header
pub struct Card<'a> {
    title: Option<&'a str>,
    padding: f32,
}

impl<'a> Card<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            padding: 16.0,
        }
    }

    pub fn titled(title: &'a str) -> Self {
        Self {
            title: Some(title),
            padding: 16.0,
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Show card with content (ViewCtx version)
    pub fn show_ctx<Msg, R>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        content: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        ctx.group(|ctx| {
            if let Some(title) = self.title {
                ctx.ui.heading(title);
                ctx.ui.separator();
                ctx.ui.add_space(8.0);
            }
            content(ctx)
        })
    }

    /// Show card with content (Ui version)
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        egui::Frame::group(ui.style())
            .inner_margin(self.padding)
            .show(ui, |ui| {
                if let Some(title) = self.title {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(8.0);
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
