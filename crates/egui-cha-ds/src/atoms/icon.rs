//! Icon atom - Phosphor Icons integration

use egui::{Color32, Label, Response, RichText, Ui, Widget};
use egui_cha::ViewCtx;

/// Phosphor Icons codepoints
pub mod icons {
    // Navigation
    pub const HOUSE: &str = "\u{e2c2}";
    pub const ARROW_LEFT: &str = "\u{e058}";
    pub const ARROW_RIGHT: &str = "\u{e06c}";

    // Actions
    pub const PLUS: &str = "\u{e3d4}";
    pub const MINUS: &str = "\u{e32a}";
    pub const X: &str = "\u{e4f6}";
    pub const CHECK: &str = "\u{e182}";

    // UI
    pub const GEAR: &str = "\u{e270}";
    pub const INFO: &str = "\u{e2ce}";
    pub const WARNING: &str = "\u{e4e0}";
    pub const HASH: &str = "\u{e2a2}";
    pub const USER: &str = "\u{e4c2}";
}

/// Icon component using Phosphor Icons
pub struct Icon {
    icon_char: &'static str,
    size: f32,
    color: Option<Color32>,
}

impl Icon {
    /// Create a new icon
    pub fn new(icon_char: &'static str) -> Self {
        Self {
            icon_char,
            size: 16.0,
            color: None,
        }
    }

    // Convenience constructors
    pub fn house() -> Self {
        Self::new(icons::HOUSE)
    }
    pub fn arrow_left() -> Self {
        Self::new(icons::ARROW_LEFT)
    }
    pub fn arrow_right() -> Self {
        Self::new(icons::ARROW_RIGHT)
    }
    pub fn plus() -> Self {
        Self::new(icons::PLUS)
    }
    pub fn minus() -> Self {
        Self::new(icons::MINUS)
    }
    pub fn x() -> Self {
        Self::new(icons::X)
    }
    pub fn check() -> Self {
        Self::new(icons::CHECK)
    }
    pub fn gear() -> Self {
        Self::new(icons::GEAR)
    }
    pub fn info() -> Self {
        Self::new(icons::INFO)
    }
    pub fn warning() -> Self {
        Self::new(icons::WARNING)
    }
    pub fn hash() -> Self {
        Self::new(icons::HASH)
    }
    pub fn user() -> Self {
        Self::new(icons::USER)
    }

    /// Set icon size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set icon color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Show the icon
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }

    /// Show the icon using ViewCtx
    pub fn show_ctx<Msg>(self, ctx: &mut ViewCtx<Msg>) -> Response {
        ctx.ui.add(self)
    }
}

impl Widget for Icon {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut text = RichText::new(self.icon_char)
            .size(self.size)
            .family(egui::FontFamily::Name("icons".into()));

        if let Some(color) = self.color {
            text = text.color(color);
        }

        ui.add(Label::new(text).selectable(false))
    }
}
