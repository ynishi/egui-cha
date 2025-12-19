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

pub use button::{
    // Display style
    ButtonStyle,
    // File operations
    save, close, delete, edit,
    // Common actions
    add, remove, search, refresh, settings,
    // Media
    play, pause, stop,
    // Clipboard
    copy,
    // Navigation
    back, forward,
    // Confirmation
    confirm, cancel,
};
