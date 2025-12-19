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
pub mod router;
mod runtime;
pub mod testing;
mod view_ctx;

pub use app::App;
pub use cmd::Cmd;
pub use component::Component;
pub use router::{Router, RouterMsg};
pub use runtime::{run, RunConfig};
pub use view_ctx::ViewCtx;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::router::{BackButton, NavLink, Router, RouterMsg};
    pub use crate::{App, Cmd, Component, RunConfig, ViewCtx};
    pub use egui;
}

/// Testing utilities prelude
pub mod test_prelude {
    pub use crate::testing::{CmdRecord, ModelAssert, TestRunner};
    pub use crate::{App, Cmd};
}
