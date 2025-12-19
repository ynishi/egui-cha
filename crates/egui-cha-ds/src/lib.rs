//! egui-cha-ds: Design System for egui-cha
//!
//! Provides themed UI components following Atomic Design principles:
//! - Atoms: Basic building blocks (Button, Input, Badge, etc.)
//! - Molecules: Combinations of atoms (SearchBar, FormField, etc.)
//! - Theme: Consistent styling across all components
//!
//! # Usage with TEA (The Elm Architecture)
//!
//! Components are designed to work with egui-cha's ViewCtx:
//!
//! ```ignore
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     Button::primary("Click me").on_click(ctx, Msg::Clicked);
//!     Input::new()
//!         .placeholder("Enter text")
//!         .show_with(ctx, &model.text, Msg::TextChanged);
//! }
//! ```

mod atoms;
mod molecules;
pub mod semantics;
mod theme;

pub use atoms::*;
pub use molecules::*;
pub use theme::{Theme, ThemeVariant};

// Re-export macro when feature is enabled
#[cfg(feature = "macros")]
pub use egui_cha_macros::cha;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::atoms::*;
    pub use crate::molecules::*;
    pub use crate::semantics::{self, ButtonStyle};
    pub use crate::theme::{Theme, ThemeVariant};
}
