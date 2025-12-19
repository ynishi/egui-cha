//! Context Menu extension for egui Response

use egui::Response;
use egui_cha::ViewCtx;

use crate::Theme;

/// Context menu item
pub enum ContextMenuItem<Msg> {
    /// Regular menu item with label and message
    Item {
        label: String,
        msg: Msg,
        danger: bool,
    },
    /// Separator line
    Separator,
}

impl<Msg> ContextMenuItem<Msg> {
    /// Create a new menu item
    pub fn new(label: impl Into<String>, msg: Msg) -> Self {
        Self::Item {
            label: label.into(),
            msg,
            danger: false,
        }
    }

    /// Create a danger-styled menu item (e.g., for delete actions)
    pub fn danger(label: impl Into<String>, msg: Msg) -> Self {
        Self::Item {
            label: label.into(),
            msg,
            danger: true,
        }
    }

    /// Create a separator
    pub fn separator() -> Self {
        Self::Separator
    }
}

/// Extension trait for adding context menu to Response
pub trait ContextMenuExt {
    /// Add a context menu (right-click menu) to this response
    ///
    /// # Example
    /// ```ignore
    /// Button::primary("Item")
    ///     .show(ctx.ui)
    ///     .with_context_menu(ctx, [
    ///         ContextMenuItem::new("Edit", Msg::Edit),
    ///         ContextMenuItem::separator(),
    ///         ContextMenuItem::danger("Delete", Msg::Delete),
    ///     ]);
    /// ```
    fn with_context_menu<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        items: impl IntoIterator<Item = ContextMenuItem<Msg>>,
    ) -> Self
    where
        Msg: Clone;
}

impl ContextMenuExt for Response {
    fn with_context_menu<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        items: impl IntoIterator<Item = ContextMenuItem<Msg>>,
    ) -> Self
    where
        Msg: Clone,
    {
        let items: Vec<_> = items.into_iter().collect();

        self.context_menu(|ui| {
            let theme = Theme::current(ui.ctx());

            for item in items {
                match item {
                    ContextMenuItem::Item { label, msg, danger } => {
                        let text_color = if danger {
                            theme.danger
                        } else {
                            theme.text_primary
                        };

                        let text = egui::RichText::new(&label).color(text_color);
                        if ui.button(text).clicked() {
                            ctx.emit(msg);
                            ui.close_menu();
                        }
                    }
                    ContextMenuItem::Separator => {
                        ui.separator();
                    }
                }
            }
        });

        self
    }
}
