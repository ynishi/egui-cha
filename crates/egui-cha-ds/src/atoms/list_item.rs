//! ListItem atom - Selectable list item with optional icon and badge

use crate::Theme;
use egui::{FontFamily, Response, RichText, Ui, Widget};
use egui_cha::ViewCtx;

/// A selectable list item with optional icon and badge
pub struct ListItem {
    label: String,
    icon: Option<&'static str>,
    badge: Option<String>,
    selected: bool,
    disabled: bool,
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
        }
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

        // Calculate size
        let desired_height = theme.spacing_lg * 2.0;
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

            // Selected indicator (left border)
            if self.selected {
                let indicator_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(3.0, rect.height()),
                );
                painter.rect_filled(indicator_rect, 0.0, theme.primary);
            }

            // Content layout
            let padding = theme.spacing_md;
            let mut x = rect.min.x + padding + if self.selected { 3.0 } else { 0.0 };
            let center_y = rect.center().y;

            // Icon
            if let Some(icon) = self.icon {
                let icon_text = RichText::new(icon)
                    .family(FontFamily::Name("icons".into()))
                    .color(text_color);
                let galley = painter.layout_no_wrap(
                    icon_text.text().to_string(),
                    egui::FontId::new(16.0, FontFamily::Name("icons".into())),
                    text_color,
                );
                let icon_pos = egui::pos2(x, center_y - galley.size().y / 2.0);
                painter.galley(icon_pos, galley, text_color);
                x += 20.0 + theme.spacing_sm;
            }

            // Label
            let galley = painter.layout_no_wrap(
                self.label.clone(),
                egui::FontId::proportional(14.0),
                text_color,
            );
            let label_pos = egui::pos2(x, center_y - galley.size().y / 2.0);
            painter.galley(label_pos, galley, text_color);

            // Badge (right side)
            if let Some(badge) = &self.badge {
                let galley = painter.layout_no_wrap(
                    badge.clone(),
                    egui::FontId::proportional(12.0),
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
                let text_pos = egui::pos2(
                    badge_x + theme.spacing_sm,
                    center_y - galley.size().y / 2.0,
                );
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
