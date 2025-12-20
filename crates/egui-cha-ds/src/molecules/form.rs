//! Form molecule - Combines ValidatedInput atoms for form layouts
//!
//! # Example
//!
//! ```ignore
//! Form::new()
//!     .field("Email", &model.email, &model.email_state, Msg::EmailChanged)
//!     .password_field("Password", &model.pw, &model.pw_state, Msg::PwChanged)
//!     .submit_button("Sign Up")
//!     .submit_if(model.can_submit())
//!     .on_submit(Msg::Submit)
//!     .show(ctx);
//! ```

use crate::atoms::ValidationState;
use crate::theme::Theme;
use egui_cha::ViewCtx;

/// A form field definition with TEA-style callback
pub struct FormField<'a, Msg> {
    label: &'a str,
    value: &'a str,
    state: &'a ValidationState,
    on_change: Box<dyn FnOnce(String) -> Msg + 'a>,
    password: bool,
    placeholder: &'a str,
}

/// Form builder for creating validated forms (TEA-style)
pub struct Form<'a, Msg> {
    fields: Vec<FormField<'a, Msg>>,
    submit_text: &'a str,
    on_submit: Option<Msg>,
    submit_enabled: bool,
    spacing: f32,
}

impl<'a, Msg: Clone> Form<'a, Msg> {
    /// Create a new form
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            submit_text: "Submit",
            on_submit: None,
            submit_enabled: true,
            spacing: 12.0,
        }
    }

    /// Add a text field with change callback
    pub fn field(
        mut self,
        label: &'a str,
        value: &'a str,
        state: &'a ValidationState,
        on_change: impl FnOnce(String) -> Msg + 'a,
    ) -> Self {
        self.fields.push(FormField {
            label,
            value,
            state,
            on_change: Box::new(on_change),
            password: false,
            placeholder: "",
        });
        self
    }

    /// Add a password field with change callback
    pub fn password_field(
        mut self,
        label: &'a str,
        value: &'a str,
        state: &'a ValidationState,
        on_change: impl FnOnce(String) -> Msg + 'a,
    ) -> Self {
        self.fields.push(FormField {
            label,
            value,
            state,
            on_change: Box::new(on_change),
            password: true,
            placeholder: "",
        });
        self
    }

    /// Set submit button text
    pub fn submit_button(mut self, text: &'a str) -> Self {
        self.submit_text = text;
        self
    }

    /// Set condition for submit button to be enabled
    pub fn submit_if(mut self, condition: bool) -> Self {
        self.submit_enabled = condition;
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

    /// Show the form (TEA-style)
    pub fn show(self, ctx: &mut ViewCtx<'_, Msg>) {
        let theme = Theme::current(ctx.ui.ctx());

        // Collect messages to emit after UI rendering
        let mut messages: Vec<Msg> = Vec::new();

        ctx.ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = self.spacing;

            // Render each field
            for field in self.fields {
                let mut current = field.value.to_string();
                let original = current.clone();

                // Label
                ui.label(field.label);

                // Border color based on state
                let border_color = match field.state {
                    ValidationState::None => theme.border,
                    ValidationState::Valid => theme.state_success,
                    ValidationState::Invalid(_) => theme.state_danger,
                };

                let mut edit = egui::TextEdit::singleline(&mut current)
                    .hint_text(field.placeholder)
                    .text_color(theme.text_primary)
                    .frame(false);

                if field.password {
                    edit = edit.password(true);
                }

                // Custom frame with validation color
                egui::Frame::new()
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .corner_radius(theme.radius_sm)
                    .fill(theme.bg_primary)
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.add(edit);
                    });

                // Collect change message
                if current != original {
                    messages.push((field.on_change)(current));
                }

                // Validation indicator
                match field.state {
                    ValidationState::Valid => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(crate::icons::CHECK)
                                    .family(egui::FontFamily::Name("icons".into()))
                                    .color(theme.state_success)
                                    .size(14.0),
                            );
                            ui.label(egui::RichText::new("Valid").color(theme.state_success).small());
                        });
                    }
                    ValidationState::Invalid(msg) => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(crate::icons::WARNING)
                                    .family(egui::FontFamily::Name("icons".into()))
                                    .color(theme.state_danger)
                                    .size(14.0),
                            );
                            ui.label(egui::RichText::new(msg).color(theme.state_danger).small());
                        });
                    }
                    ValidationState::None => {}
                }
            }

            ui.add_space(8.0);

            // Submit button
            let button_text = egui::RichText::new(self.submit_text)
                .color(if self.submit_enabled {
                    theme.primary_text
                } else {
                    theme.text_muted
                })
                .strong();

            let button = egui::Button::new(button_text)
                .fill(if self.submit_enabled {
                    theme.primary
                } else {
                    theme.bg_secondary
                })
                .corner_radius(theme.radius_sm)
                .min_size(egui::vec2(ui.available_width(), 36.0));

            let response = ui.add_enabled(self.submit_enabled, button);

            if response.clicked() {
                if let Some(msg) = self.on_submit {
                    messages.push(msg);
                }
            }
        });

        // Emit all collected messages
        for msg in messages {
            ctx.emit(msg);
        }
    }
}

impl<'a, Msg: Clone> Default for Form<'a, Msg> {
    fn default() -> Self {
        Self::new()
    }
}
