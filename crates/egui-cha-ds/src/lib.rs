//! egui-cha-ds: Design System for egui-cha
//!
//! Provides themed UI components following Atomic Design principles:
//! - Atoms: Basic building blocks (Button, Input, Badge, etc.)
//! - Molecules: Combinations of atoms (SearchBar, FormField, etc.)
//! - Layout: Fluent Builder for composing layouts (col, row, grid)
//! - Theme: Consistent styling across all components

mod atoms;
mod layout;
mod molecules;
mod theme;

pub use atoms::*;
pub use molecules::*;
pub use theme::{Theme, ThemeVariant};

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
