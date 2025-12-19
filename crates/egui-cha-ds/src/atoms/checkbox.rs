//! Checkbox atom

use egui::Ui;
use egui_cha::ViewCtx;

/// A checkbox component for TEA architecture
pub struct Checkbox<'a> {
    label: &'a str,
    disabled: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show checkbox with immutable value, emit Msg on toggle
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        checked: bool,
        on_toggle: impl FnOnce(bool) -> Msg,
    ) {
        let mut current = checked;
        let response = ctx.ui.add_enabled(
            !self.disabled,
            egui::Checkbox::new(&mut current, self.label),
        );

        if response.changed() {
            ctx.emit(on_toggle(current));
        }
    }

    /// Simple: emit msg when toggled (for simple bool toggle)
    pub fn on_toggle<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, checked: bool, msg: Msg)
    where
        Msg: Clone,
    {
        self.show_with(ctx, checked, |_| msg);
    }

    /// Show checkbox (modifies value in place)
    pub fn show(self, ui: &mut Ui, checked: &mut bool) -> bool {
        let response = ui.add_enabled(!self.disabled, egui::Checkbox::new(checked, self.label));
        response.changed()
    }
}
