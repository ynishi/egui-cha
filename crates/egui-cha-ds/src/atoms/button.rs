//! Button atom

use egui::{Color32, Response, RichText, Stroke, Ui, Widget};
use egui_cha::ViewCtx;

/// Button variant for different styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
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

        // Get dark_mode from Context (not Ui) since theme.apply() updates Context style
        let is_dark = ui.ctx().style().visuals.dark_mode;
        let (fill, text_color, stroke) = self.variant_style(is_dark);

        let rich_text = RichText::new(text).color(text_color);
        let mut button = egui::Button::new(rich_text).fill(fill);

        if let Some(s) = stroke {
            button = button.stroke(s);
        }

        let response = ui.add_enabled(!self.disabled, button);
        response.clicked()
    }

    /// Get style colors for variant
    fn variant_style(&self, is_dark: bool) -> (Color32, Color32, Option<Stroke>) {
        match self.variant {
            ButtonVariant::Primary => {
                let bg = if is_dark {
                    Color32::from_rgb(96, 165, 250) // primary dark
                } else {
                    Color32::from_rgb(59, 130, 246) // primary light
                };
                let fg = if is_dark {
                    Color32::from_rgb(17, 24, 39) // dark text on light button
                } else {
                    Color32::WHITE
                };
                (bg, fg, None)
            }
            ButtonVariant::Secondary => {
                let bg = if is_dark {
                    Color32::from_rgb(55, 65, 81) // bg_tertiary dark
                } else {
                    Color32::from_rgb(107, 114, 128) // secondary light
                };
                let fg = if is_dark {
                    Color32::from_rgb(249, 250, 251) // text_primary dark
                } else {
                    Color32::WHITE
                };
                (bg, fg, None)
            }
            ButtonVariant::Outline => {
                let border = if is_dark {
                    Color32::from_rgb(156, 163, 175) // text_muted
                } else {
                    Color32::from_rgb(107, 114, 128)
                };
                let fg = if is_dark {
                    Color32::from_rgb(249, 250, 251)
                } else {
                    Color32::from_rgb(17, 24, 39)
                };
                (Color32::TRANSPARENT, fg, Some(Stroke::new(1.0, border)))
            }
            ButtonVariant::Ghost => {
                let fg = if is_dark {
                    Color32::from_rgb(209, 213, 219) // text_secondary dark
                } else {
                    Color32::from_rgb(75, 85, 99)
                };
                (Color32::TRANSPARENT, fg, None)
            }
            ButtonVariant::Danger => {
                let bg = if is_dark {
                    Color32::from_rgb(248, 113, 113) // error dark
                } else {
                    Color32::from_rgb(239, 68, 68) // error light
                };
                let fg = if is_dark {
                    Color32::from_rgb(17, 24, 39)
                } else {
                    Color32::WHITE
                };
                (bg, fg, None)
            }
        }
    }
}

impl<'a> Widget for Button<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = match self.icon {
            Some(icon) => format!("{} {}", icon, self.label),
            None => self.label.to_string(),
        };

        // Get dark_mode from Context (not Ui) since theme.apply() updates Context style
        let is_dark = ui.ctx().style().visuals.dark_mode;
        let (fill, text_color, stroke) = self.variant_style(is_dark);

        let rich_text = RichText::new(text).color(text_color);
        let mut button = egui::Button::new(rich_text).fill(fill);

        if let Some(s) = stroke {
            button = button.stroke(s);
        }

        ui.add_enabled(!self.disabled, button)
    }
}
