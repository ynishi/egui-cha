//! Validated Input atom
//!
//! A text input with validation state display.
//!
//! # Example
//!
//! ```ignore
//! ValidatedInput::new("Email")
//!     .placeholder("user@example.com")
//!     .show(&mut model.email, &model.email_validation, ctx);
//! ```

use egui::{RichText, Ui};
use egui_cha::ViewCtx;

use crate::{icons, Theme};

/// Validation state for form inputs
#[derive(Debug, Clone, Default)]
pub enum ValidationState {
    /// Not yet validated
    #[default]
    None,
    /// Validation passed
    Valid,
    /// Validation failed with error message
    Invalid(String),
}

impl ValidationState {
    /// Create a valid state
    pub fn valid() -> Self {
        Self::Valid
    }

    /// Create an invalid state with message
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    /// Check if state is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Check if state is invalid
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    /// Get error message if invalid
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Invalid(msg) => Some(msg),
            _ => None,
        }
    }
}

/// A text input with validation state
pub struct ValidatedInput<'a> {
    label: &'a str,
    placeholder: &'a str,
    password: bool,
    desired_width: Option<f32>,
}

impl<'a> ValidatedInput<'a> {
    /// Create a new validated input with label
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            placeholder: "",
            password: false,
            desired_width: None,
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Make this a password input
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Set desired width
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    /// Show the input with validation state
    pub fn show(self, value: &mut String, state: &ValidationState, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());

        // Label
        ui.label(self.label);

        // Input with border color based on state
        let border_color = match state {
            ValidationState::None => theme.border,
            ValidationState::Valid => theme.state_success,
            ValidationState::Invalid(_) => theme.state_danger,
        };

        let mut edit = egui::TextEdit::singleline(value)
            .hint_text(self.placeholder)
            .text_color(theme.text_primary)
            .frame(false);  // Disable default frame

        if self.password {
            edit = edit.password(true);
        }

        if let Some(width) = self.desired_width {
            edit = edit.desired_width(width);
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

        // Validation indicator and message
        match state {
            ValidationState::Valid => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icons::CHECK)
                            .family(egui::FontFamily::Name("icons".into()))
                            .color(theme.state_success)
                            .size(14.0),
                    );
                    ui.label(RichText::new("Valid").color(theme.state_success).small());
                });
            }
            ValidationState::Invalid(msg) => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icons::WARNING)
                            .family(egui::FontFamily::Name("icons".into()))
                            .color(theme.state_danger)
                            .size(14.0),
                    );
                    ui.label(RichText::new(msg).color(theme.state_danger).small());
                });
            }
            ValidationState::None => {}
        }
    }

    /// TEA-style: Show with immutable value, emit on change
    pub fn show_with<Msg>(
        self,
        value: &str,
        state: &ValidationState,
        ctx: &mut ViewCtx<'_, Msg>,
        on_change: impl FnOnce(String) -> Msg,
    ) {
        let theme = Theme::current(ctx.ui.ctx());
        let mut current = value.to_string();

        // Label
        ctx.ui.label(self.label);

        // Border color based on state
        let border_color = match state {
            ValidationState::None => theme.border,
            ValidationState::Valid => theme.state_success,
            ValidationState::Invalid(_) => theme.state_danger,
        };

        let mut edit = egui::TextEdit::singleline(&mut current)
            .hint_text(self.placeholder)
            .text_color(theme.text_primary)
            .frame(false);  // Disable default frame

        if self.password {
            edit = edit.password(true);
        }

        if let Some(width) = self.desired_width {
            edit = edit.desired_width(width);
        }

        // Custom frame with validation color
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, border_color))
            .corner_radius(theme.radius_sm)
            .fill(theme.bg_primary)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ctx.ui, |ui| {
                ui.add(edit);
            });

        // Emit on change
        if current != value {
            ctx.emit(on_change(current));
        }

        // Validation indicator
        match state {
            ValidationState::Valid => {
                ctx.ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icons::CHECK)
                            .family(egui::FontFamily::Name("icons".into()))
                            .color(theme.state_success)
                            .size(14.0),
                    );
                    ui.label(RichText::new("Valid").color(theme.state_success).small());
                });
            }
            ValidationState::Invalid(msg) => {
                ctx.ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icons::WARNING)
                            .family(egui::FontFamily::Name("icons".into()))
                            .color(theme.state_danger)
                            .size(14.0),
                    );
                    ui.label(RichText::new(msg).color(theme.state_danger).small());
                });
            }
            ValidationState::None => {}
        }
    }
}
