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

/// Phosphor Icons font (embedded)
const PHOSPHOR_FONT: &[u8] = include_bytes!("../assets/fonts/Phosphor.ttf");

/// Set up icon fonts for egui-cha-ds components.
///
/// Call this during app initialization to register the Phosphor Icons font
/// as `FontFamily::Name("icons")`. This is required for icon components to work.
///
/// # Example
///
/// ```rust,ignore
/// eframe::run_native(
///     "My App",
///     options,
///     Box::new(|cc| {
///         egui_cha_ds::setup_fonts(&cc.egui_ctx);
///         Ok(Box::new(MyApp::default()))
///     }),
/// )
/// ```
///
/// Note: If using `egui_cha::run()`, fonts are set up automatically.
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "phosphor".to_owned(),
        egui::FontData::from_static(PHOSPHOR_FONT).into(),
    );

    fonts.families.insert(
        egui::FontFamily::Name("icons".into()),
        vec!["phosphor".to_owned()],
    );

    ctx.set_fonts(fonts);
}

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
