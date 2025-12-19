//! Form molecule - Combines ValidatedInput atoms for form layouts

use crate::atoms::{ValidatedInput, ValidationState};
use crate::theme::Theme;
use egui::Ui;

/// A form field definition
pub struct FormField<'a> {
    label: &'a str,
    value: &'a mut String,
    state: &'a ValidationState,
    password: bool,
    placeholder: &'a str,
}

impl<'a> FormField<'a> {
    /// Create a new form field
    pub fn new(label: &'a str, value: &'a mut String, state: &'a ValidationState) -> Self {
        Self {
            label,
            value,
            state,
            password: false,
            placeholder: "",
        }
    }

    /// Set password mode
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

/// Form builder for creating validated forms
pub struct Form<'a, Msg> {
    fields: Vec<FormField<'a>>,
    submit_text: &'a str,
    on_submit: Option<Msg>,
    spacing: f32,
}

impl<'a, Msg: Clone> Form<'a, Msg> {
    /// Create a new form
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            submit_text: "Submit",
            on_submit: None,
            spacing: 12.0,
        }
    }

    /// Add a text field
    pub fn field(
        mut self,
        label: &'a str,
        value: &'a mut String,
        state: &'a ValidationState,
    ) -> Self {
        self.fields.push(FormField::new(label, value, state));
        self
    }

    /// Add a password field
    pub fn password_field(
        mut self,
        label: &'a str,
        value: &'a mut String,
        state: &'a ValidationState,
    ) -> Self {
        self.fields
            .push(FormField::new(label, value, state).password());
        self
    }

    /// Add a custom field
    pub fn custom_field(mut self, field: FormField<'a>) -> Self {
        self.fields.push(field);
        self
    }

    /// Set submit button text
    pub fn submit_button(mut self, text: &'a str) -> Self {
        self.submit_text = text;
        self
    }

    /// Set the message to emit on submit
    pub fn on_submit(mut self, msg: Msg) -> Self {
        self.on_submit = Some(msg);
        self
    }

    /// Set vertical spacing between fields
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Show the form and return optional message
    pub fn show(self, ui: &mut Ui, theme: &Theme) -> Option<Msg> {
        let mut result = None;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = self.spacing;

            // Render each field
            for field in self.fields {
                let mut input = ValidatedInput::new(field.label)
                    .placeholder(field.placeholder);

                if field.password {
                    input = input.password();
                }

                input.show(field.value, field.state, ui);
            }

            ui.add_space(8.0);

            // Submit button
            let button = egui::Button::new(
                egui::RichText::new(self.submit_text)
                    .color(theme.primary_text)
                    .strong(),
            )
            .fill(theme.primary)
            .corner_radius(theme.radius_sm)
            .min_size(egui::vec2(ui.available_width(), 36.0));

            if ui.add(button).clicked() {
                result = self.on_submit;
            }
        });

        result
    }
}

impl<'a, Msg: Clone> Default for Form<'a, Msg> {
    fn default() -> Self {
        Self::new()
    }
}
