//! egui-cha-ds: Design System for egui-cha
//!
//! Provides themed UI components following Atomic Design principles:
//! - Atoms: Basic building blocks (Button, Input, Badge, etc.)
//! - Molecules: Combinations of atoms (SearchBar, FormField, etc.)
//! - Theme: Consistent styling across all components

mod atoms;
mod molecules;
mod theme;

pub use atoms::*;
pub use molecules::*;
pub use theme::{Theme, ThemeVariant};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::atoms::*;
    pub use crate::molecules::*;
    pub use crate::theme::{Theme, ThemeVariant};
}
