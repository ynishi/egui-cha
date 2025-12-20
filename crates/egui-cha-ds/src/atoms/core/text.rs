//! Text atom - Styled text components with semantic variants
//!
//! Provides consistent typography across the application using theme tokens.
//!
//! # Examples
//!
//! ```rust
//! use egui_cha_ds::atoms::Text;
//!
//! // Headings
//! Text::h1("Page Title").show(ui);
//! Text::h2("Section Title").show(ui);
//! Text::h3("Subsection").show(ui);
//!
//! // Body text
//! Text::body("Regular paragraph text").show(ui);
//! Text::small("Smaller text").show(ui);
//! Text::caption("Caption or hint text").muted().show(ui);
//!
//! // With modifiers
//! Text::body("Important").bold().show(ui);
//! Text::body("Secondary info").muted().show(ui);
//! Text::body("Error message").color(theme.state_danger).show(ui);
//! ```

use egui::{Color32, RichText, Ui, Widget};

use crate::Theme;

/// Text size variants based on theme typography tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextSize {
    /// Extra small (font_size_xs) - for fine print
    Xs,
    /// Small (font_size_sm) - for captions, hints
    Sm,
    /// Medium/default (font_size_md) - for body text
    #[default]
    Md,
    /// Large (font_size_lg) - for emphasis
    Lg,
    /// Extra large (font_size_xl) - for h3
    Xl,
    /// 2x large (font_size_2xl) - for h2
    Xl2,
    /// 3x large (font_size_3xl) - for h1
    Xl3,
}

impl TextSize {
    /// Get the font size from theme
    pub fn to_size(self, theme: &Theme) -> f32 {
        match self {
            TextSize::Xs => theme.font_size_xs,
            TextSize::Sm => theme.font_size_sm,
            TextSize::Md => theme.font_size_md,
            TextSize::Lg => theme.font_size_lg,
            TextSize::Xl => theme.font_size_xl,
            TextSize::Xl2 => theme.font_size_2xl,
            TextSize::Xl3 => theme.font_size_3xl,
        }
    }
}

/// A styled text component
#[derive(Clone)]
pub struct Text<'a> {
    content: &'a str,
    size: TextSize,
    color: Option<Color32>,
    bold: bool,
    italic: bool,
    strikethrough: bool,
    underline: bool,
}

impl<'a> Text<'a> {
    /// Create text with default (medium) size
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            size: TextSize::Md,
            color: None,
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
        }
    }

    // === Semantic constructors ===

    /// Heading 1 - largest heading (font_size_3xl, bold)
    pub fn h1(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Xl3).bold()
    }

    /// Heading 2 - large heading (font_size_2xl, bold)
    pub fn h2(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Xl2).bold()
    }

    /// Heading 3 - medium heading (font_size_xl, bold)
    pub fn h3(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Xl).bold()
    }

    /// Body text - default size
    pub fn body(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Md)
    }

    /// Small text
    pub fn small(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Sm)
    }

    /// Caption/hint text - extra small, muted
    pub fn caption(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Xs)
    }

    /// Large emphasized text
    pub fn large(content: &'a str) -> Self {
        Self::new(content).size(TextSize::Lg)
    }

    // === Modifiers (builder pattern) ===

    /// Set text size
    pub fn size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }

    /// Set custom color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Use muted (secondary) color from theme
    pub fn muted(mut self) -> Self {
        // Will be resolved in show() using theme
        self.color = None; // Mark for muted resolution
        self
    }

    /// Make text bold
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Make text italic
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Add strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Add underline
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Display the text
    pub fn show(self, ui: &mut Ui) {
        ui.add(self);
    }

    /// Build RichText with theme
    fn to_rich_text(&self, theme: &Theme) -> RichText {
        let mut text = RichText::new(self.content).size(self.size.to_size(theme));

        // Apply color (default to text_primary if not specified)
        let color = self.color.unwrap_or(theme.text_primary);
        text = text.color(color);

        // Apply styles
        if self.bold {
            text = text.strong();
        }
        if self.italic {
            text = text.italics();
        }
        if self.strikethrough {
            text = text.strikethrough();
        }
        if self.underline {
            text = text.underline();
        }

        text
    }
}

impl<'a> Widget for Text<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let theme = Theme::current(ui.ctx());
        let rich_text = self.to_rich_text(&theme);
        ui.label(rich_text)
    }
}

/// Extension methods for creating muted text variants
impl<'a> Text<'a> {
    /// Create text with secondary (muted) color
    pub fn secondary(content: &'a str) -> Self {
        Self::new(content) // Color resolved in widget impl
    }
}

/// Builder for creating text with explicit muted color
pub struct MutedText<'a> {
    inner: Text<'a>,
}

impl<'a> MutedText<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            inner: Text::new(content),
        }
    }

    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let text = self.inner.color(theme.text_muted);
        ui.add(text);
    }
}

impl<'a> Widget for MutedText<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let theme = Theme::current(ui.ctx());
        let text = self.inner.color(theme.text_muted);
        ui.add(text)
    }
}
