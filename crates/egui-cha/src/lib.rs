//! egui-cha: TEA (The Elm Architecture) framework for egui
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────┐
//! │  Design System (egui-cha-ds)           │
//! │  Button, Input, Card, Modal...         │
//! ├────────────────────────────────────────┤
//! │  Component Layer                       │
//! │  Props / Emit / Hierarchical           │
//! ├────────────────────────────────────────┤
//! │  TEA Core                              │
//! │  Model, Msg, update(), view(), Cmd     │
//! └────────────────────────────────────────┘
//!              ↓
//!           egui
//! ```

mod app;
mod cmd;
mod component;
pub mod drag_drop;
pub mod helpers;
pub mod router;
pub mod sub;
pub mod testing;
mod view_ctx;

#[cfg(feature = "eframe")]
mod runtime;

pub use app::App;
pub use cmd::Cmd;
pub use component::Component;
pub use router::{Router, RouterMsg};
pub use sub::Sub;
pub use view_ctx::ViewCtx;

#[cfg(feature = "eframe")]
pub use runtime::{run, RunConfig};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::drag_drop::{DragSourceResponse, DropZoneResponse};
    pub use crate::helpers::{Debouncer, Throttler, TrailingThrottler};
    pub use crate::router::{BackButton, NavLink, Router, RouterMsg};
    pub use crate::sub::Sub;
    pub use crate::{App, Cmd, Component, ViewCtx};
    pub use egui;

    #[cfg(feature = "eframe")]
    pub use crate::RunConfig;
}

/// Testing utilities prelude
pub mod test_prelude {
    pub use crate::testing::{CmdRecord, ModelAssert, TestRunner};
    pub use crate::{App, Cmd};
}
