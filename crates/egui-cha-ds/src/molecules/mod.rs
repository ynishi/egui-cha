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
// #[cfg(feature = "dock")]
// mod dock;  // TODO: waiting for egui 0.31 compatible egui_dock
mod error_console;
mod form;
mod menu;
mod modal;
mod navbar;
mod search_bar;
mod table;
mod tabs;
// #[cfg(feature = "tiles")]
// mod tiles;  // TODO: egui 0.31 compat
mod toast;
#[cfg(feature = "dock")]
mod dock;
#[cfg(feature = "snarl")]
mod snarl;
mod workspace;

pub use card::Card;
#[cfg(feature = "dock")]
pub use dock::{layout as dock_layout, DockArea, DockEvent, DockStyle, DockTree, TabInfo};
pub use error_console::{ErrorConsole, ErrorConsoleMsg, ErrorConsoleState, ErrorEntry, ErrorLevel};
pub use form::Form;
pub use menu::{IconMenu, Menu};
pub use modal::{ConfirmDialog, ConfirmResult, Modal};
pub use navbar::{navbar, sidebar, Navbar};
pub use search_bar::SearchBar;
pub use table::{DataTable, Table};
#[cfg(feature = "extras")]
pub use table::DataColumnWidth;
pub use tabs::{TabPanel, Tabs};
// #[cfg(feature = "tiles")]
// pub use tiles::{...};  // TODO: egui 0.31 compat
pub use toast::{ToastContainer, ToastId, ToastPosition, ToastVariant};
#[cfg(feature = "snarl")]
pub use snarl::{
    presets as node_presets, MenuAction, NodeGraph, NodeGraphArea, NodeGraphEvent,
    NodeGraphStyle, NodeId, InPin, InPinId, OutPin, OutPinId, PinInfo, Snarl, SnarlViewer,
};
pub use workspace::{
    Edge, LayoutMode, SnapTarget, WorkspaceCanvas, WorkspaceEvent, WorkspacePane,
};
