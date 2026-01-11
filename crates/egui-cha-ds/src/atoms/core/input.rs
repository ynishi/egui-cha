//! Input atom
//!
//! A styled text input component with Theme integration.
//!
//! # Example
//!
//! ```ignore
//! Input::new()
//!     .placeholder("Enter text...")
//!     .desired_width(200.0)
//!     .show(ui, &mut value);
//! ```

use crate::Theme;
use egui::Ui;
use egui_cha::ViewCtx;

/// A text input component with Theme styling
pub struct Input<'a> {
    placeholder: &'a str,
    password: bool,
    desired_width: Option<f32>,
}

impl<'a> Input<'a> {
    pub fn new() -> Self {
        Self {
            placeholder: "",
            password: false,
            desired_width: None,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    /// TEA-style: Show input with immutable value, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: &str,
        on_change: impl FnOnce(String) -> Msg,
    ) {
        let theme = Theme::current(ctx.ui.ctx());
        let mut current = value.to_string();

        let mut edit = egui::TextEdit::singleline(&mut current)
            .hint_text(self.placeholder)
            .text_color(theme.text_primary)
            .frame(false);

        if self.password {
            edit = edit.password(true);
        }

        if let Some(width) = self.desired_width {
            edit = edit.desired_width(width);
        }

        // Custom frame with theme styling
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, theme.border))
            .corner_radius(theme.radius_sm)
            .fill(theme.bg_primary)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ctx.ui, |ui| {
                ui.add(edit);
            });

        if current != value {
            ctx.emit(on_change(current));
        }
    }

    /// Legacy: Show input and emit msg on change (requires &mut)
    pub fn on_change<Msg, F>(self, ctx: &mut ViewCtx<'_, Msg>, value: &mut String, to_msg: F)
    where
        F: FnOnce(String) -> Msg,
    {
        let old_value = value.clone();
        self.show(ctx.ui, value);
        if *value != old_value {
            ctx.emit(to_msg(value.clone()));
        }
    }

    /// Show the input (modifies value in place)
    pub fn show(self, ui: &mut Ui, value: &mut String) {
        let theme = Theme::current(ui.ctx());

        let mut edit = egui::TextEdit::singleline(value)
            .hint_text(self.placeholder)
            .text_color(theme.text_primary)
            .frame(false);

        if self.password {
            edit = edit.password(true);
        }

        if let Some(width) = self.desired_width {
            edit = edit.desired_width(width);
        }

        // Custom frame with theme styling
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, theme.border))
            .corner_radius(theme.radius_sm)
            .fill(theme.bg_primary)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.add(edit);
            });
    }
}

impl<'a> Default for Input<'a> {
    fn default() -> Self {
        Self::new()
    }
}
