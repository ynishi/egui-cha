//! Layout primitives with Fluent Builder pattern
//!
//! Provides type-safe, IDE-friendly layout composition.
//! Supports seamless nesting of layouts.
//!
//! # Example
//!
//! ```ignore
//! use egui_cha_ds::cha;
//!
//! cha::col()
//!     .spacing(8.0)
//!     .add(|ui| { ui.label("Title"); })
//!     .add(cha::row()  // Direct nesting - no closure needed!
//!         .add(|ui| { ui.button("OK"); })
//!         .add(|ui| { ui.button("Cancel"); }))
//!     .show(ui);
//! ```

mod builders;
pub mod spacer;

pub use builders::{col, row, grid, Col, Row, Grid, LayoutItem};
pub use spacer::Spacer;
