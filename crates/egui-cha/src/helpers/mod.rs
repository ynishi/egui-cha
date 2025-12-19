//! Helper utilities for common TEA patterns
//!
//! This module provides utilities for handling common UI patterns
//! that are tricky to implement correctly in a pure TEA architecture.
//!
//! # Debouncing
//!
//! Use [`Debouncer`] when you want to delay an action until input stops.
//! Perfect for search-as-you-type, form validation, auto-save.
//!
//! ```ignore
//! use egui_cha::helpers::Debouncer;
//!
//! struct Model {
//!     search: String,
//!     debouncer: Debouncer,
//! }
//!
//! fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
//!     match msg {
//!         Msg::Input(s) => {
//!             model.search = s;
//!             model.debouncer.trigger(Duration::from_millis(300), Msg::Search)
//!         }
//!         Msg::Search => {
//!             if model.debouncer.should_fire() {
//!                 // Do search
//!             }
//!             Cmd::none()
//!         }
//!     }
//! }
//! ```
//!
//! # Throttling
//!
//! Use [`Throttler`] when you want to limit action frequency.
//! Perfect for scroll handlers, resize events, API rate limiting.
//!
//! ```ignore
//! use egui_cha::helpers::Throttler;
//!
//! struct Model {
//!     throttler: Throttler,
//! }
//!
//! fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
//!     match msg {
//!         Msg::Scroll(pos) => {
//!             model.throttler.run(Duration::from_millis(100), || {
//!                 Cmd::msg(Msg::UpdateView)
//!             })
//!         }
//!     }
//! }
//! ```
//!
//! # Comparison
//!
//! | Pattern | Behavior | Use Case |
//! |---------|----------|----------|
//! | Debounce | Waits until input stops | Search input, form validation |
//! | Throttle | Limits frequency | Scroll, resize, API calls |

mod clock;
mod debounce;
mod throttle;

pub use clock::{Clock, SystemClock};
pub use debounce::{Debouncer, DebouncerWithClock};
pub use throttle::{Throttler, ThrottlerWithClock, TrailingThrottler};
