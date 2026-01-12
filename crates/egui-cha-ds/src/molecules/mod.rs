//! Molecules - Combinations of atoms
//!
//! Molecules combine multiple atoms into more complex components:
//! cards, menus, modals, forms, etc.
//!
//! # Creating New Molecules
//!
//! When creating a new molecule, follow these guidelines:
//!
//! ## 1. Use Atoms Where Possible
//!
//! Prefer composing existing atoms over custom painting:
//!
//! ```ignore
//! // ✅ Good: Reuse atoms
//! use crate::atoms::{Button, ListItem};
//!
//! // Inside molecule
//! ListItem::new(label).compact().show(ui);
//! Button::primary("Submit").show(ui);
//! ```
//!
//! ## 2. Respect Theme Scaling
//!
//! When atoms aren't sufficient, use theme values directly:
//!
//! ```ignore
//! let theme = Theme::current(ui.ctx());
//!
//! // Spacing between sections
//! ui.add_space(theme.spacing_md);
//!
//! // Custom frame
//! egui::Frame::new()
//!     .inner_margin(theme.spacing_sm)
//!     .corner_radius(theme.radius_md)
//!     .show(ui, |ui| { ... });
//! ```
//!
//! ## 3. Propagate Options to Atoms
//!
//! If your molecule has size/style options, pass them to child atoms:
//!
//! ```ignore
//! pub struct Menu { compact: bool }
//!
//! // In render:
//! let mut item = ListItem::new(label);
//! if self.compact {
//!     item = item.compact();
//! }
//! item.show(ui);
//! ```
//!
//! See [`atoms`](crate::atoms) module for the full theme token reference.

mod card;
mod dashboard_layout;
// #[cfg(feature = "dock")]
// mod dock;  // TODO: waiting for egui 0.31 compatible egui_dock
mod error_console;
mod form;
mod heatmap_grid;
mod log_stream;
mod menu;
mod modal;
mod navbar;
mod search_bar;
mod table;
mod tabs;
// #[cfg(feature = "tiles")]
// mod tiles;  // TODO: egui 0.31 compat
#[cfg(feature = "dock")]
mod dock;
pub mod layout_helpers;
mod node_layout;
#[cfg(feature = "snarl")]
mod snarl;
mod toast;
mod workspace;

pub use card::Card;
pub use dashboard_layout::{
    dashboard_3col, dashboard_full, DashboardEvent, DashboardLayout, DashboardState, SidebarConfig,
    TopBarConfig,
};
#[cfg(feature = "dock")]
pub use dock::{layout as dock_layout, DockArea, DockEvent, DockStyle, DockTree, TabInfo};
pub use error_console::{ErrorConsole, ErrorConsoleMsg, ErrorConsoleState, ErrorEntry, ErrorLevel};
pub use form::Form;
pub use heatmap_grid::{CellState, HeatmapCell, HeatmapGrid};
pub use log_stream::{LogEntry, LogFilter, LogStream, LogStreamState, TimestampFormat};
pub use menu::{IconMenu, Menu};
pub use modal::{ConfirmDialog, ConfirmResult, Modal};
pub use navbar::{navbar, sidebar, Navbar};
pub use search_bar::SearchBar;
#[cfg(feature = "extras")]
pub use table::DataColumnWidth;
pub use table::{DataTable, Table};
pub use tabs::{TabPanel, Tabs};
// #[cfg(feature = "tiles")]
// pub use tiles::{...};  // TODO: egui 0.31 compat
pub use node_layout::{
    ArrangeStrategy, LayoutPane, LockLevel, NodeLayout, NodeLayoutArea, NodeLayoutEvent,
};
#[cfg(feature = "snarl")]
pub use snarl::{
    presets as node_presets, InPin, InPinId, MenuAction, NodeGraph, NodeGraphArea, NodeGraphEvent,
    NodeGraphStyle, NodeId, OutPin, OutPinId, PinInfo, Snarl, SnarlViewer,
};
pub use toast::{ToastContainer, ToastId, ToastPosition, ToastVariant};
pub use workspace::{Edge, LayoutMode, SnapTarget, WorkspaceCanvas, WorkspaceEvent, WorkspacePane};
