//! egui-cha-ds: Design System for egui-cha
//!
//! Provides themed UI components following Atomic Design principles:
//! - Atoms: Basic building blocks (Button, Input, Badge, etc.)
//! - Molecules: Combinations of atoms (SearchBar, FormField, etc.)
//! - Layout: Fluent Builder for composing layouts (col, row, grid)
//! - Theme: Consistent styling across all components
//!
//! # Layout Macro (Sugar Layer)
//!
//! With the `macros` feature (enabled by default), you can use the `cha!` macro
//! for declarative layout composition:
//!
//! ```ignore
//! use egui_cha_ds::cha;
//!
//! cha! {
//!     Col(spacing: 8.0) {
//!         Row(fill_x) {
//!             [|ui: &mut egui::Ui| { ui.label("Title"); }]
//!             Spacer
//!         }
//!         Grid(3, gap: 4.0) {
//!             [|ui: &mut egui::Ui| { ui.label("A"); }]
//!             [|ui: &mut egui::Ui| { ui.label("B"); }]
//!             [|ui: &mut egui::Ui| { ui.label("C"); }]
//!         }
//!     }
//! }.show(ui);
//! ```

mod atoms;
mod layout;
mod molecules;
mod theme;

pub use atoms::*;
pub use molecules::*;
pub use theme::{Theme, ThemeVariant};

// Re-export macro when feature is enabled
#[cfg(feature = "macros")]
pub use egui_cha_macros::cha;

/// Layout builders namespace
///
/// Supports seamless nesting of layouts (Col, Row, Grid).
///
/// ```ignore
/// use egui_cha_ds::cha;
///
/// // Nested layouts - no closures needed for nesting!
/// cha::col()
///     .spacing(8.0)
///     .add(|ui| { ui.label("Title"); })
///     .add(cha::row()
///         .spacing(4.0)
///         .add(|ui| { ui.button("OK"); })
///         .add(|ui| { ui.button("Cancel"); }))
///     .add(cha::grid(3)
///         .gap(8.0)
///         .add(|ui| { ui.label("A"); })
///         .add(|ui| { ui.label("B"); })
///         .add(|ui| { ui.label("C"); }))
///     .show(ui);
/// ```
pub mod cha {
    pub use crate::layout::{col, row, grid, Col, Row, Grid, Spacer, LayoutItem};
    pub use crate::layout::spacer::{spacer, space};
}

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::atoms::*;
    pub use crate::molecules::*;
    pub use crate::theme::{Theme, ThemeVariant};
    pub use crate::cha;
}
