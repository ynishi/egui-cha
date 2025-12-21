//! Semantics - Domain-specific UI components
//!
//! This module contains UI components with specific semantic meaning.
//! These are built on top of atoms but carry domain-specific intent.
//!
//! ## Design Philosophy
//!
//! - **Atoms**: Pure style primitives (colors, variants, layouts)
//! - **Semantics**: Meaningful actions with fixed labels (save, edit, delete)
//!
//! The key difference is that semantic components have **fixed labels and icons**,
//! ensuring UI consistency across the entire application. You can only choose
//! the display style (icon-only, text-only, or both).
//!
//! ## Example
//!
//! ```ignore
//! use egui_cha_ds::semantics::{self, ButtonStyle};
//!
//! // Icon only (compact)
//! semantics::save(ButtonStyle::Icon).on_click(ctx, Msg::Save);
//!
//! // Text only
//! semantics::close(ButtonStyle::Text).on_click(ctx, Msg::Close);
//!
//! // Icon + Text (most explicit)
//! semantics::delete(ButtonStyle::Both).on_click(ctx, Msg::Delete);
//! ```

mod button;
mod severity_log;

pub use button::{
    // Common actions
    add,
    // Navigation
    back,
    cancel,
    close,
    // Confirmation
    confirm,
    // Clipboard
    copy,
    delete,
    edit,
    forward,
    pause,
    // Media
    play,
    refresh,
    remove,
    // File operations
    save,
    search,
    settings,
    stop,
    // Display style
    ButtonStyle,
};

pub use severity_log::{LogSeverity, SeverityLog};
