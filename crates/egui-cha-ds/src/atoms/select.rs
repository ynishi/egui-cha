//! Select/Dropdown atom

use egui::Ui;
use egui_cha::ViewCtx;

/// A dropdown select component
pub struct Select<'a, T> {
    options: &'a [(T, &'a str)],
    placeholder: Option<&'a str>,
    disabled: bool,
}

impl<'a, T> Select<'a, T>
where
    T: PartialEq + Clone,
{
    /// Create a new select with options as (value, label) pairs
    pub fn new(options: &'a [(T, &'a str)]) -> Self {
        Self {
            options,
            placeholder: None,
            disabled: false,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show select with immutable value, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        selected: Option<&T>,
        on_select: impl Fn(T) -> Msg,
    ) {
        let current_label = selected
            .and_then(|s| self.options.iter().find(|(v, _)| v == s).map(|(_, l)| *l))
            .or(self.placeholder)
            .unwrap_or("Select...");

        // Collect clicked value first
        let mut clicked_value: Option<T> = None;

        egui::ComboBox::from_id_salt(ctx.ui.next_auto_id())
            .selected_text(current_label)
            .show_ui(ctx.ui, |ui| {
                for (value, label) in self.options {
                    let is_selected = selected.map_or(false, |s| s == value);
                    if ui.selectable_label(is_selected, *label).clicked() && !self.disabled {
                        clicked_value = Some(value.clone());
                    }
                }
            });

        // Emit after ComboBox is done
        if let Some(value) = clicked_value {
            ctx.emit(on_select(value));
        }
    }

    /// Show select (modifies value in place)
    pub fn show(self, ui: &mut Ui, selected: &mut Option<T>) {
        let current_label = selected
            .as_ref()
            .and_then(|s| self.options.iter().find(|(v, _)| v == s).map(|(_, l)| *l))
            .or(self.placeholder)
            .unwrap_or("Select...");

        let mut clicked_value: Option<T> = None;

        egui::ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (value, label) in self.options {
                    let is_selected = selected.as_ref().map_or(false, |s| s == value);
                    if ui.selectable_label(is_selected, *label).clicked() && !self.disabled {
                        clicked_value = Some(value.clone());
                    }
                }
            });

        if let Some(value) = clicked_value {
            *selected = Some(value);
        }
    }
}
