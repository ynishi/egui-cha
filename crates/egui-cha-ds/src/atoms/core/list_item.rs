//! ListItem atom - Selectable list item with optional icon and badge
//!
//! A versatile list item component for navigation menus, selection lists, etc.
//!
//! # Features
//! - Optional icon (left side)
//! - Optional badge (right side)
//! - Selection state with indicator
//! - Disabled state
//! - Three size variants (respects theme scaling)
//!
//! # Example
//! ```ignore
//! // Basic item
//! ListItem::new("Settings").on_click(ctx, Msg::GoSettings);
//!
//! // With icon and selection
//! ListItem::new("Home")
//!     .icon(icons::HOUSE)
//!     .selected(is_home)
//!     .on_click(ctx, Msg::GoHome);
//!
//! // Compact size for dense menus
//! ListItem::new("Item").compact().on_click(ctx, Msg::Select);
//!
//! // With badge
//! ListItem::new("Messages").badge("5").on_click(ctx, Msg::Go);
//! ```

use crate::Theme;
use egui::{FontFamily, Response, RichText, Ui, Widget};
use egui_cha::ViewCtx;

/// Size variant for ListItem
///
/// Sizes are based on theme spacing values and respect theme scaling.
/// At default scale (1.0):
/// - Compact: ~24px (matches button height)
/// - Medium: ~32px (default)
/// - Large: ~40px (touch-friendly)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListItemSize {
    /// Compact size (~24px), matches button height
    Compact,
    /// Default size (~32px)
    #[default]
    Medium,
    /// Large size (~40px), touch-friendly
    Large,
}

/// A selectable list item with optional icon and badge
///
/// Used by [`Menu`](crate::Menu) and [`IconMenu`](crate::IconMenu) internally,
/// but can also be used standalone for custom list layouts.
pub struct ListItem {
    label: String,
    icon: Option<&'static str>,
    badge: Option<String>,
    selected: bool,
    disabled: bool,
    size: ListItemSize,
}

impl ListItem {
    /// Create a new list item with label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            badge: None,
            selected: false,
            disabled: false,
            size: ListItemSize::default(),
        }
    }

    /// Set compact size (matches button height)
    pub fn compact(mut self) -> Self {
        self.size = ListItemSize::Compact;
        self
    }

    /// Set size variant
    pub fn size(mut self, size: ListItemSize) -> Self {
        self.size = size;
        self
    }

    /// Set the icon (from icons module)
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set a badge (e.g., count)
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Show and emit message on click (TEA style)
    pub fn on_click<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) {
        let disabled = self.disabled;
        if self.show(ctx.ui).clicked() && !disabled {
            ctx.emit(msg);
        }
    }

    /// Show the list item and return response
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for ListItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());

        // Colors based on state
        let (text_color, bg_color) = if self.disabled {
            (theme.text_muted, None)
        } else if self.selected {
            (theme.primary, Some(theme.bg_secondary))
        } else {
            (theme.text_primary, None)
        };

        let hover_color = theme.bg_tertiary;

        // Calculate size based on size variant (respects theme scaling)
        let desired_height = match self.size {
            ListItemSize::Compact => theme.spacing_md + theme.spacing_sm, // ~24px at 1.0 scale
            ListItemSize::Medium => theme.spacing_lg + theme.spacing_md,  // ~32px at 1.0 scale
            ListItemSize::Large => theme.spacing_xl + theme.spacing_md,   // ~40px at 1.0 scale
        };
        let available_width = ui.available_width();

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_width, desired_height),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            let bg = if response.hovered() && !self.disabled {
                Some(hover_color)
            } else {
                bg_color
            };

            if let Some(color) = bg {
                painter.rect_filled(rect, theme.radius_sm, color);
            }

            // Selected indicator (left border, respects stroke_scale)
            let indicator_width = theme.stroke_width * 3.0;
            if self.selected {
                let indicator_rect =
                    egui::Rect::from_min_size(rect.min, egui::vec2(indicator_width, rect.height()));
                painter.rect_filled(indicator_rect, 0.0, theme.primary);
            }

            // Content layout
            let padding = theme.spacing_md;
            let mut x = rect.min.x + padding + if self.selected { indicator_width } else { 0.0 };
            let center_y = rect.center().y;

            // Icon (uses font_size_md for scaling)
            if let Some(icon) = self.icon {
                let icon_text = RichText::new(icon)
                    .family(FontFamily::Name("icons".into()))
                    .color(text_color);
                let galley = painter.layout_no_wrap(
                    icon_text.text().to_string(),
                    egui::FontId::new(theme.font_size_md, FontFamily::Name("icons".into())),
                    text_color,
                );
                let icon_pos = egui::pos2(x, center_y - galley.size().y / 2.0);
                painter.galley(icon_pos, galley, text_color);
                x += theme.font_size_md + theme.spacing_sm;
            }

            // Label (uses font_size_sm for scaling)
            let galley = painter.layout_no_wrap(
                self.label.clone(),
                egui::FontId::proportional(theme.font_size_sm),
                text_color,
            );
            let label_pos = egui::pos2(x, center_y - galley.size().y / 2.0);
            painter.galley(label_pos, galley, text_color);

            // Badge (uses font_size_xs for scaling)
            if let Some(badge) = &self.badge {
                let galley = painter.layout_no_wrap(
                    badge.clone(),
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.primary_text,
                );
                let badge_width = galley.size().x + theme.spacing_sm * 2.0;
                let badge_height = galley.size().y + theme.spacing_xs;
                let badge_x = rect.max.x - padding - badge_width;
                let badge_rect = egui::Rect::from_min_size(
                    egui::pos2(badge_x, center_y - badge_height / 2.0),
                    egui::vec2(badge_width, badge_height),
                );
                painter.rect_filled(badge_rect, theme.radius_sm, theme.primary);
                let text_pos =
                    egui::pos2(badge_x + theme.spacing_sm, center_y - galley.size().y / 2.0);
                painter.galley(text_pos, galley, theme.primary_text);
            }
        }

        // Cursor
        if !self.disabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        response
    }
}
