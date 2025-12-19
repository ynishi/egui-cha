//! SearchBar molecule

use egui::Ui;
use egui_cha::ViewCtx;

/// A search bar with input and button
pub struct SearchBar<'a> {
    placeholder: &'a str,
    button_text: &'a str,
}

impl<'a> SearchBar<'a> {
    pub fn new() -> Self {
        Self {
            placeholder: "Search...",
            button_text: "Search",
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn button_text(mut self, text: &'a str) -> Self {
        self.button_text = text;
        self
    }

    /// Show search bar with callbacks for input change and search submit
    ///
    /// - `on_change`: Called when text changes (for updating model)
    /// - `on_submit`: Called when Enter or button clicked
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: &str,
        on_change: impl FnOnce(String) -> Msg,
        on_submit: impl FnOnce(String) -> Msg,
    ) {
        let mut current = value.to_string();
        let mut should_search = false;

        ctx.horizontal(|ctx| {
            let response = ctx.ui.add(
                egui::TextEdit::singleline(&mut current)
                    .hint_text(self.placeholder)
                    .desired_width(200.0),
            );

            if response.lost_focus() && ctx.ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                should_search = true;
            }

            if ctx.ui.button(self.button_text).clicked() {
                should_search = true;
            }
        });

        // Emit change if text was modified
        if current != value {
            ctx.emit(on_change(current.clone()));
        }

        // Emit submit if search triggered
        if should_search && !current.is_empty() {
            ctx.emit(on_submit(current));
        }
    }

    /// Show search bar with callback on search (legacy - value must be mutable ref)
    pub fn on_search<Msg, F>(self, ctx: &mut ViewCtx<'_, Msg>, value: &mut String, to_msg: F)
    where
        F: FnOnce(String) -> Msg,
    {
        let mut should_search = false;

        ctx.horizontal(|ctx| {
            let response = ctx.ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text(self.placeholder)
                    .desired_width(200.0),
            );

            if response.lost_focus() && ctx.ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                should_search = true;
            }

            if ctx.ui.button(self.button_text).clicked() {
                should_search = true;
            }
        });

        if should_search && !value.is_empty() {
            ctx.emit(to_msg(value.clone()));
        }
    }

    /// Show without ctx (basic version)
    pub fn show(self, ui: &mut Ui, value: &mut String) -> bool {
        let mut submitted = false;

        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text(self.placeholder)
                    .desired_width(200.0),
            );

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                submitted = true;
            }

            if ui.button(self.button_text).clicked() {
                submitted = true;
            }
        });

        submitted && !value.is_empty()
    }
}

impl<'a> Default for SearchBar<'a> {
    fn default() -> Self {
        Self::new()
    }
}
