//! Command Palette - Keyboard-driven command launcher
//!
//! A "Raycast-like" overlay for quick command access with fuzzy search,
//! keyboard navigation, and grouping support.
//!
//! # Features
//!
//! - **Keyboard Navigation**: Arrow keys, Enter to execute, Escape to close
//! - **Search**: Filter commands by typing (fuzzy matching optional via feature)
//! - **Grouping**: Organize commands with groups and separators
//! - **Shortcuts**: Display keyboard shortcuts for each command
//! - **Icons**: Phosphor icons support
//!
//! # Example
//!
//! ```ignore
//! // In your Model
//! struct Model {
//!     command_palette: CommandPaletteState,
//! }
//!
//! // In your view
//! if model.command_palette.is_open {
//!     CommandPalette::new()
//!         .placeholder("Type a command...")
//!         .item(icons::GEAR, "Settings", Msg::OpenSettings)
//!         .item_with_shortcut(icons::FLOPPY_DISK, "Save", Msg::Save, "⌘S")
//!         .separator()
//!         .group("File", |p| {
//!             p.item(icons::FILE, "New File", Msg::NewFile)
//!              .item(icons::FOLDER_SIMPLE, "Open Folder", Msg::OpenFolder)
//!         })
//!         .show(ctx, &mut model.command_palette, Msg::CommandPaletteClose);
//! }
//! ```

use egui::{Key, Ui};
use egui_cha::ViewCtx;

use crate::Theme;

/// State for CommandPalette (store in your Model)
#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    /// Whether the palette is open
    pub is_open: bool,
    /// Current search query
    pub query: String,
    /// Currently selected item index (flattened)
    pub selected_index: usize,
}

impl CommandPaletteState {
    /// Create a new closed state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the palette and reset state
    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Close the palette
    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Toggle open/close
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }
}

/// A single command item
#[derive(Clone)]
pub struct CommandItem<Msg> {
    icon: Option<&'static str>,
    label: String,
    description: Option<String>,
    shortcut: Option<String>,
    msg: Msg,
}

impl<Msg: Clone> CommandItem<Msg> {
    /// Create a new command item
    pub fn new(label: impl Into<String>, msg: Msg) -> Self {
        Self {
            icon: None,
            label: label.into(),
            description: None,
            shortcut: None,
            msg,
        }
    }

    /// Add an icon
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Add a description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a keyboard shortcut display
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// An entry in the command palette (item, separator, or group)
#[derive(Clone)]
pub enum CommandEntry<Msg> {
    /// A command item
    Item(CommandItem<Msg>),
    /// A visual separator
    Separator,
    /// A group of commands with a label
    Group {
        label: String,
        items: Vec<CommandEntry<Msg>>,
    },
}

/// Command Palette UI component
pub struct CommandPalette<Msg> {
    placeholder: String,
    entries: Vec<CommandEntry<Msg>>,
    width: f32,
    max_height: f32,
    show_icons: bool,
}

impl<Msg: Clone> CommandPalette<Msg> {
    /// Create a new command palette
    pub fn new() -> Self {
        Self {
            placeholder: "Type a command...".to_string(),
            entries: Vec::new(),
            width: 500.0,
            max_height: 400.0,
            show_icons: true,
        }
    }

    /// Set placeholder text for the search input
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the width of the palette
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the maximum height of the palette
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// Hide icons
    pub fn hide_icons(mut self) -> Self {
        self.show_icons = false;
        self
    }

    /// Add a simple command item
    pub fn item(mut self, icon: &'static str, label: impl Into<String>, msg: Msg) -> Self {
        self.entries
            .push(CommandEntry::Item(CommandItem::new(label, msg).icon(icon)));
        self
    }

    /// Add a command item without icon
    pub fn item_plain(mut self, label: impl Into<String>, msg: Msg) -> Self {
        self.entries
            .push(CommandEntry::Item(CommandItem::new(label, msg)));
        self
    }

    /// Add a command item with shortcut
    pub fn item_with_shortcut(
        mut self,
        icon: &'static str,
        label: impl Into<String>,
        msg: Msg,
        shortcut: impl Into<String>,
    ) -> Self {
        self.entries.push(CommandEntry::Item(
            CommandItem::new(label, msg).icon(icon).shortcut(shortcut),
        ));
        self
    }

    /// Add a command item with description
    pub fn item_with_description(
        mut self,
        icon: &'static str,
        label: impl Into<String>,
        msg: Msg,
        description: impl Into<String>,
    ) -> Self {
        self.entries.push(CommandEntry::Item(
            CommandItem::new(label, msg)
                .icon(icon)
                .description(description),
        ));
        self
    }

    /// Add a full command item
    pub fn item_full(mut self, item: CommandItem<Msg>) -> Self {
        self.entries.push(CommandEntry::Item(item));
        self
    }

    /// Add a separator
    pub fn separator(mut self) -> Self {
        self.entries.push(CommandEntry::Separator);
        self
    }

    /// Add a group of commands
    pub fn group(mut self, label: impl Into<String>, build: impl FnOnce(Self) -> Self) -> Self {
        let builder = Self::new();
        let built = build(builder);
        self.entries.push(CommandEntry::Group {
            label: label.into(),
            items: built.entries,
        });
        self
    }

    /// Show the command palette
    ///
    /// Returns the message to emit when a command is selected
    pub fn show(self, ctx: &mut ViewCtx<'_, Msg>, state: &mut CommandPaletteState, on_close: Msg) {
        if !state.is_open {
            return;
        }

        let theme = Theme::current(ctx.ui.ctx());

        // Handle keyboard navigation (before rendering, using previous frame's state)
        let mut should_close = false;
        let mut selected_msg: Option<Msg> = None;

        ctx.ui.input(|input| {
            if input.key_pressed(Key::Escape) {
                should_close = true;
            }
        });

        // Store query for later keyboard handling
        let query_before = state.query.clone();

        // Pre-calculate items to determine height
        let row_height = theme.spacing_lg + theme.spacing_sm;
        let header_height = 60.0; // input + separator area
        let flat_items_for_height = self.flatten_items(&query_before);
        let content_height = flat_items_for_height.len() as f32 * row_height + header_height;
        let actual_height = content_height.min(self.max_height);

        // Render overlay
        egui::Area::new(egui::Id::new("command_palette_area"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .order(egui::Order::Foreground)
            .show(ctx.ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(theme.bg_primary)
                    .stroke(egui::Stroke::new(theme.border_width, theme.border))
                    .rounding(theme.radius_md)
                    .shadow(egui::Shadow {
                        spread: 8,
                        blur: 16,
                        color: egui::Color32::from_black_alpha(60),
                        offset: [0, 4],
                    })
                    .show(ui, |ui| {
                        ui.set_width(self.width);
                        ui.set_min_height(actual_height);

                        // Search input
                        ui.add_space(theme.spacing_sm);
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut state.query)
                                .hint_text(&self.placeholder)
                                .desired_width(f32::INFINITY)
                                .frame(false)
                                .font(egui::TextStyle::Body)
                                .margin(egui::vec2(theme.spacing_sm, 0.0)),
                        );
                        // Auto-focus
                        response.request_focus();
                        ui.add_space(theme.spacing_xs);

                        ui.separator();

                        // Flatten entries AFTER TextEdit has updated query
                        let flat_items = self.flatten_items(&state.query);
                        let item_count = flat_items.len();

                        // Clamp selected_index to valid range
                        if item_count > 0 && state.selected_index >= item_count {
                            state.selected_index = item_count - 1;
                        }

                        // Reset selected_index when query changes
                        if state.query != query_before {
                            state.selected_index = 0;
                        }

                        // Handle arrow keys and enter
                        ui.input(|input| {
                            if input.key_pressed(Key::ArrowDown) && item_count > 0 {
                                state.selected_index = (state.selected_index + 1) % item_count;
                            }
                            if input.key_pressed(Key::ArrowUp) && item_count > 0 {
                                state.selected_index = state
                                    .selected_index
                                    .checked_sub(1)
                                    .unwrap_or(item_count - 1);
                            }
                            if input.key_pressed(Key::Enter) && !flat_items.is_empty() {
                                if let Some(item) = flat_items.get(state.selected_index) {
                                    selected_msg = Some(item.msg.clone());
                                    should_close = true;
                                }
                            }
                        });

                        // Results list - height is min of content and max_height
                        let row_height = theme.spacing_lg + theme.spacing_sm;
                        let content_height = flat_items.len() as f32 * row_height;
                        let scroll_max = content_height.min(self.max_height - 60.0);

                        egui::ScrollArea::vertical()
                            .id_salt("command_palette_scroll")
                            .max_height(scroll_max)
                            .show(ui, |ui| {
                                self.render_entries(
                                    ui,
                                    &self.entries,
                                    &state.query,
                                    state.selected_index,
                                    &mut 0,
                                    &flat_items,
                                    &theme,
                                    &mut selected_msg,
                                    &mut should_close,
                                );
                            });
                    });
            });

        // Handle close
        if should_close {
            state.close();
            ctx.emit(on_close);
        }

        // Emit selected command
        if let Some(msg) = selected_msg {
            ctx.emit(msg);
        }
    }

    /// Show without ViewCtx (returns selected index or None)
    pub fn show_raw(self, ui: &mut Ui, state: &mut CommandPaletteState) -> Option<usize> {
        if !state.is_open {
            return None;
        }

        let theme = Theme::current(ui.ctx());

        // Handle escape key
        let mut should_close = false;
        let mut selected_index: Option<usize> = None;

        ui.input(|input| {
            if input.key_pressed(Key::Escape) {
                should_close = true;
            }
        });

        // Store query for change detection
        let query_before = state.query.clone();

        // Pre-calculate items to determine height
        let row_height = theme.spacing_lg + theme.spacing_sm;
        let header_height = 60.0; // input + separator area
        let flat_items_for_height = self.flatten_items(&query_before);
        let content_height = flat_items_for_height.len() as f32 * row_height + header_height;
        let actual_height = content_height.min(self.max_height);

        // Render
        egui::Area::new(egui::Id::new("command_palette_area"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(theme.bg_primary)
                    .stroke(egui::Stroke::new(theme.border_width, theme.border))
                    .rounding(theme.radius_md)
                    .show(ui, |ui| {
                        ui.set_width(self.width);
                        ui.set_min_height(actual_height);

                        ui.add_space(theme.spacing_sm);
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut state.query)
                                .hint_text(&self.placeholder)
                                .desired_width(f32::INFINITY)
                                .frame(false)
                                .margin(egui::vec2(theme.spacing_sm, 0.0)),
                        );
                        response.request_focus();
                        ui.add_space(theme.spacing_xs);
                        ui.separator();

                        // Flatten entries AFTER TextEdit has updated query
                        let flat_items = self.flatten_items(&state.query);
                        let item_count = flat_items.len();

                        // Clamp selected_index to valid range
                        if item_count > 0 && state.selected_index >= item_count {
                            state.selected_index = item_count - 1;
                        }

                        // Reset selected_index when query changes
                        if state.query != query_before {
                            state.selected_index = 0;
                        }

                        // Handle arrow keys and enter
                        ui.input(|input| {
                            if input.key_pressed(Key::ArrowDown) && item_count > 0 {
                                state.selected_index = (state.selected_index + 1) % item_count;
                            }
                            if input.key_pressed(Key::ArrowUp) && item_count > 0 {
                                state.selected_index = state
                                    .selected_index
                                    .checked_sub(1)
                                    .unwrap_or(item_count - 1);
                            }
                            if input.key_pressed(Key::Enter) && !flat_items.is_empty() {
                                selected_index = Some(state.selected_index);
                                should_close = true;
                            }
                        });

                        // Results list - height is min of content and max_height
                        let row_height = theme.spacing_lg + theme.spacing_sm;
                        let content_height = flat_items.len() as f32 * row_height;
                        let scroll_max = content_height.min(self.max_height - 60.0);

                        let mut dummy_msg: Option<Msg> = None;
                        let mut dummy_close = false;

                        egui::ScrollArea::vertical()
                            .id_salt("command_palette_scroll_raw")
                            .max_height(scroll_max)
                            .show(ui, |ui| {
                                self.render_entries(
                                    ui,
                                    &self.entries,
                                    &state.query,
                                    state.selected_index,
                                    &mut 0,
                                    &flat_items,
                                    &theme,
                                    &mut dummy_msg,
                                    &mut dummy_close,
                                );
                            });

                        if dummy_close {
                            should_close = true;
                        }
                    });
            });

        if should_close {
            state.close();
        }

        selected_index
    }

    /// Flatten items for indexing (filtered by query)
    fn flatten_items(&self, query: &str) -> Vec<&CommandItem<Msg>> {
        let mut items = Vec::new();
        self.collect_items(&self.entries, query, &mut items);
        items
    }

    fn collect_items<'a>(
        &'a self,
        entries: &'a [CommandEntry<Msg>],
        query: &str,
        out: &mut Vec<&'a CommandItem<Msg>>,
    ) {
        let query_lower = query.to_lowercase();

        for entry in entries {
            match entry {
                CommandEntry::Item(item) => {
                    if query.is_empty() || self.matches_query(item, &query_lower) {
                        out.push(item);
                    }
                }
                CommandEntry::Group { items, .. } => {
                    self.collect_items(items, query, out);
                }
                CommandEntry::Separator => {}
            }
        }
    }

    fn matches_query(&self, item: &CommandItem<Msg>, query: &str) -> bool {
        let label_lower = item.label.to_lowercase();

        #[cfg(feature = "fuzzy")]
        {
            // Fuzzy matching with nucleo or similar
            fuzzy_match(&label_lower, query)
        }

        #[cfg(not(feature = "fuzzy"))]
        {
            // Simple substring match
            label_lower.contains(query)
                || item
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(query))
                    .unwrap_or(false)
        }
    }

    fn render_entries(
        &self,
        ui: &mut Ui,
        entries: &[CommandEntry<Msg>],
        query: &str,
        selected_index: usize,
        current_index: &mut usize,
        flat_items: &[&CommandItem<Msg>],
        theme: &Theme,
        selected_msg: &mut Option<Msg>,
        should_close: &mut bool,
    ) {
        let query_lower = query.to_lowercase();

        for entry in entries {
            match entry {
                CommandEntry::Item(item) => {
                    if !query.is_empty() && !self.matches_query(item, &query_lower) {
                        continue;
                    }

                    let is_selected = *current_index == selected_index;
                    let clicked = self.render_item(ui, item, is_selected, theme);

                    if clicked {
                        *selected_msg = Some(item.msg.clone());
                        *should_close = true;
                    }

                    *current_index += 1;
                }
                CommandEntry::Separator => {
                    // Hide separator when searching
                    if query.is_empty() {
                        ui.add_space(theme.spacing_xs);
                        ui.separator();
                        ui.add_space(theme.spacing_xs);
                    }
                }
                CommandEntry::Group { label, items } => {
                    // Check if any items in group match
                    let has_matches = query.is_empty()
                        || items.iter().any(|e| {
                            if let CommandEntry::Item(item) = e {
                                self.matches_query(item, &query_lower)
                            } else {
                                false
                            }
                        });

                    if has_matches {
                        // Group label
                        ui.add_space(theme.spacing_sm);
                        ui.label(
                            egui::RichText::new(label.to_uppercase())
                                .size(theme.font_size_xs)
                                .color(theme.text_muted),
                        );
                        ui.add_space(theme.spacing_xs);

                        self.render_entries(
                            ui,
                            items,
                            query,
                            selected_index,
                            current_index,
                            flat_items,
                            theme,
                            selected_msg,
                            should_close,
                        );
                    }
                }
            }
        }
    }

    fn render_item(
        &self,
        ui: &mut Ui,
        item: &CommandItem<Msg>,
        is_selected: bool,
        theme: &Theme,
    ) -> bool {
        // Colors based on state
        let (text_color, icon_color) = if is_selected {
            (theme.text_primary, theme.text_primary)
        } else {
            (theme.text_secondary, theme.text_muted)
        };

        let hover_color = theme.bg_tertiary;
        let selected_color = theme.bg_secondary;

        // Fixed row height
        let row_height = theme.spacing_lg + theme.spacing_sm;
        let available_width = ui.available_width();
        let padding = theme.spacing_sm;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_width, row_height),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            let bg = if is_selected {
                Some(selected_color)
            } else if response.hovered() {
                Some(hover_color)
            } else {
                None
            };

            if let Some(color) = bg {
                painter.rect_filled(rect, theme.radius_sm, color);
            }

            // Content layout
            let mut x = rect.min.x + padding;
            let center_y = rect.center().y;

            // Icon
            if self.show_icons {
                if let Some(icon) = item.icon {
                    let galley = painter.layout_no_wrap(
                        icon.to_string(),
                        egui::FontId::new(
                            theme.font_size_md,
                            egui::FontFamily::Name("icons".into()),
                        ),
                        icon_color,
                    );
                    let icon_pos = egui::pos2(x, center_y - galley.size().y / 2.0);
                    painter.galley(icon_pos, galley, icon_color);
                    x += theme.font_size_md + theme.spacing_sm;
                }
            }

            // Label
            let label_galley = painter.layout_no_wrap(
                item.label.clone(),
                egui::FontId::proportional(theme.font_size_sm),
                text_color,
            );
            let label_pos = egui::pos2(x, center_y - label_galley.size().y / 2.0);
            let label_width = label_galley.size().x;
            painter.galley(label_pos, label_galley, text_color);

            // Description (after label, smaller and muted)
            if let Some(desc) = &item.description {
                let desc_x = x + label_width + theme.spacing_sm;
                let desc_galley = painter.layout_no_wrap(
                    desc.clone(),
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_muted,
                );
                let desc_pos = egui::pos2(desc_x, center_y - desc_galley.size().y / 2.0);
                painter.galley(desc_pos, desc_galley, theme.text_muted);
            }

            // Shortcut (right-aligned)
            if let Some(shortcut) = &item.shortcut {
                let galley = painter.layout_no_wrap(
                    shortcut.clone(),
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_muted,
                );
                let shortcut_x = rect.max.x - padding - galley.size().x;
                let shortcut_pos = egui::pos2(shortcut_x, center_y - galley.size().y / 2.0);
                painter.galley(shortcut_pos, galley, theme.text_muted);
            }
        }

        // Cursor
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // Scroll into view if selected
        if is_selected {
            ui.scroll_to_rect(rect, Some(egui::Align::Center));
        }

        response.clicked()
    }
}

impl<Msg: Clone> Default for CommandPalette<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "fuzzy")]
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    // Simple fuzzy: all chars in pattern appear in order in text
    let mut pattern_chars = pattern.chars().peekable();
    for c in text.chars() {
        if pattern_chars.peek() == Some(&c) {
            pattern_chars.next();
        }
    }
    pattern_chars.peek().is_none()
}
