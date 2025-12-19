//! Toast notifications
//!
//! Temporary notifications that appear and auto-dismiss.
//!
//! # Usage
//!
//! ```ignore
//! use egui_cha_ds::ToastContainer;
//! use std::time::Duration;
//!
//! struct Model {
//!     toasts: ToastContainer,
//! }
//!
//! enum Msg {
//!     SaveClicked,
//!     DismissToast(ToastId),
//! }
//!
//! fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
//!     match msg {
//!         Msg::SaveClicked => {
//!             // Add toast and get auto-dismiss command
//!             model.toasts.success("Saved!", Duration::from_secs(3), Msg::DismissToast)
//!         }
//!         Msg::DismissToast(id) => {
//!             model.toasts.dismiss(id);
//!             Cmd::none()
//!         }
//!     }
//! }
//!
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     // ... main UI
//!
//!     // Show toasts (typically at the end of view)
//!     model.toasts.show(ctx, Msg::DismissToast);
//! }
//! ```

use egui::{Align2, Area, Color32, CornerRadius, Frame, Id, Order, RichText, Vec2};
use egui_cha::{Cmd, ViewCtx};
use std::time::Duration;

use crate::{icons, Theme, ThemeVariant};

/// Unique identifier for a toast
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastId(u64);

impl ToastId {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Toast variant determines the visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastVariant {
    /// Get colors from theme (subtle style: light bg, dark text)
    fn colors(&self, theme: &Theme) -> (Color32, Color32, &'static str) {
        let is_dark = theme.variant == ThemeVariant::Dark;

        match self {
            ToastVariant::Info => {
                let (bg, fg) = if is_dark {
                    (darken(theme.info, 0.4), lighten(theme.info, 0.6))
                } else {
                    (lighten(theme.info, 0.85), darken(theme.info, 0.3))
                };
                (bg, fg, icons::INFO)
            }
            ToastVariant::Success => {
                let (bg, fg) = if is_dark {
                    (darken(theme.success, 0.4), lighten(theme.success, 0.6))
                } else {
                    (lighten(theme.success, 0.85), darken(theme.success, 0.3))
                };
                (bg, fg, icons::CHECK)
            }
            ToastVariant::Warning => {
                let (bg, fg) = if is_dark {
                    (darken(theme.warning, 0.4), lighten(theme.warning, 0.6))
                } else {
                    (lighten(theme.warning, 0.85), darken(theme.warning, 0.4))
                };
                (bg, fg, icons::WARNING)
            }
            ToastVariant::Error => {
                let (bg, fg) = if is_dark {
                    (darken(theme.error, 0.4), lighten(theme.error, 0.6))
                } else {
                    (lighten(theme.error, 0.85), darken(theme.error, 0.3))
                };
                (bg, fg, icons::X)
            }
        }
    }
}

/// Position for toast container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    #[default]
    TopRight,
    BottomRight,
    TopLeft,
    BottomLeft,
}

impl ToastPosition {
    fn anchor(&self) -> Align2 {
        match self {
            ToastPosition::TopRight => Align2::RIGHT_TOP,
            ToastPosition::BottomRight => Align2::RIGHT_BOTTOM,
            ToastPosition::TopLeft => Align2::LEFT_TOP,
            ToastPosition::BottomLeft => Align2::LEFT_BOTTOM,
        }
    }

    fn offset(&self, theme: &Theme) -> Vec2 {
        let margin = theme.spacing_md;
        match self {
            ToastPosition::TopRight => Vec2::new(-margin, margin),
            ToastPosition::BottomRight => Vec2::new(-margin, -margin),
            ToastPosition::TopLeft => Vec2::new(margin, margin),
            ToastPosition::BottomLeft => Vec2::new(margin, -margin),
        }
    }

    fn is_bottom(&self) -> bool {
        matches!(self, ToastPosition::BottomRight | ToastPosition::BottomLeft)
    }
}

/// A single toast notification
#[derive(Debug, Clone)]
struct Toast {
    id: ToastId,
    message: String,
    variant: ToastVariant,
}

/// Container for managing toast notifications
#[derive(Debug, Clone, Default)]
pub struct ToastContainer {
    toasts: Vec<Toast>,
    position: ToastPosition,
}

impl ToastContainer {
    /// Create a new toast container
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the position for toasts
    pub fn with_position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Add an info toast
    pub fn info<Msg, F>(&mut self, message: impl Into<String>, duration: Duration, dismiss_msg: F) -> Cmd<Msg>
    where
        Msg: 'static + Send + Clone,
        F: Fn(ToastId) -> Msg + 'static + Send,
    {
        self.push(message, ToastVariant::Info, duration, dismiss_msg)
    }

    /// Add a success toast
    pub fn success<Msg, F>(&mut self, message: impl Into<String>, duration: Duration, dismiss_msg: F) -> Cmd<Msg>
    where
        Msg: 'static + Send + Clone,
        F: Fn(ToastId) -> Msg + 'static + Send,
    {
        self.push(message, ToastVariant::Success, duration, dismiss_msg)
    }

    /// Add a warning toast
    pub fn warning<Msg, F>(&mut self, message: impl Into<String>, duration: Duration, dismiss_msg: F) -> Cmd<Msg>
    where
        Msg: 'static + Send + Clone,
        F: Fn(ToastId) -> Msg + 'static + Send,
    {
        self.push(message, ToastVariant::Warning, duration, dismiss_msg)
    }

    /// Add an error toast
    pub fn error<Msg, F>(&mut self, message: impl Into<String>, duration: Duration, dismiss_msg: F) -> Cmd<Msg>
    where
        Msg: 'static + Send + Clone,
        F: Fn(ToastId) -> Msg + 'static + Send,
    {
        self.push(message, ToastVariant::Error, duration, dismiss_msg)
    }

    /// Add a toast with custom variant
    fn push<Msg, F>(
        &mut self,
        message: impl Into<String>,
        variant: ToastVariant,
        duration: Duration,
        dismiss_msg: F,
    ) -> Cmd<Msg>
    where
        Msg: 'static + Send + Clone,
        F: Fn(ToastId) -> Msg + 'static + Send,
    {
        let id = ToastId::new();
        self.toasts.push(Toast {
            id,
            message: message.into(),
            variant,
        });

        // Return command to auto-dismiss after duration
        Cmd::delay(duration, dismiss_msg(id))
    }

    /// Dismiss a toast by ID
    pub fn dismiss(&mut self, id: ToastId) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Check if there are any toasts
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Get the number of toasts
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Show all toasts
    pub fn show<Msg, F>(&self, ctx: &mut ViewCtx<'_, Msg>, _dismiss_msg: F)
    where
        F: Fn(ToastId) -> Msg + Clone,
    {
        if self.toasts.is_empty() {
            return;
        }

        let theme = Theme::current(ctx.ui.ctx());
        let screen_rect = ctx.ui.ctx().screen_rect();

        // Calculate starting position
        let anchor = self.position.anchor();
        let offset = self.position.offset(&theme);
        let base_pos = anchor.pos_in_rect(&screen_rect) + offset;

        // Render each toast
        let toast_height = 48.0;
        let toast_spacing = theme.spacing_sm;

        for (i, toast) in self.toasts.iter().enumerate() {
            let y_offset = if self.position.is_bottom() {
                -((i as f32) * (toast_height + toast_spacing))
            } else {
                (i as f32) * (toast_height + toast_spacing)
            };

            let pos = base_pos + Vec2::new(0.0, y_offset);

            Area::new(Id::new("toast").with(toast.id.0))
                .anchor(anchor, pos - anchor.pos_in_rect(&screen_rect))
                .order(Order::Foreground)
                .show(ctx.ui.ctx(), |ui| {
                    let (bg, fg, icon) = toast.variant.colors(&theme);

                    Frame::new()
                        .fill(bg)
                        .corner_radius(CornerRadius::same(theme.radius_md as u8))
                        .inner_margin(theme.spacing_sm + 4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Icon
                                ui.label(
                                    RichText::new(icon)
                                        .family(egui::FontFamily::Name("icons".into()))
                                        .color(fg)
                                        .size(16.0),
                                );

                                ui.add_space(theme.spacing_sm);

                                // Message
                                ui.label(RichText::new(&toast.message).color(fg));

                                ui.add_space(theme.spacing_md);

                                // Close button
                                let _close_response = ui.add(
                                    egui::Button::new(
                                        RichText::new(icons::X)
                                            .family(egui::FontFamily::Name("icons".into()))
                                            .color(fg)
                                            .size(14.0),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE),
                                );

                                // Note: dismiss via auto-dismiss Cmd
                            });
                        });
                });
        }
    }
}

/// Lighten a color by mixing with white
fn lighten(color: Color32, amount: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        (r as f32 + (255.0 - r as f32) * amount) as u8,
        (g as f32 + (255.0 - g as f32) * amount) as u8,
        (b as f32 + (255.0 - b as f32) * amount) as u8,
        a,
    )
}

/// Darken a color by mixing with black
fn darken(color: Color32, amount: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        (r as f32 * (1.0 - amount)) as u8,
        (g as f32 * (1.0 - amount)) as u8,
        (b as f32 * (1.0 - amount)) as u8,
        a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_container() {
        let mut container = ToastContainer::new();
        assert!(container.is_empty());

        let _cmd: Cmd<()> = container.success("Test", Duration::from_secs(3), |_| ());
        assert_eq!(container.len(), 1);

        let id = container.toasts[0].id;
        container.dismiss(id);
        assert!(container.is_empty());
    }

    #[test]
    fn test_toast_id_uniqueness() {
        let id1 = ToastId::new();
        let id2 = ToastId::new();
        assert_ne!(id1, id2);
    }
}
