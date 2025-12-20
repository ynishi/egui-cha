//! Node Graph module - themed node graph editor
//!
//! Wraps egui_snarl with theme integration for visual node-based editing.
//!
//! # Features
//! - Theme-aware node and wire styling
//! - Context menus for nodes and graph background
//! - Action system for custom menu items
//! - Multi-connection support
//! - Serde serialization support
//!
//! # Example
//! ```ignore
//! // Define your node type
//! #[derive(Clone)]
//! enum MyNode {
//!     Source { name: String },
//!     Effect { intensity: f32 },
//!     Output,
//! }
//!
//! // In your Model
//! struct Model {
//!     graph: NodeGraph<MyNode>,
//! }
//!
//! // Implement the viewer
//! impl SnarlViewer<MyNode> for MyViewer {
//!     fn title(&mut self, node: &MyNode) -> String {
//!         match node {
//!             MyNode::Source { name } => format!("Source: {}", name),
//!             MyNode::Effect { .. } => "Effect".into(),
//!             MyNode::Output => "Output".into(),
//!         }
//!     }
//!
//!     fn inputs(&mut self, node: &MyNode) -> usize {
//!         match node {
//!             MyNode::Source { .. } => 0,
//!             MyNode::Effect { .. } => 1,
//!             MyNode::Output => 1,
//!         }
//!     }
//!
//!     fn outputs(&mut self, node: &MyNode) -> usize {
//!         match node {
//!             MyNode::Source { .. } => 1,
//!             MyNode::Effect { .. } => 1,
//!             MyNode::Output => 0,
//!         }
//!     }
//!
//!     fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<MyNode>) -> PinInfo {
//!         PinInfo::circle().with_fill(Color32::RED)
//!     }
//!
//!     fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<MyNode>) -> PinInfo {
//!         PinInfo::circle().with_fill(Color32::GREEN)
//!     }
//! }
//!
//! // In view
//! NodeGraphArea::new(&mut model.graph)
//!     .show(ui, &mut MyViewer::default());
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Ui};

// Re-export core types
pub use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
pub use egui_snarl::ui::{PinInfo, SnarlViewer};

/// Events emitted by NodeGraphArea
#[derive(Debug, Clone)]
pub enum NodeGraphEvent<T> {
    /// Node was added
    NodeAdded(NodeId),
    /// Node was removed
    NodeRemoved(NodeId, T),
    /// Node was selected
    NodeSelected(NodeId),
    /// Connection was made
    Connected { from: OutPinId, to: InPinId },
    /// Connection was removed
    Disconnected { from: OutPinId, to: InPinId },
    /// Custom action triggered from context menu
    Action(String),
}

/// A menu action for context menus
#[derive(Clone)]
pub struct MenuAction {
    /// Unique identifier for the action
    pub id: String,
    /// Display label
    pub label: String,
    /// Optional icon (emoji or text)
    pub icon: Option<String>,
    /// Whether the action is enabled
    pub enabled: bool,
}

impl MenuAction {
    /// Create a new menu action
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            enabled: true,
        }
    }

    /// Add an icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Style configuration for node graph
#[derive(Clone)]
pub struct NodeGraphStyle {
    /// Wire width
    pub wire_width: Option<f32>,
    /// Pin fill color
    pub pin_fill: Option<Color32>,
    /// Pin size
    pub pin_size: Option<f32>,
}

impl Default for NodeGraphStyle {
    fn default() -> Self {
        Self {
            wire_width: None,
            pin_fill: None,
            pin_size: None,
        }
    }
}

impl NodeGraphStyle {
    /// Create a new style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set wire width
    pub fn wire_width(mut self, width: f32) -> Self {
        self.wire_width = Some(width);
        self
    }

    /// Set pin fill color
    pub fn pin_fill(mut self, color: Color32) -> Self {
        self.pin_fill = Some(color);
        self
    }

    /// Set pin size
    pub fn pin_size(mut self, size: f32) -> Self {
        self.pin_size = Some(size);
        self
    }

    /// Build egui_snarl style from theme
    fn build_snarl_style(&self, theme: &Theme) -> egui_snarl::ui::SnarlStyle {
        let mut style = egui_snarl::ui::SnarlStyle::new();

        // Wire styling
        style.wire_width = self.wire_width;

        // Pin styling
        style.pin_fill = Some(self.pin_fill.unwrap_or(theme.primary));
        style.pin_size = self.pin_size;

        style
    }
}

/// A themed node graph (wraps egui_snarl::Snarl)
pub struct NodeGraph<T> {
    inner: Snarl<T>,
    style: NodeGraphStyle,
}

impl<T> Default for NodeGraph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> NodeGraph<T> {
    /// Create a new empty node graph
    pub fn new() -> Self {
        Self {
            inner: Snarl::new(),
            style: NodeGraphStyle::default(),
        }
    }

    /// Create from an existing Snarl
    pub fn from_snarl(snarl: Snarl<T>) -> Self {
        Self {
            inner: snarl,
            style: NodeGraphStyle::default(),
        }
    }

    /// Set custom style
    pub fn with_style(mut self, style: NodeGraphStyle) -> Self {
        self.style = style;
        self
    }

    /// Get immutable access to the inner Snarl
    pub fn inner(&self) -> &Snarl<T> {
        &self.inner
    }

    /// Get mutable access to the inner Snarl
    pub fn inner_mut(&mut self) -> &mut Snarl<T> {
        &mut self.inner
    }

    /// Get the style
    pub fn style(&self) -> &NodeGraphStyle {
        &self.style
    }

    /// Get mutable access to style
    pub fn style_mut(&mut self) -> &mut NodeGraphStyle {
        &mut self.style
    }

    /// Insert a new node at position
    pub fn insert(&mut self, pos: Pos2, node: T) -> NodeId {
        self.inner.insert_node(pos, node)
    }

    /// Remove a node (panics if node doesn't exist)
    pub fn remove(&mut self, id: NodeId) -> T {
        self.inner.remove_node(id)
    }

    /// Try to remove a node, returns None if it doesn't exist
    pub fn try_remove(&mut self, id: NodeId) -> Option<T> {
        if self.inner.get_node(id).is_some() {
            Some(self.inner.remove_node(id))
        } else {
            None
        }
    }

    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.inner.get_node(id)
    }

    /// Get a mutable node by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.inner.get_node_mut(id)
    }

    /// Connect two pins
    pub fn connect(&mut self, from: OutPinId, to: InPinId) {
        self.inner.connect(from, to);
    }

    /// Disconnect two pins
    pub fn disconnect(&mut self, from: OutPinId, to: InPinId) {
        self.inner.disconnect(from, to);
    }

    /// Get all node IDs with their data
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &T)> + '_ {
        self.inner.node_ids()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.inner.node_ids().count()
    }
}

/// Themed node graph area widget
pub struct NodeGraphArea<'a, T> {
    graph: &'a mut NodeGraph<T>,
    /// Actions for graph context menu (right-click on empty space)
    graph_actions: Vec<MenuAction>,
    /// Actions for node context menu (right-click on node)
    node_actions: Vec<MenuAction>,
}

impl<'a, T> NodeGraphArea<'a, T> {
    /// Create a new node graph area
    pub fn new(graph: &'a mut NodeGraph<T>) -> Self {
        Self {
            graph,
            graph_actions: Vec::new(),
            node_actions: Vec::new(),
        }
    }

    /// Add an action to the graph context menu
    pub fn graph_action(mut self, action: MenuAction) -> Self {
        self.graph_actions.push(action);
        self
    }

    /// Add multiple actions to the graph context menu
    pub fn graph_actions(mut self, actions: impl IntoIterator<Item = MenuAction>) -> Self {
        self.graph_actions.extend(actions);
        self
    }

    /// Add an action to the node context menu
    pub fn node_action(mut self, action: MenuAction) -> Self {
        self.node_actions.push(action);
        self
    }

    /// Add multiple actions to the node context menu
    pub fn node_actions(mut self, actions: impl IntoIterator<Item = MenuAction>) -> Self {
        self.node_actions.extend(actions);
        self
    }

    /// Show the node graph with a viewer
    pub fn show<V>(self, ui: &mut Ui, viewer: &mut V)
    where
        V: SnarlViewer<T>,
    {
        let theme = Theme::current(ui.ctx());
        let snarl_style = self.graph.style.build_snarl_style(&theme);

        self.graph
            .inner
            .show(viewer, &snarl_style, egui::Id::new("node_graph"), ui);
    }

    /// Show the node graph with events (TEA-style)
    pub fn show_with<V, Msg>(
        self,
        ctx: &mut egui_cha::ViewCtx<'_, Msg>,
        viewer: &mut V,
        on_event: impl Fn(NodeGraphEvent<T>) -> Msg,
    ) where
        V: SnarlViewer<T>,
        T: Clone,
    {
        let theme = Theme::current(ctx.ui.ctx());
        let snarl_style = self.graph.style.build_snarl_style(&theme);

        // Wrap viewer to capture events
        let mut event_viewer = EventCapturingViewer {
            inner: viewer,
            events: Vec::new(),
            graph_actions: &self.graph_actions,
            node_actions: &self.node_actions,
        };

        self.graph.inner.show(
            &mut event_viewer,
            &snarl_style,
            egui::Id::new("node_graph"),
            ctx.ui,
        );

        // Emit captured events
        for event in event_viewer.events {
            ctx.emit(on_event(event));
        }
    }
}

/// Internal viewer wrapper that captures events
struct EventCapturingViewer<'a, V, T> {
    inner: &'a mut V,
    events: Vec<NodeGraphEvent<T>>,
    graph_actions: &'a [MenuAction],
    node_actions: &'a [MenuAction],
}

impl<'a, V, T> SnarlViewer<T> for EventCapturingViewer<'a, V, T>
where
    V: SnarlViewer<T>,
    T: Clone,
{
    fn title(&mut self, node: &T) -> String {
        self.inner.title(node)
    }

    fn inputs(&mut self, node: &T) -> usize {
        self.inner.inputs(node)
    }

    fn outputs(&mut self, node: &T) -> usize {
        self.inner.outputs(node)
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        self.inner.show_input(pin, ui, snarl)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        self.inner.show_output(pin, ui, snarl)
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>) {
        self.events.push(NodeGraphEvent::Connected {
            from: from.id,
            to: to.id,
        });
        self.inner.connect(from, to, snarl);
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>) {
        self.events.push(NodeGraphEvent::Disconnected {
            from: from.id,
            to: to.id,
        });
        self.inner.disconnect(from, to, snarl);
    }

    fn has_graph_menu(&mut self, pos: Pos2, snarl: &mut Snarl<T>) -> bool {
        !self.graph_actions.is_empty() || self.inner.has_graph_menu(pos, snarl)
    }

    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, snarl: &mut Snarl<T>) {
        // Show custom actions
        for action in self.graph_actions {
            let label = if let Some(icon) = &action.icon {
                format!("{} {}", icon, action.label)
            } else {
                action.label.clone()
            };

            let button = ui.add_enabled(action.enabled, egui::Button::new(&label));
            if button.clicked() {
                self.events.push(NodeGraphEvent::Action(action.id.clone()));
                ui.close();
            }
        }

        // Separator if we have both custom and inner actions
        if !self.graph_actions.is_empty() && self.inner.has_graph_menu(pos, snarl) {
            ui.separator();
        }

        // Show inner viewer's menu
        if self.inner.has_graph_menu(pos, snarl) {
            self.inner.show_graph_menu(pos, ui, snarl);
        }
    }

    fn has_node_menu(&mut self, node: &T) -> bool {
        !self.node_actions.is_empty() || self.inner.has_node_menu(node)
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) {
        // Show custom actions
        for action in self.node_actions {
            let label = if let Some(icon) = &action.icon {
                format!("{} {}", icon, action.label)
            } else {
                action.label.clone()
            };

            let button = ui.add_enabled(action.enabled, egui::Button::new(&label));
            if button.clicked() {
                self.events.push(NodeGraphEvent::Action(action.id.clone()));
                ui.close();
            }
        }

        // Separator
        if !self.node_actions.is_empty() {
            ui.separator();
        }

        // Delete node option
        if ui.button("Delete Node").clicked() {
            let removed = snarl.remove_node(node);
            self.events
                .push(NodeGraphEvent::NodeRemoved(node, removed));
            ui.close();
        }
    }
}

/// Prebuilt node types for common use cases
pub mod presets {
    use super::*;

    /// Common pin types with associated colors
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum PinType {
        /// Generic data flow
        Flow,
        /// Numeric value
        Number,
        /// Text string
        Text,
        /// Boolean
        Bool,
        /// Color
        Color,
        /// Texture/Image
        Texture,
        /// Audio signal
        Audio,
        /// Video signal
        Video,
        /// MIDI data
        Midi,
        /// Custom type
        Custom(u8),
    }

    impl PinType {
        /// Get the default color for this pin type
        pub fn default_color(&self) -> Color32 {
            match self {
                PinType::Flow => Color32::WHITE,
                PinType::Number => Color32::from_rgb(100, 200, 100),
                PinType::Text => Color32::from_rgb(200, 200, 100),
                PinType::Bool => Color32::from_rgb(200, 100, 100),
                PinType::Color => Color32::from_rgb(200, 100, 200),
                PinType::Texture => Color32::from_rgb(100, 150, 200),
                PinType::Audio => Color32::from_rgb(100, 200, 200),
                PinType::Video => Color32::from_rgb(200, 150, 100),
                PinType::Midi => Color32::from_rgb(150, 100, 200),
                PinType::Custom(_) => Color32::GRAY,
            }
        }
    }
}
