//! Button atom

use egui::{Color32, Response, RichText, Stroke, Ui, Widget};
use egui_cha::ViewCtx;

use crate::Theme;

/// Button variant for different styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
    Warning,
    Success,
    Info,
}

/// A styled button component
#[derive(Clone)]
pub struct Button<'a> {
    label: &'a str,
    variant: ButtonVariant,
    disabled: bool,
    icon: Option<&'a str>,
}

impl<'a> Button<'a> {
    /// Create a new primary button
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Primary,
            disabled: false,
            icon: None,
        }
    }

    /// Create a primary variant button
    pub fn primary(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Primary)
    }

    /// Create a secondary variant button
    pub fn secondary(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Secondary)
    }

    /// Create an outline variant button
    pub fn outline(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Outline)
    }

    /// Create a ghost variant button
    pub fn ghost(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Ghost)
    }

    /// Create a danger variant button
    pub fn danger(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Danger)
    }

    /// Create a warning variant button
    pub fn warning(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Warning)
    }

    /// Create a success variant button
    pub fn success(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Success)
    }

    /// Create an info variant button
    pub fn info(label: &'a str) -> Self {
        Self::new(label).variant(ButtonVariant::Info)
    }

    /// Set the variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Add an icon prefix
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Show the button and emit msg on click
    pub fn on_click<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) -> bool {
        let clicked = self.show(ctx.ui);
        if clicked {
            ctx.emit(msg);
        }
        clicked
    }

    /// Show the button (returns true if clicked)
    pub fn show(self, ui: &mut Ui) -> bool {
        let text = match self.icon {
            Some(icon) => format!("{} {}", icon, self.label),
            None => self.label.to_string(),
        };

        let theme = Theme::current(ui.ctx());
        let (fill, text_color, stroke) = self.variant_style(&theme);

        let rich_text = RichText::new(text).color(text_color);
        let mut button = egui::Button::new(rich_text).fill(fill);

        if let Some(s) = stroke {
            button = button.stroke(s);
        }

        let response = ui.add_enabled(!self.disabled, button);
        response.clicked()
    }

    /// Get style colors for variant from theme
    fn variant_style(&self, theme: &Theme) -> (Color32, Color32, Option<Stroke>) {
        match self.variant {
            ButtonVariant::Primary => (theme.primary, theme.primary_text, None),
            ButtonVariant::Secondary => (theme.secondary, theme.secondary_text, None),
            ButtonVariant::Outline => (
                Color32::TRANSPARENT,
                theme.text_primary,
                Some(Stroke::new(1.0, theme.border)),
            ),
            ButtonVariant::Ghost => (Color32::TRANSPARENT, theme.text_secondary, None),
            ButtonVariant::Danger => (theme.state_danger, theme.state_danger_text, None),
            ButtonVariant::Warning => (theme.state_warning, theme.state_warning_text, None),
            ButtonVariant::Success => (theme.state_success, theme.state_success_text, None),
            ButtonVariant::Info => (theme.state_info, theme.state_info_text, None),
        }
    }
}

impl<'a> Widget for Button<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = match self.icon {
            Some(icon) => format!("{} {}", icon, self.label),
            None => self.label.to_string(),
        };

        let theme = Theme::current(ui.ctx());
        let (fill, text_color, stroke) = self.variant_style(&theme);

        let rich_text = RichText::new(text).color(text_color);
        let mut button = egui::Button::new(rich_text).fill(fill);

        if let Some(s) = stroke {
            button = button.stroke(s);
        }

        ui.add_enabled(!self.disabled, button)
    }
}
