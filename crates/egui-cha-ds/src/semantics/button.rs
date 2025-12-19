//! Semantic buttons - Domain-specific button components
//!
//! These buttons carry specific meaning and have **fixed labels/icons**.
//! This ensures UI consistency across the entire application.
//!
//! ## Design Philosophy
//!
//! - **Atoms**: Pure style primitives (primary, danger) - label is caller's choice
//! - **Semantics**: Domain-meaningful actions with fixed labels
//!
//! ## Usage
//!
//! ```ignore
//! use egui_cha_ds::semantics::{self, ButtonStyle};
//!
//! // Different display styles
//! semantics::save(ButtonStyle::Icon).on_click(ctx, Msg::Save);  // 💾
//! semantics::save(ButtonStyle::Text).on_click(ctx, Msg::Save);  // Save
//! semantics::save(ButtonStyle::Both).on_click(ctx, Msg::Save);  // 💾 Save
//!
//! // Using with egui directly
//! if semantics::delete(ButtonStyle::Both).show(ui) {
//!     // handle delete
//! }
//! ```

use crate::atoms::icons;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, RichText, Stroke, Ui};
use egui_cha::ViewCtx;

/// Button display style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    /// Icon only (compact)
    #[default]
    Icon,
    /// Text label only
    Text,
    /// Icon + Text (most explicit)
    Both,
}

/// A semantic button with fixed label and icon
pub struct SemanticButton {
    icon: &'static str,
    label: &'static str,
    style: ButtonStyle,
    variant: SemanticVariant,
}

/// Semantic variant determines the visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum SemanticVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Success,
}

impl SemanticButton {
    fn new(icon: &'static str, label: &'static str, style: ButtonStyle) -> Self {
        Self {
            icon,
            label,
            style,
            variant: SemanticVariant::Primary,
        }
    }

    fn with_variant(mut self, variant: SemanticVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Show the button and emit msg on click (TEA style)
    pub fn on_click<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) -> bool {
        let clicked = self.show(ctx.ui);
        if clicked {
            ctx.emit(msg);
        }
        clicked
    }

    /// Show the button (returns true if clicked)
    pub fn show(self, ui: &mut Ui) -> bool {
        let is_dark = ui.ctx().style().visuals.dark_mode;
        let (fill, text_color, stroke) = self.variant_style(is_dark);

        let button = match self.style {
            ButtonStyle::Icon => {
                // Icon only - use icons font family
                let content = RichText::new(self.icon)
                    .family(egui::FontFamily::Name("icons".into()))
                    .color(text_color);
                egui::Button::new(content).fill(fill)
            }
            ButtonStyle::Text => {
                // Text only - use default font
                let content = RichText::new(self.label).color(text_color);
                egui::Button::new(content).fill(fill)
            }
            ButtonStyle::Both => {
                // Icon + Text with mixed fonts using LayoutJob
                let mut job = LayoutJob::default();

                // Icon part (icons font)
                job.append(
                    self.icon,
                    0.0,
                    TextFormat {
                        font_id: FontId::new(14.0, egui::FontFamily::Name("icons".into())),
                        color: text_color,
                        ..Default::default()
                    },
                );

                // Space
                job.append(
                    " ",
                    0.0,
                    TextFormat {
                        font_id: FontId::default(),
                        color: text_color,
                        ..Default::default()
                    },
                );

                // Text part (default font)
                job.append(
                    self.label,
                    0.0,
                    TextFormat {
                        font_id: FontId::default(),
                        color: text_color,
                        ..Default::default()
                    },
                );

                egui::Button::new(job).fill(fill)
            }
        };

        let mut button = button;
        if let Some(s) = stroke {
            button = button.stroke(s);
        }

        ui.add(button).clicked()
    }

    /// Get style colors for variant
    fn variant_style(&self, is_dark: bool) -> (Color32, Color32, Option<Stroke>) {
        match self.variant {
            SemanticVariant::Primary => {
                let bg = if is_dark {
                    Color32::from_rgb(96, 165, 250)
                } else {
                    Color32::from_rgb(59, 130, 246)
                };
                let fg = if is_dark {
                    Color32::from_rgb(17, 24, 39)
                } else {
                    Color32::WHITE
                };
                (bg, fg, None)
            }
            SemanticVariant::Secondary => {
                let bg = if is_dark {
                    Color32::from_rgb(55, 65, 81)
                } else {
                    Color32::from_rgb(107, 114, 128)
                };
                let fg = if is_dark {
                    Color32::from_rgb(249, 250, 251)
                } else {
                    Color32::WHITE
                };
                (bg, fg, None)
            }
            SemanticVariant::Danger => {
                let bg = if is_dark {
                    Color32::from_rgb(248, 113, 113)
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                let fg = if is_dark {
                    Color32::from_rgb(17, 24, 39)
                } else {
                    Color32::WHITE
                };
                (bg, fg, None)
            }
            SemanticVariant::Success => {
                let bg = if is_dark {
                    Color32::from_rgb(74, 222, 128)
                } else {
                    Color32::from_rgb(34, 197, 94)
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

// =============================================================================
// File Operations
// =============================================================================

/// Save button
pub fn save(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::FLOPPY_DISK, "Save", style)
}

/// Close button
pub fn close(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::X, "Close", style).with_variant(SemanticVariant::Secondary)
}

/// Delete button (danger styled)
pub fn delete(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::TRASH, "Delete", style).with_variant(SemanticVariant::Danger)
}

/// Edit button
pub fn edit(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::PENCIL_SIMPLE, "Edit", style)
}

// =============================================================================
// Common Actions
// =============================================================================

/// Add button
pub fn add(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::PLUS, "Add", style)
}

/// Remove button (danger styled)
pub fn remove(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::MINUS, "Remove", style).with_variant(SemanticVariant::Danger)
}

/// Search button
pub fn search(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::MAGNIFYING_GLASS, "Search", style)
}

/// Refresh button
pub fn refresh(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::ARROWS_CLOCKWISE, "Refresh", style)
        .with_variant(SemanticVariant::Secondary)
}

/// Settings button
pub fn settings(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::GEAR, "Settings", style).with_variant(SemanticVariant::Secondary)
}

// =============================================================================
// Media
// =============================================================================

/// Play button
pub fn play(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::PLAY, "Play", style).with_variant(SemanticVariant::Success)
}

/// Pause button
pub fn pause(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::PAUSE, "Pause", style).with_variant(SemanticVariant::Secondary)
}

/// Stop button
pub fn stop(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::STOP, "Stop", style).with_variant(SemanticVariant::Danger)
}

// =============================================================================
// Clipboard
// =============================================================================

/// Copy button
pub fn copy(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::COPY, "Copy", style).with_variant(SemanticVariant::Secondary)
}

// =============================================================================
// Navigation
// =============================================================================

/// Back button
pub fn back(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::ARROW_LEFT, "Back", style).with_variant(SemanticVariant::Secondary)
}

/// Forward button
pub fn forward(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::ARROW_RIGHT, "Forward", style)
        .with_variant(SemanticVariant::Secondary)
}

// =============================================================================
// Confirmation
// =============================================================================

/// Confirm/OK button
pub fn confirm(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::CHECK, "Confirm", style).with_variant(SemanticVariant::Success)
}

/// Cancel button
pub fn cancel(style: ButtonStyle) -> SemanticButton {
    SemanticButton::new(icons::X, "Cancel", style).with_variant(SemanticVariant::Secondary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_style_text_generation() {
        // Icon style
        let btn = save(ButtonStyle::Icon);
        assert_eq!(btn.icon, icons::FLOPPY_DISK);
        assert_eq!(btn.label, "Save");

        // Text style
        let btn = delete(ButtonStyle::Text);
        assert_eq!(btn.label, "Delete");

        // Both style
        let btn = edit(ButtonStyle::Both);
        assert_eq!(btn.icon, icons::PENCIL_SIMPLE);
        assert_eq!(btn.label, "Edit");
    }

    #[test]
    fn test_semantic_variants() {
        assert_eq!(save(ButtonStyle::Icon).variant, SemanticVariant::Primary);
        assert_eq!(delete(ButtonStyle::Icon).variant, SemanticVariant::Danger);
        assert_eq!(close(ButtonStyle::Icon).variant, SemanticVariant::Secondary);
    }
}
