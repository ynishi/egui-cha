//! Modal/Dialog molecule

use egui::{Align2, Area, Color32, Frame, Id, Order, RichText, Ui, Vec2};

/// A modal dialog component
pub struct Modal<'a> {
    title: Option<&'a str>,
    width: f32,
    closable: bool,
}

impl<'a> Modal<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            width: 400.0,
            closable: true,
        }
    }

    pub fn titled(title: &'a str) -> Self {
        Self {
            title: Some(title),
            width: 400.0,
            closable: true,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Show modal (returns true if close was requested)
    pub fn show(self, ui: &mut Ui, open: bool, content: impl FnOnce(&mut Ui)) -> bool {
        if !open {
            return false;
        }

        let is_dark = ui.ctx().style().visuals.dark_mode;
        let mut close_requested = false;

        // Backdrop
        let screen_rect = ui.ctx().screen_rect();
        let backdrop_color = if is_dark {
            Color32::from_rgba_unmultiplied(0, 0, 0, 180)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        };

        Area::new(Id::new("modal_backdrop"))
            .fixed_pos(screen_rect.min)
            .order(Order::Foreground)
            .show(ui.ctx(), |ui| {
                let response = ui.allocate_response(screen_rect.size(), egui::Sense::click());
                ui.painter().rect_filled(screen_rect, 0.0, backdrop_color);

                // Close on backdrop click if closable
                if self.closable && response.clicked() {
                    close_requested = true;
                }
            });

        // Modal window
        let bg_color = if is_dark {
            Color32::from_rgb(31, 41, 55)
        } else {
            Color32::WHITE
        };

        let border_color = if is_dark {
            Color32::from_rgb(55, 65, 81)
        } else {
            Color32::from_rgb(229, 231, 235)
        };

        Area::new(Id::new("modal_content"))
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .order(Order::Foreground)
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::same(20))
                    .show(ui, |ui| {
                        ui.set_width(self.width);

                        // Header
                        if self.title.is_some() || self.closable {
                            ui.horizontal(|ui| {
                                if let Some(title) = self.title {
                                    ui.label(RichText::new(title).strong().size(18.0));
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if self.closable && ui.button("✕").clicked() {
                                            close_requested = true;
                                        }
                                    },
                                );
                            });
                            ui.add_space(12.0);
                        }

                        // Content
                        content(ui);
                    });
            });

        close_requested
    }
}

impl<'a> Default for Modal<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience for confirmation dialogs
pub struct ConfirmDialog<'a> {
    title: &'a str,
    message: &'a str,
    confirm_text: &'a str,
    cancel_text: &'a str,
    danger: bool,
}

/// Result of showing a confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    None,
    Confirmed,
    Cancelled,
}

impl<'a> ConfirmDialog<'a> {
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            confirm_text: "Confirm",
            cancel_text: "Cancel",
            danger: false,
        }
    }

    pub fn confirm_text(mut self, text: &'a str) -> Self {
        self.confirm_text = text;
        self
    }

    pub fn cancel_text(mut self, text: &'a str) -> Self {
        self.cancel_text = text;
        self
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Show confirmation dialog, returns ConfirmResult
    pub fn show(self, ui: &mut Ui, open: bool) -> ConfirmResult {
        if !open {
            return ConfirmResult::None;
        }

        use crate::atoms::Button;

        let mut result = ConfirmResult::None;

        let close_requested = Modal::titled(self.title).show(ui, open, |ui| {
            ui.label(self.message);
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if self.danger {
                    if Button::danger(self.confirm_text).show(ui) {
                        result = ConfirmResult::Confirmed;
                    }
                } else if Button::primary(self.confirm_text).show(ui) {
                    result = ConfirmResult::Confirmed;
                }

                if Button::outline(self.cancel_text).show(ui) {
                    result = ConfirmResult::Cancelled;
                }
            });
        });

        if close_requested && result == ConfirmResult::None {
            result = ConfirmResult::Cancelled;
        }

        result
    }
}
