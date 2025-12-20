//! Menu molecule - Vertical tabs / navigation menu
//!
//! A vertical menu component similar to Tabs but arranged vertically.
//! Useful for sidebar navigation, settings menus, etc.

use crate::atoms::ListItem;
use egui::Ui;
use egui_cha::ViewCtx;

/// Vertical menu component (like Tabs but vertical)
pub struct Menu<'a> {
    items: &'a [&'a str],
}

impl<'a> Menu<'a> {
    /// Create a new menu with items
    pub fn new(items: &'a [&'a str]) -> Self {
        Self { items }
    }

    /// TEA-style: Show menu with current index, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        active: usize,
        on_change: impl Fn(usize) -> Msg,
    ) {
        if let Some(new_idx) = self.render(ctx.ui, active) {
            ctx.emit(on_change(new_idx));
        }
    }

    /// Show menu (modifies index in place)
    pub fn show(self, ui: &mut Ui, active: &mut usize) {
        if let Some(new_idx) = self.render(ui, *active) {
            *active = new_idx;
        }
    }

    /// Render menu and return clicked index if any
    fn render(self, ui: &mut Ui, active: usize) -> Option<usize> {
        let mut clicked_idx: Option<usize> = None;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0; // No gap between items
            for (i, item) in self.items.iter().enumerate() {
                let is_active = i == active;

                let response = ListItem::new(*item)
                    .selected(is_active)
                    .show(ui);

                if response.clicked() && !is_active {
                    clicked_idx = Some(i);
                }
            }
        });

        clicked_idx
    }
}

/// Menu with icons
pub struct IconMenu<'a> {
    items: &'a [(&'a str, &'static str)], // (label, icon)
}

impl<'a> IconMenu<'a> {
    /// Create a new menu with (label, icon) pairs
    pub fn new(items: &'a [(&'a str, &'static str)]) -> Self {
        Self { items }
    }

    /// TEA-style: Show menu with current index, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        active: usize,
        on_change: impl Fn(usize) -> Msg,
    ) {
        if let Some(new_idx) = self.render(ctx.ui, active) {
            ctx.emit(on_change(new_idx));
        }
    }

    /// Show menu (modifies index in place)
    pub fn show(self, ui: &mut Ui, active: &mut usize) {
        if let Some(new_idx) = self.render(ui, *active) {
            *active = new_idx;
        }
    }

    /// Render menu and return clicked index if any
    fn render(self, ui: &mut Ui, active: usize) -> Option<usize> {
        let mut clicked_idx: Option<usize> = None;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0; // No gap between items
            for (i, (label, icon)) in self.items.iter().enumerate() {
                let is_active = i == active;

                let response = ListItem::new(*label)
                    .icon(*icon)
                    .selected(is_active)
                    .show(ui);

                if response.clicked() && !is_active {
                    clicked_idx = Some(i);
                }
            }
        });

        clicked_idx
    }
}
