//! Input atom

use egui::Ui;
use egui_cha::ViewCtx;

/// A text input component
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
        let mut current = value.to_string();

        let mut edit = egui::TextEdit::singleline(&mut current).hint_text(self.placeholder);

        if self.password {
            edit = edit.password(true);
        }

        if let Some(width) = self.desired_width {
            edit = edit.desired_width(width);
        }

        ctx.ui.add(edit);

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
        let mut edit = egui::TextEdit::singleline(value).hint_text(self.placeholder);

        if self.password {
            edit = edit.password(true);
        }

        if let Some(width) = self.desired_width {
            edit = edit.desired_width(width);
        }

        ui.add(edit);
    }
}

impl<'a> Default for Input<'a> {
    fn default() -> Self {
        Self::new()
    }
}
