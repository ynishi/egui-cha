// Clippy/compiler warnings to address in v0.2.0
#![allow(dead_code)] // Some fields reserved for future use
#![allow(deprecated)] // allocate_ui_at_rect -> allocate_new_ui migration pending
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::let_and_return)]
#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::derivable_impls)]

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
    pub use crate::semantics::{self, ButtonStyle, LogSeverity, SeverityLog};
    pub use crate::theme::{Theme, ThemeVariant};
}
