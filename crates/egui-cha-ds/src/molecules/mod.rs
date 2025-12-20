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
mod error_console;
mod form;
mod menu;
mod modal;
mod navbar;
mod search_bar;
mod table;
mod tabs;
mod toast;

pub use card::Card;
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
pub use toast::{ToastContainer, ToastId, ToastPosition, ToastVariant};
