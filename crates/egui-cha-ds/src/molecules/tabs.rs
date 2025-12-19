//! Tabs molecule

use egui::{Color32, RichText, Ui};
use egui_cha::ViewCtx;

/// Tab bar component
pub struct Tabs<'a> {
    tabs: &'a [&'a str],
}

impl<'a> Tabs<'a> {
    pub fn new(tabs: &'a [&'a str]) -> Self {
        Self { tabs }
    }

    /// TEA-style: Show tabs with current index, emit Msg on tab change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        active: usize,
        on_change: impl Fn(usize) -> Msg,
    ) {
        if let Some(new_idx) = self.render(ctx.ui, active) {
            ctx.emit(on_change(new_idx));
        }
    }

    /// Show tabs (modifies index in place)
    pub fn show(self, ui: &mut Ui, active: &mut usize) {
        if let Some(new_idx) = self.render(ui, *active) {
            *active = new_idx;
        }
    }

    /// Render tabs and return clicked index if any
    fn render(self, ui: &mut Ui, active: usize) -> Option<usize> {
        let is_dark = ui.ctx().style().visuals.dark_mode;

        let active_color = if is_dark {
            Color32::from_rgb(96, 165, 250)
        } else {
            Color32::from_rgb(59, 130, 246)
        };

        let inactive_color = if is_dark {
            Color32::from_rgb(156, 163, 175)
        } else {
            Color32::from_rgb(107, 114, 128)
        };

        let border_color = if is_dark {
            Color32::from_rgb(55, 65, 81)
        } else {
            Color32::from_rgb(229, 231, 235)
        };

        let mut clicked_idx: Option<usize> = None;

        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter().enumerate() {
                let is_active = i == active;

                let text_color = if is_active {
                    active_color
                } else {
                    inactive_color
                };

                let response =
                    ui.selectable_label(is_active, RichText::new(*tab).color(text_color));

                if response.clicked() && !is_active {
                    clicked_idx = Some(i);
                }
            }
        });

        // Bottom border
        ui.painter().hline(
            ui.available_rect_before_wrap().x_range(),
            ui.cursor().top(),
            egui::Stroke::new(1.0, border_color),
        );

        clicked_idx
    }
}

/// Tab panel - content container that shows based on active tab
pub struct TabPanel;

impl TabPanel {
    /// Show content only if this tab is active
    pub fn show(ui: &mut Ui, active: usize, index: usize, content: impl FnOnce(&mut Ui)) {
        if active == index {
            content(ui);
        }
    }

    /// Show content with ViewCtx only if this tab is active
    pub fn show_ctx<Msg>(
        ctx: &mut ViewCtx<'_, Msg>,
        active: usize,
        index: usize,
        content: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        if active == index {
            content(ctx);
        }
    }
}
