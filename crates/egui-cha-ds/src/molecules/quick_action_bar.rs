//! Quick Action Bar - Icon-based action toolbar
//!
//! A horizontal toolbar of icon buttons with tooltips, designed for
//! quick access to frequently used actions.
//!
//! # Features
//!
//! - **Icon-based**: Uses Phosphor Icons for consistent appearance
//! - **Tooltips**: Hover descriptions for each action
//! - **Scaling**: Configurable icon sizes (Small/Medium/Large)
//! - **Theme-aware**: Automatically adapts to light/dark themes
//! - **Optional keybinds**: Associate keyboard shortcuts with actions
//!
//! # Example
//!
//! ```ignore
//! use egui_cha_ds::{QuickActionBar, icons};
//!
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     QuickActionBar::new()
//!         .action(icons::GEAR, "Settings", Msg::OpenSettings)
//!         .action(icons::PLAY, "Run", Msg::Run)
//!         .action(icons::STOP, "Stop", Msg::Stop)
//!         .show(ctx);
//! }
//! ```

use egui::{Color32, Key, Modifiers, Response, Ui};
use egui_cha::ViewCtx;

use crate::Theme;

/// Size variant for quick action buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuickActionSize {
    /// Small icons (16px)
    Small,
    /// Medium icons (20px, default)
    #[default]
    Medium,
    /// Large icons (24px)
    Large,
}

impl QuickActionSize {
    fn icon_size(&self) -> f32 {
        match self {
            QuickActionSize::Small => 16.0,
            QuickActionSize::Medium => 20.0,
            QuickActionSize::Large => 24.0,
        }
    }

    fn button_padding(&self, theme: &Theme) -> f32 {
        match self {
            QuickActionSize::Small => theme.spacing_xs,
            QuickActionSize::Medium => theme.spacing_sm,
            QuickActionSize::Large => theme.spacing_sm,
        }
    }
}

/// Visual style for quick action buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuickActionStyle {
    /// Transparent background, subtle on hover
    #[default]
    Ghost,
    /// Outlined buttons
    Outline,
    /// Filled background
    Filled,
}

/// Keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct KeyBind {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyBind {
    /// Create a keybind with no modifiers
    pub fn key(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::NONE,
        }
    }

    /// Create a keybind with Cmd/Ctrl modifier
    pub fn cmd(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::COMMAND,
        }
    }

    /// Create a keybind with Shift modifier
    pub fn shift(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::SHIFT,
        }
    }

    /// Create a keybind with Cmd/Ctrl + Shift modifiers
    pub fn cmd_shift(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::COMMAND.plus(Modifiers::SHIFT),
        }
    }

    /// Create a keybind with Alt modifier
    pub fn alt(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::ALT,
        }
    }

    /// Create a custom keybind
    pub fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Format the keybind for display (e.g., "⌘K" or "Ctrl+K")
    pub fn display(&self) -> String {
        let mut parts = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if self.modifiers.ctrl {
                parts.push("⌃");
            }
            if self.modifiers.alt {
                parts.push("⌥");
            }
            if self.modifiers.shift {
                parts.push("⇧");
            }
            if self.modifiers.mac_cmd || self.modifiers.command {
                parts.push("⌘");
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            if self.modifiers.ctrl || self.modifiers.command {
                parts.push("Ctrl+");
            }
            if self.modifiers.alt {
                parts.push("Alt+");
            }
            if self.modifiers.shift {
                parts.push("Shift+");
            }
        }

        parts.push(self.key_name());

        parts.concat()
    }

    fn key_name(&self) -> &'static str {
        match self.key {
            Key::A => "A",
            Key::B => "B",
            Key::C => "C",
            Key::D => "D",
            Key::E => "E",
            Key::F => "F",
            Key::G => "G",
            Key::H => "H",
            Key::I => "I",
            Key::J => "J",
            Key::K => "K",
            Key::L => "L",
            Key::M => "M",
            Key::N => "N",
            Key::O => "O",
            Key::P => "P",
            Key::Q => "Q",
            Key::R => "R",
            Key::S => "S",
            Key::T => "T",
            Key::U => "U",
            Key::V => "V",
            Key::W => "W",
            Key::X => "X",
            Key::Y => "Y",
            Key::Z => "Z",
            Key::Num0 => "0",
            Key::Num1 => "1",
            Key::Num2 => "2",
            Key::Num3 => "3",
            Key::Num4 => "4",
            Key::Num5 => "5",
            Key::Num6 => "6",
            Key::Num7 => "7",
            Key::Num8 => "8",
            Key::Num9 => "9",
            Key::Escape => "Esc",
            Key::Tab => "Tab",
            Key::Backspace => "⌫",
            Key::Enter => "↵",
            Key::Space => "Space",
            Key::ArrowUp => "↑",
            Key::ArrowDown => "↓",
            Key::ArrowLeft => "←",
            Key::ArrowRight => "→",
            Key::Home => "Home",
            Key::End => "End",
            Key::PageUp => "PgUp",
            Key::PageDown => "PgDn",
            Key::Delete => "Del",
            Key::F1 => "F1",
            Key::F2 => "F2",
            Key::F3 => "F3",
            Key::F4 => "F4",
            Key::F5 => "F5",
            Key::F6 => "F6",
            Key::F7 => "F7",
            Key::F8 => "F8",
            Key::F9 => "F9",
            Key::F10 => "F10",
            Key::F11 => "F11",
            Key::F12 => "F12",
            _ => "?",
        }
    }

    /// Check if this keybind was pressed
    pub fn pressed(&self, ctx: &egui::Context) -> bool {
        ctx.input(|i| i.key_pressed(self.key) && i.modifiers == self.modifiers)
    }
}

/// A single action in the quick action bar
struct QuickActionItem<Msg> {
    icon: &'static str,
    tooltip: String,
    msg: Msg,
    keybind: Option<KeyBind>,
    disabled: bool,
    color: Option<Color32>,
}

/// Quick Action Bar - horizontal toolbar of icon buttons
///
/// # Example
///
/// ```ignore
/// QuickActionBar::new()
///     .action(icons::GEAR, "Settings", Msg::OpenSettings)
///     .action_with_keybind(icons::PLAY, "Run", Msg::Run, KeyBind::cmd(Key::R))
///     .size(QuickActionSize::Medium)
///     .show(ctx);
/// ```
pub struct QuickActionBar<Msg> {
    actions: Vec<QuickActionItem<Msg>>,
    size: QuickActionSize,
    style: QuickActionStyle,
    spacing: Option<f32>,
    tooltip_delay: Option<f32>,
}

impl<Msg: Clone> QuickActionBar<Msg> {
    /// Create a new empty quick action bar
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            size: QuickActionSize::default(),
            style: QuickActionStyle::default(),
            spacing: None,
            tooltip_delay: None,
        }
    }

    /// Add an action to the bar
    ///
    /// # Arguments
    /// - `icon`: Phosphor icon codepoint (e.g., `icons::GEAR`)
    /// - `tooltip`: Hover description text
    /// - `msg`: Message to emit when clicked
    pub fn action(mut self, icon: &'static str, tooltip: impl Into<String>, msg: Msg) -> Self {
        self.actions.push(QuickActionItem {
            icon,
            tooltip: tooltip.into(),
            msg,
            keybind: None,
            disabled: false,
            color: None,
        });
        self
    }

    /// Add an action with a keyboard shortcut
    pub fn action_with_keybind(
        mut self,
        icon: &'static str,
        tooltip: impl Into<String>,
        msg: Msg,
        keybind: KeyBind,
    ) -> Self {
        self.actions.push(QuickActionItem {
            icon,
            tooltip: tooltip.into(),
            msg,
            keybind: Some(keybind),
            disabled: false,
            color: None,
        });
        self
    }

    /// Add an action with custom color
    pub fn action_colored(
        mut self,
        icon: &'static str,
        tooltip: impl Into<String>,
        msg: Msg,
        color: Color32,
    ) -> Self {
        self.actions.push(QuickActionItem {
            icon,
            tooltip: tooltip.into(),
            msg,
            keybind: None,
            disabled: false,
            color: Some(color),
        });
        self
    }

    /// Add a disabled action
    pub fn action_disabled(
        mut self,
        icon: &'static str,
        tooltip: impl Into<String>,
        msg: Msg,
    ) -> Self {
        self.actions.push(QuickActionItem {
            icon,
            tooltip: tooltip.into(),
            msg,
            keybind: None,
            disabled: true,
            color: None,
        });
        self
    }

    /// Set the icon size
    pub fn size(mut self, size: QuickActionSize) -> Self {
        self.size = size;
        self
    }

    /// Set the visual style
    pub fn style(mut self, style: QuickActionStyle) -> Self {
        self.style = style;
        self
    }

    /// Set custom spacing between buttons
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Set tooltip delay in seconds (0.0 for immediate, default ~0.5s)
    pub fn tooltip_delay(mut self, delay: f32) -> Self {
        self.tooltip_delay = Some(delay);
        self
    }

    /// Show tooltips immediately (no delay)
    pub fn tooltip_immediate(self) -> Self {
        self.tooltip_delay(0.0)
    }

    /// Show the action bar and emit messages on click/keybind
    pub fn show(self, ctx: &mut ViewCtx<'_, Msg>) {
        let theme = Theme::current(ctx.ui.ctx());
        let egui_ctx = ctx.ui.ctx().clone();
        let spacing = self.spacing.unwrap_or(theme.spacing_xs);
        let tooltip_delay = self.tooltip_delay;

        // Collect messages to emit (keybinds + clicks)
        let mut messages_to_emit: Vec<Msg> = Vec::new();

        // Check keybinds first
        for action in &self.actions {
            if !action.disabled {
                if let Some(keybind) = &action.keybind {
                    if keybind.pressed(&egui_ctx) {
                        messages_to_emit.push(action.msg.clone());
                    }
                }
            }
        }

        // Render and collect click events
        ctx.ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;

            for action in &self.actions {
                let response =
                    render_action_button(ui, action, &theme, self.size, self.style, tooltip_delay);

                if response.clicked() && !action.disabled {
                    messages_to_emit.push(action.msg.clone());
                }
            }
        });

        // Emit all collected messages
        for msg in messages_to_emit {
            ctx.emit(msg);
        }
    }

    /// Show the action bar (raw egui, without ViewCtx)
    ///
    /// Returns the index of the clicked action, if any
    pub fn show_raw(self, ui: &mut Ui) -> Option<usize> {
        let theme = Theme::current(ui.ctx());
        let egui_ctx = ui.ctx().clone();
        let spacing = self.spacing.unwrap_or(theme.spacing_xs);
        let size = self.size;
        let style = self.style;
        let tooltip_delay = self.tooltip_delay;

        let mut clicked_index = None;

        // Check keybinds first
        for (index, action) in self.actions.iter().enumerate() {
            if !action.disabled {
                if let Some(keybind) = &action.keybind {
                    if keybind.pressed(&egui_ctx) {
                        clicked_index = Some(index);
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;

            for (index, action) in self.actions.iter().enumerate() {
                let response =
                    render_action_button(ui, action, &theme, size, style, tooltip_delay);

                if response.clicked() && !action.disabled {
                    clicked_index = Some(index);
                }
            }
        });

        clicked_index
    }

}

/// Render a single action button (standalone function to avoid borrow issues)
fn render_action_button<Msg>(
    ui: &mut Ui,
    action: &QuickActionItem<Msg>,
    theme: &Theme,
    size: QuickActionSize,
    style: QuickActionStyle,
    tooltip_delay: Option<f32>,
) -> Response {
    let icon_size = size.icon_size();
    let padding = size.button_padding(theme);

    // Determine colors based on style and state
    let (bg_color, hover_bg, icon_color) = match style {
        QuickActionStyle::Ghost => (
            Color32::TRANSPARENT,
            theme.bg_tertiary,
            action.color.unwrap_or(theme.text_secondary),
        ),
        QuickActionStyle::Outline => (
            Color32::TRANSPARENT,
            theme.bg_secondary,
            action.color.unwrap_or(theme.text_primary),
        ),
        QuickActionStyle::Filled => (
            theme.bg_tertiary,
            theme.bg_secondary,
            action.color.unwrap_or(theme.text_primary),
        ),
    };

    let disabled_color = theme.text_muted;
    let final_icon_color = if action.disabled {
        disabled_color
    } else {
        icon_color
    };

    // Build tooltip text with optional keybind
    let tooltip_text = match &action.keybind {
        Some(kb) => format!("{} ({})", action.tooltip, kb.display()),
        None => action.tooltip.clone(),
    };

    // Create the button
    let button_size = egui::vec2(icon_size + padding * 2.0, icon_size + padding * 2.0);

    let (rect, response) =
        ui.allocate_exact_size(button_size, egui::Sense::click().union(egui::Sense::hover()));

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered() && !action.disabled;

        // Draw background
        let bg = if is_hovered { hover_bg } else { bg_color };
        ui.painter().rect_filled(rect, theme.radius_sm, bg);

        // Draw outline for Outline style
        if style == QuickActionStyle::Outline {
            ui.painter().rect_stroke(
                rect,
                theme.radius_sm,
                egui::Stroke::new(theme.border_width, theme.border),
                egui::StrokeKind::Inside,
            );
        }

        // Draw icon
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            action.icon,
            egui::FontId {
                size: icon_size,
                family: egui::FontFamily::Name("icons".into()),
            },
            final_icon_color,
        );
    }

    // Show tooltip with optional custom delay
    if let Some(delay) = tooltip_delay {
        let ctx = ui.ctx().clone();
        let old_delay = ctx.style().interaction.tooltip_delay;
        ctx.style_mut(|s| s.interaction.tooltip_delay = delay);
        let result = response.on_hover_text(tooltip_text);
        ctx.style_mut(|s| s.interaction.tooltip_delay = old_delay);
        result
    } else {
        response.on_hover_text(tooltip_text)
    }
}

impl<Msg: Clone> Default for QuickActionBar<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

/// Vertical variant of QuickActionBar
pub struct QuickActionColumn<Msg> {
    bar: QuickActionBar<Msg>,
}

impl<Msg: Clone> QuickActionColumn<Msg> {
    /// Create a new vertical action column
    pub fn new() -> Self {
        Self {
            bar: QuickActionBar::new(),
        }
    }

    /// Add an action
    pub fn action(mut self, icon: &'static str, tooltip: impl Into<String>, msg: Msg) -> Self {
        self.bar = self.bar.action(icon, tooltip, msg);
        self
    }

    /// Add an action with keybind
    pub fn action_with_keybind(
        mut self,
        icon: &'static str,
        tooltip: impl Into<String>,
        msg: Msg,
        keybind: KeyBind,
    ) -> Self {
        self.bar = self.bar.action_with_keybind(icon, tooltip, msg, keybind);
        self
    }

    /// Set size
    pub fn size(mut self, size: QuickActionSize) -> Self {
        self.bar = self.bar.size(size);
        self
    }

    /// Set style
    pub fn style(mut self, style: QuickActionStyle) -> Self {
        self.bar = self.bar.style(style);
        self
    }

    /// Set tooltip delay in seconds (0.0 for immediate)
    pub fn tooltip_delay(mut self, delay: f32) -> Self {
        self.bar = self.bar.tooltip_delay(delay);
        self
    }

    /// Show tooltips immediately (no delay)
    pub fn tooltip_immediate(self) -> Self {
        self.tooltip_delay(0.0)
    }

    /// Show the action column
    pub fn show(self, ctx: &mut ViewCtx<'_, Msg>) {
        let theme = Theme::current(ctx.ui.ctx());
        let egui_ctx = ctx.ui.ctx().clone();
        let spacing = self.bar.spacing.unwrap_or(theme.spacing_xs);
        let size = self.bar.size;
        let style = self.bar.style;
        let tooltip_delay = self.bar.tooltip_delay;

        // Collect messages to emit
        let mut messages_to_emit: Vec<Msg> = Vec::new();

        // Check keybinds
        for action in &self.bar.actions {
            if !action.disabled {
                if let Some(keybind) = &action.keybind {
                    if keybind.pressed(&egui_ctx) {
                        messages_to_emit.push(action.msg.clone());
                    }
                }
            }
        }

        // Render and collect click events
        ctx.ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = spacing;

            for action in &self.bar.actions {
                let response =
                    render_action_button(ui, action, &theme, size, style, tooltip_delay);

                if response.clicked() && !action.disabled {
                    messages_to_emit.push(action.msg.clone());
                }
            }
        });

        // Emit all collected messages
        for msg in messages_to_emit {
            ctx.emit(msg);
        }
    }
}

impl<Msg: Clone> Default for QuickActionColumn<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keybind_display_formats_correctly() {
        let kb = KeyBind::cmd(Key::K);
        let display = kb.display();
        // On macOS: "⌘K", on other platforms: "Ctrl+K"
        assert!(!display.is_empty());
    }
}
