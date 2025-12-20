//! Dock module - themed docking system
//!
//! Wraps egui_dock with theme integration for flexible panel layouts with tabs.
//!
//! # Example
//! ```ignore
//! // Define your tab type
//! #[derive(Clone, PartialEq)]
//! enum Tab {
//!     Browser,
//!     Editor { id: usize },
//!     Console,
//! }
//!
//! // In your Model
//! struct Model {
//!     dock: DockTree<Tab>,
//! }
//!
//! // Initialize with layout helper
//! let dock = layout::left_right(Tab::Browser, Tab::Editor { id: 0 }, 0.25);
//!
//! // In view
//! DockArea::new(&mut model.dock)
//!     .show_close_buttons(true)
//!     .show(ui, |ui, tab| {
//!         match tab {
//!             Tab::Browser => ui.label("Browser"),
//!             Tab::Editor { id } => ui.label(format!("Editor {}", id)),
//!             Tab::Console => ui.label("Console"),
//!         }
//!     });
//! ```

use crate::Theme;
use egui::Ui;
use egui_cha::ViewCtx;

// Re-export core types for advanced usage
pub use egui_dock::{NodeIndex, SurfaceIndex};

/// Events emitted by DockArea
#[derive(Debug, Clone)]
pub enum DockEvent<Tab> {
    /// Tab was closed (returns the closed tab)
    TabClosed(Tab),
    /// Add button clicked on surface
    AddClicked { surface: SurfaceIndex, node: NodeIndex },
    /// Tab focus changed
    FocusChanged,
}

/// Optional trait for enhanced tab display
pub trait TabInfo {
    /// Tab title
    fn title(&self) -> String;

    /// Tab icon (optional, use emoji or egui icon)
    fn icon(&self) -> Option<&str> {
        None
    }

    /// Whether tab has unsaved changes
    fn is_dirty(&self) -> bool {
        false
    }

    /// Whether tab can be closed
    fn closeable(&self) -> bool {
        true
    }
}

/// A themed dock tree (wraps egui_dock::DockState)
pub struct DockTree<Tab> {
    inner: egui_dock::DockState<Tab>,
}

impl<Tab: Default> Default for DockTree<Tab> {
    fn default() -> Self {
        Self::new_single(Tab::default())
    }
}

impl<Tab> DockTree<Tab> {
    /// Create a new dock tree with a single tab
    pub fn new_single(tab: Tab) -> Self {
        Self {
            inner: egui_dock::DockState::new(vec![tab]),
        }
    }

    /// Create a dock tree with multiple tabs in a single tabbed container
    pub fn new_tabs(tabs: Vec<Tab>) -> Self {
        Self {
            inner: egui_dock::DockState::new(tabs),
        }
    }

    /// Create from an existing DockState
    pub fn from_state(state: egui_dock::DockState<Tab>) -> Self {
        Self { inner: state }
    }

    /// Get immutable access to the inner state
    pub fn inner(&self) -> &egui_dock::DockState<Tab> {
        &self.inner
    }

    /// Get mutable access to the inner state
    pub fn inner_mut(&mut self) -> &mut egui_dock::DockState<Tab> {
        &mut self.inner
    }

    /// Push a new tab to the focused leaf
    pub fn push(&mut self, tab: Tab) {
        self.inner.push_to_focused_leaf(tab);
    }

    /// Get the number of tabs across all surfaces
    pub fn tab_count(&self) -> usize {
        self.inner
            .iter_all_tabs()
            .count()
    }
}

impl<Tab: PartialEq> DockTree<Tab> {
    /// Find and remove a tab by reference
    pub fn close(&mut self, tab: &Tab) -> Option<Tab> {
        self.inner.find_tab(tab).map(|(surface, node, tab_idx)| {
            self.inner.remove_tab((surface, node, tab_idx)).unwrap()
        })
    }
}

/// Style configuration for dock area
#[derive(Clone)]
pub struct DockStyle {
    /// Show close buttons on tabs
    pub show_close_buttons: bool,
    /// Show add tab buttons
    pub show_add_buttons: bool,
    /// Allow tab reordering by dragging
    pub tabs_are_draggable: bool,
    /// Allow dragging tabs out of their containers
    pub allowed_splits: bool,
    /// Custom tab bar height (None = use theme)
    pub tab_bar_height: Option<f32>,
}

impl Default for DockStyle {
    fn default() -> Self {
        Self {
            show_close_buttons: true,
            show_add_buttons: false,
            tabs_are_draggable: true,
            allowed_splits: true,
            tab_bar_height: None,
        }
    }
}

/// Themed dock area wrapper
pub struct DockArea<'a, Tab> {
    tree: &'a mut DockTree<Tab>,
    style: DockStyle,
}

impl<'a, Tab> DockArea<'a, Tab> {
    /// Create a new dock area
    pub fn new(tree: &'a mut DockTree<Tab>) -> Self {
        Self {
            tree,
            style: DockStyle::default(),
        }
    }

    /// Show/hide close buttons on tabs
    pub fn show_close_buttons(mut self, show: bool) -> Self {
        self.style.show_close_buttons = show;
        self
    }

    /// Show/hide add tab buttons
    pub fn show_add_buttons(mut self, show: bool) -> Self {
        self.style.show_add_buttons = show;
        self
    }

    /// Enable/disable tab dragging
    pub fn tabs_are_draggable(mut self, draggable: bool) -> Self {
        self.style.tabs_are_draggable = draggable;
        self
    }

    /// Enable/disable splitting (dragging tabs to create new panes)
    pub fn allowed_splits(mut self, allowed: bool) -> Self {
        self.style.allowed_splits = allowed;
        self
    }

    /// Set custom tab bar height
    pub fn tab_bar_height(mut self, height: f32) -> Self {
        self.style.tab_bar_height = Some(height);
        self
    }

    /// Show the dock area with a simple tab renderer
    pub fn show<F>(self, ui: &mut Ui, mut tab_ui: F)
    where
        Tab: std::fmt::Debug,
        F: FnMut(&mut Ui, &mut Tab),
    {
        let theme = Theme::current(ui.ctx());
        let dock_style = build_dock_style(&theme, &self.style, ui.style());

        let mut viewer = SimpleTabViewer {
            tab_ui: &mut tab_ui,
            style: &self.style,
            _events: Vec::new(),
        };

        egui_dock::DockArea::new(&mut self.tree.inner)
            .style(dock_style)
            .show_close_buttons(self.style.show_close_buttons)
            .show_add_buttons(self.style.show_add_buttons)
            .draggable_tabs(self.style.tabs_are_draggable)
            .show_inside(ui, &mut viewer);
    }

    /// TEA-style: Show dock area and emit events via ViewCtx
    pub fn show_with<Msg, F>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        mut tab_ui: F,
        on_event: impl Fn(DockEvent<Tab>) -> Msg,
    ) where
        Tab: std::fmt::Debug + Clone,
        F: FnMut(&mut Ui, &mut Tab),
    {
        let theme = Theme::current(ctx.ui.ctx());
        let dock_style = build_dock_style(&theme, &self.style, ctx.ui.style());

        let mut events = Vec::new();
        let mut viewer = EventTabViewer {
            tab_ui: &mut tab_ui,
            style: &self.style,
            events: &mut events,
        };

        egui_dock::DockArea::new(&mut self.tree.inner)
            .style(dock_style)
            .show_close_buttons(self.style.show_close_buttons)
            .show_add_buttons(self.style.show_add_buttons)
            .draggable_tabs(self.style.tabs_are_draggable)
            .show_inside(ctx.ui, &mut viewer);

        // Emit collected events
        for event in events {
            ctx.emit(on_event(event));
        }
    }
}

/// Build egui_dock::Style from Theme
fn build_dock_style(theme: &Theme, style: &DockStyle, egui_style: &egui::Style) -> egui_dock::Style {
    let mut dock_style = egui_dock::Style::from_egui(egui_style);

    // Tab bar styling
    dock_style.tab_bar.height = style.tab_bar_height.unwrap_or(theme.spacing_lg + theme.spacing_sm);
    dock_style.tab_bar.fill_tab_bar = true;
    dock_style.tab_bar.bg_fill = theme.bg_primary;

    // Tab styling
    dock_style.tab.tab_body.bg_fill = theme.bg_secondary;
    dock_style.tab.active.bg_fill = theme.bg_secondary;
    dock_style.tab.inactive.bg_fill = theme.bg_primary;
    dock_style.tab.focused.bg_fill = theme.bg_secondary;
    dock_style.tab.hovered.bg_fill = theme.bg_tertiary;

    // Separator styling
    dock_style.separator.width = theme.spacing_xs;
    dock_style.separator.color_idle = theme.border;
    dock_style.separator.color_hovered = theme.primary;
    dock_style.separator.color_dragged = theme.primary;

    // Main surface border
    dock_style.main_surface_border_stroke = egui::Stroke::new(theme.border_width, theme.border);

    dock_style
}

/// Simple tab viewer for basic usage
struct SimpleTabViewer<'a, Tab, F>
where
    F: FnMut(&mut Ui, &mut Tab),
{
    tab_ui: &'a mut F,
    style: &'a DockStyle,
    _events: Vec<DockEvent<Tab>>,
}

impl<'a, Tab, F> egui_dock::TabViewer for SimpleTabViewer<'a, Tab, F>
where
    Tab: std::fmt::Debug,
    F: FnMut(&mut Ui, &mut Tab),
{
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{:?}", tab).into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        (self.tab_ui)(ui, tab);
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        self.style.show_close_buttons
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false // Disable floating windows for now
    }
}

/// Event-collecting tab viewer for TEA-style usage
struct EventTabViewer<'a, Tab, F>
where
    F: FnMut(&mut Ui, &mut Tab),
{
    tab_ui: &'a mut F,
    style: &'a DockStyle,
    events: &'a mut Vec<DockEvent<Tab>>,
}

impl<'a, Tab, F> egui_dock::TabViewer for EventTabViewer<'a, Tab, F>
where
    Tab: std::fmt::Debug + Clone,
    F: FnMut(&mut Ui, &mut Tab),
{
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{:?}", tab).into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        (self.tab_ui)(ui, tab);
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        self.style.show_close_buttons
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> egui_dock::widgets::tab_viewer::OnCloseResponse {
        self.events.push(DockEvent::TabClosed(tab.clone()));
        egui_dock::widgets::tab_viewer::OnCloseResponse::Close
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.events.push(DockEvent::AddClicked { surface, node });
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false // Disable floating windows for now
    }
}

/// Helper module for creating common layouts
pub mod layout {
    use super::*;

    /// Create a simple left-right split
    pub fn left_right<Tab>(left: Tab, right: Tab, left_fraction: f32) -> DockTree<Tab> {
        let mut state = egui_dock::DockState::new(vec![left]);
        let surface = state.main_surface_mut();

        // Split the root node horizontally
        let [_left_node, _right_node] = surface.split_right(
            NodeIndex::root(),
            1.0 - left_fraction,
            vec![right],
        );

        DockTree::from_state(state)
    }

    /// Create a simple top-bottom split
    pub fn top_bottom<Tab>(top: Tab, bottom: Tab, top_fraction: f32) -> DockTree<Tab> {
        let mut state = egui_dock::DockState::new(vec![top]);
        let surface = state.main_surface_mut();

        // Split the root node vertically
        let [_top_node, _bottom_node] = surface.split_below(
            NodeIndex::root(),
            1.0 - top_fraction,
            vec![bottom],
        );

        DockTree::from_state(state)
    }

    /// Create a three-column layout (left | center | right)
    pub fn three_column<Tab>(
        left: Tab,
        center: Tab,
        right: Tab,
        left_frac: f32,
        right_frac: f32,
    ) -> DockTree<Tab> {
        let mut state = egui_dock::DockState::new(vec![center]);
        let surface = state.main_surface_mut();

        // Add left panel
        let [_left_node, center_node] = surface.split_left(
            NodeIndex::root(),
            left_frac,
            vec![left],
        );

        // Add right panel
        let [_center_node, _right_node] = surface.split_right(
            center_node,
            right_frac / (1.0 - left_frac),
            vec![right],
        );

        DockTree::from_state(state)
    }

    /// DAW-style layout: Browser | Main | Inspector, Timeline at bottom
    ///
    /// ```text
    /// +----------+----------------+----------+
    /// | Browser  |     Main       | Inspector|
    /// |          |                |          |
    /// +----------+----------------+----------+
    /// |              Timeline                |
    /// +--------------------------------------+
    /// ```
    pub fn daw<Tab>(browser: Tab, main: Tab, inspector: Tab, timeline: Tab) -> DockTree<Tab> {
        let mut state = egui_dock::DockState::new(vec![main]);
        let surface = state.main_surface_mut();

        // Add browser on left
        let [_browser_node, center_node] = surface.split_left(
            NodeIndex::root(),
            0.2,
            vec![browser],
        );

        // Add inspector on right
        let [_center_node, _inspector_node] = surface.split_right(
            center_node,
            0.25,
            vec![inspector],
        );

        // Add timeline at bottom (spans full width by splitting the root)
        let [_top_node, _timeline_node] = surface.split_below(
            NodeIndex::root(),
            0.25,
            vec![timeline],
        );

        DockTree::from_state(state)
    }

    /// VSCode-style layout: Sidebar | Editor tabs over Terminal
    ///
    /// ```text
    /// +--------+-------------------------+
    /// |        |    Editor (tabbed)      |
    /// |Sidebar |-------------------------|
    /// |        |    Terminal (tabbed)    |
    /// +--------+-------------------------+
    /// ```
    pub fn vscode<Tab>(sidebar: Tab, editors: Vec<Tab>, terminals: Vec<Tab>) -> DockTree<Tab> {
        let editor_tabs = if editors.is_empty() {
            return DockTree::new_single(sidebar);
        } else {
            editors
        };

        let mut state = egui_dock::DockState::new(editor_tabs);
        let surface = state.main_surface_mut();

        // Add sidebar on left
        let [_sidebar_node, editor_node] = surface.split_left(
            NodeIndex::root(),
            0.2,
            vec![sidebar],
        );

        // Add terminals below editors
        if !terminals.is_empty() {
            let [_editor_node, _terminal_node] = surface.split_below(
                editor_node,
                0.3,
                terminals,
            );
        }

        DockTree::from_state(state)
    }
}

/// Re-export egui_dock for advanced usage
pub mod raw {
    #[allow(unused_imports)]
    pub use egui_dock::*;
}
