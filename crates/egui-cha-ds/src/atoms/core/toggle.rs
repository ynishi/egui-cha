//! Toggle/Switch atom

use egui::{Color32, Response, Sense, Ui, Vec2};
use egui_cha::ViewCtx;

/// A toggle switch component
pub struct Toggle<'a> {
    label: Option<&'a str>,
    disabled: bool,
}

impl<'a> Toggle<'a> {
    pub fn new() -> Self {
        Self {
            label: None,
            disabled: false,
        }
    }

    pub fn with_label(label: &'a str) -> Self {
        Self {
            label: Some(label),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show toggle with immutable value, emit Msg on toggle
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on: bool,
        on_toggle: impl FnOnce(bool) -> Msg,
    ) {
        let response = self.render(ctx.ui, on);
        if response.clicked() && !self.disabled {
            ctx.emit(on_toggle(!on));
        }
    }

    /// Simple: emit msg when toggled
    pub fn on_toggle<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, on: bool, msg: Msg)
    where
        Msg: Clone,
    {
        self.show_with(ctx, on, |_| msg);
    }

    /// Show toggle (returns true if clicked)
    pub fn show(self, ui: &mut Ui, on: &mut bool) -> bool {
        let response = self.render(ui, *on);
        if response.clicked() && !self.disabled {
            *on = !*on;
            true
        } else {
            false
        }
    }

    fn render(&self, ui: &mut Ui, on: bool) -> Response {
        let is_dark = ui.ctx().style().visuals.dark_mode;

        ui.horizontal(|ui| {
            // Draw toggle switch
            let desired_size = Vec2::new(40.0, 20.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();

                // Background
                let bg_color = if on {
                    if is_dark {
                        Color32::from_rgb(96, 165, 250) // primary
                    } else {
                        Color32::from_rgb(59, 130, 246)
                    }
                } else if is_dark {
                    Color32::from_rgb(55, 65, 81)
                } else {
                    Color32::from_rgb(209, 213, 219)
                };

                let bg_color = if self.disabled {
                    bg_color.gamma_multiply(0.5)
                } else {
                    bg_color
                };

                painter.rect_filled(rect, rect.height() / 2.0, bg_color);

                // Knob
                let knob_radius = rect.height() / 2.0 - 2.0;
                let knob_x = if on {
                    rect.right() - knob_radius - 2.0
                } else {
                    rect.left() + knob_radius + 2.0
                };
                let knob_pos = egui::pos2(knob_x, rect.center().y);
                let knob_color = Color32::WHITE;
                painter.circle_filled(knob_pos, knob_radius, knob_color);
            }

            // Label
            if let Some(label) = self.label {
                ui.label(label);
            }

            response
        })
        .inner
    }
}

impl<'a> Default for Toggle<'a> {
    fn default() -> Self {
        Self::new()
    }
}
