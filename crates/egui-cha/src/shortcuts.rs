//! Common keyboard shortcuts
//!
//! Provides standard keyboard shortcut constants for common actions.
//! These are cross-platform (Cmd on macOS, Ctrl on Windows/Linux).
//!
//! # Example
//! ```ignore
//! use egui_cha::shortcuts;
//!
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     ctx.on_shortcut(shortcuts::SAVE, Msg::Save);
//!     ctx.on_shortcut(shortcuts::UNDO, Msg::Undo);
//!     ctx.on_shortcut(shortcuts::COPY, Msg::Copy);
//! }
//! ```

use egui::{Key, KeyboardShortcut, Modifiers};

// File operations
pub const NEW: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
pub const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
pub const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
pub const SAVE_AS: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S);
pub const CLOSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::W);

// Edit operations
pub const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
pub const REDO: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);
pub const CUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::X);
pub const COPY: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::C);
pub const PASTE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::V);
pub const SELECT_ALL: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::A);
pub const DELETE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Delete);
pub const BACKSPACE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Backspace);

// Search
pub const FIND: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::F);
pub const FIND_NEXT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::G);
pub const FIND_PREV: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::G);
pub const REPLACE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::H);

// View
pub const ZOOM_IN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Plus);
pub const ZOOM_OUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Minus);
pub const ZOOM_RESET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Num0);
pub const FULLSCREEN: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::F);

// Navigation
pub const HOME: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Home);
pub const END: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::End);
pub const PAGE_UP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::PageUp);
pub const PAGE_DOWN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::PageDown);
pub const GO_TO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::G);

// Common actions
pub const ESCAPE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Escape);
pub const ENTER: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Enter);
pub const TAB: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Tab);
pub const SHIFT_TAB: KeyboardShortcut = KeyboardShortcut::new(Modifiers::SHIFT, Key::Tab);

// Application
pub const PREFERENCES: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Comma);
pub const HELP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F1);
pub const REFRESH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);

// Developer
pub const DEV_TOOLS: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::ALT), Key::I);
pub const CONSOLE: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::ALT), Key::J);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_are_different() {
        // Ensure we didn't accidentally define the same shortcut twice
        let shortcuts = [
            ("SAVE", SAVE),
            ("UNDO", UNDO),
            ("REDO", REDO),
            ("COPY", COPY),
            ("PASTE", PASTE),
            ("CUT", CUT),
        ];

        for i in 0..shortcuts.len() {
            for j in (i + 1)..shortcuts.len() {
                assert_ne!(
                    shortcuts[i].1, shortcuts[j].1,
                    "{} and {} should be different",
                    shortcuts[i].0, shortcuts[j].0
                );
            }
        }
    }
}
