//! Dynamic input binding system
//!
//! Provides a flexible way to manage keyboard shortcuts that can be
//! rebound at runtime. This is Phase 2 of the keyboard shortcuts system,
//! building on top of the static shortcuts in the `shortcuts` module.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  ActionBindings<A>                                      │
//! │  Maps application actions to shortcuts                  │
//! │  - rebind(), reset(), find_conflicts()                  │
//! ├─────────────────────────────────────────────────────────┤
//! │  DynamicShortcut                                        │
//! │  Runtime-modifiable keyboard shortcut                   │
//! │  - Modifiers + Key, serde support                       │
//! ├─────────────────────────────────────────────────────────┤
//! │  InputBinding trait                                     │
//! │  Abstraction over KeyboardShortcut, DynamicShortcut     │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use egui_cha::bindings::{ActionBindings, DynamicShortcut};
//! use egui_cha::shortcuts;
//!
//! #[derive(Clone, PartialEq, Eq, Hash)]
//! enum Action {
//!     Save,
//!     Undo,
//!     Redo,
//! }
//!
//! // Create bindings with defaults
//! let mut bindings = ActionBindings::new()
//!     .with_default(Action::Save, shortcuts::SAVE)
//!     .with_default(Action::Undo, shortcuts::UNDO)
//!     .with_default(Action::Redo, shortcuts::REDO);
//!
//! // User rebinds Save to Ctrl+Shift+S
//! bindings.rebind(
//!     &Action::Save,
//!     DynamicShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::S),
//! );
//!
//! // In view function
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     ctx.on_action(&bindings, &Action::Save, Msg::Save);
//! }
//! ```

use egui::{Context, Key, KeyboardShortcut, Modifiers};
use std::collections::HashMap;
use std::hash::Hash;

/// Abstraction over different types of input bindings.
///
/// This trait allows treating static `KeyboardShortcut` constants and
/// dynamic `DynamicShortcut` values uniformly.
pub trait InputBinding {
    /// Check if this binding was triggered (does not consume the input).
    fn matches(&self, ctx: &Context) -> bool;

    /// Consume the input and return whether it was triggered.
    ///
    /// Once consumed, the shortcut won't trigger other handlers.
    fn consume(&self, ctx: &Context) -> bool;

    /// Get a human-readable representation of this binding.
    ///
    /// Useful for displaying in menus or help screens.
    /// Example: "⌘S" or "Ctrl+S"
    fn display(&self) -> String;

    /// Convert to KeyboardShortcut if possible.
    fn as_keyboard_shortcut(&self) -> Option<KeyboardShortcut>;
}

impl InputBinding for KeyboardShortcut {
    fn matches(&self, ctx: &Context) -> bool {
        ctx.input(|i| i.modifiers == self.modifiers && i.key_pressed(self.logical_key))
    }

    fn consume(&self, ctx: &Context) -> bool {
        ctx.input_mut(|i| i.consume_shortcut(self))
    }

    fn display(&self) -> String {
        self.format(&modifier_names(), self.logical_key == Key::Plus)
    }

    fn as_keyboard_shortcut(&self) -> Option<KeyboardShortcut> {
        Some(*self)
    }
}

/// A keyboard shortcut that can be modified at runtime.
///
/// Unlike `KeyboardShortcut` which is typically a `const`, `DynamicShortcut`
/// is designed for user-configurable keybindings.
///
/// # Serialization
///
/// When the `serde` feature is enabled, this type can be serialized/deserialized
/// for saving user preferences.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DynamicShortcut {
    /// The modifier keys (Ctrl, Shift, Alt, etc.)
    pub modifiers: Modifiers,
    /// The main key
    pub key: Key,
}

impl DynamicShortcut {
    /// Create a new dynamic shortcut.
    pub const fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Create a shortcut with no modifiers.
    pub const fn key_only(key: Key) -> Self {
        Self::new(Modifiers::NONE, key)
    }

    /// Convert to egui's KeyboardShortcut.
    pub const fn to_keyboard_shortcut(&self) -> KeyboardShortcut {
        KeyboardShortcut::new(self.modifiers, self.key)
    }
}

impl From<KeyboardShortcut> for DynamicShortcut {
    fn from(shortcut: KeyboardShortcut) -> Self {
        Self {
            modifiers: shortcut.modifiers,
            key: shortcut.logical_key,
        }
    }
}

impl From<DynamicShortcut> for KeyboardShortcut {
    fn from(shortcut: DynamicShortcut) -> Self {
        KeyboardShortcut::new(shortcut.modifiers, shortcut.key)
    }
}

impl InputBinding for DynamicShortcut {
    fn matches(&self, ctx: &Context) -> bool {
        self.to_keyboard_shortcut().matches(ctx)
    }

    fn consume(&self, ctx: &Context) -> bool {
        self.to_keyboard_shortcut().consume(ctx)
    }

    fn display(&self) -> String {
        self.to_keyboard_shortcut().display()
    }

    fn as_keyboard_shortcut(&self) -> Option<KeyboardShortcut> {
        Some(self.to_keyboard_shortcut())
    }
}

/// Manages the mapping between application actions and keyboard shortcuts.
///
/// This struct maintains both the current bindings and the defaults,
/// allowing users to customize shortcuts while being able to reset to defaults.
///
/// # Type Parameter
///
/// `A` - The action type. Typically an enum representing all possible
/// keyboard-triggered actions in your application.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, PartialEq, Eq, Hash)]
/// enum Action {
///     NewFile,
///     Open,
///     Save,
///     Undo,
///     Redo,
/// }
///
/// let bindings = ActionBindings::new()
///     .with_default(Action::NewFile, shortcuts::NEW)
///     .with_default(Action::Open, shortcuts::OPEN)
///     .with_default(Action::Save, shortcuts::SAVE)
///     .with_default(Action::Undo, shortcuts::UNDO)
///     .with_default(Action::Redo, shortcuts::REDO);
/// ```
#[derive(Clone, Debug)]
pub struct ActionBindings<A> {
    /// Current bindings (may differ from defaults after user customization)
    bindings: HashMap<A, DynamicShortcut>,
    /// Default bindings (used for reset)
    defaults: HashMap<A, DynamicShortcut>,
}

impl<A> Default for ActionBindings<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> ActionBindings<A> {
    /// Create a new empty ActionBindings.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            defaults: HashMap::new(),
        }
    }
}

impl<A: Eq + Hash + Clone> ActionBindings<A> {
    /// Register a default binding for an action (builder pattern).
    ///
    /// This sets both the default and the current binding.
    pub fn with_default(mut self, action: A, shortcut: impl Into<DynamicShortcut>) -> Self {
        self.register_default(action, shortcut);
        self
    }

    /// Register a default binding for an action.
    ///
    /// This sets both the default and the current binding.
    pub fn register_default(&mut self, action: A, shortcut: impl Into<DynamicShortcut>) {
        let shortcut = shortcut.into();
        self.defaults.insert(action.clone(), shortcut.clone());
        self.bindings.insert(action, shortcut);
    }

    /// Register multiple defaults at once.
    ///
    /// # Example
    /// ```ignore
    /// bindings.register_defaults([
    ///     (Action::Save, shortcuts::SAVE),
    ///     (Action::Undo, shortcuts::UNDO),
    ///     (Action::Redo, shortcuts::REDO),
    /// ]);
    /// ```
    pub fn register_defaults<I, S>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (A, S)>,
        S: Into<DynamicShortcut>,
    {
        for (action, shortcut) in iter {
            self.register_default(action, shortcut);
        }
    }

    /// Rebind an action to a new shortcut.
    ///
    /// Returns the previous binding, if any.
    pub fn rebind(&mut self, action: &A, shortcut: DynamicShortcut) -> Option<DynamicShortcut> {
        self.bindings.insert(action.clone(), shortcut)
    }

    /// Reset an action to its default binding.
    ///
    /// Returns true if the action had a default to reset to.
    pub fn reset(&mut self, action: &A) -> bool {
        if let Some(default) = self.defaults.get(action) {
            self.bindings.insert(action.clone(), default.clone());
            true
        } else {
            false
        }
    }

    /// Reset all actions to their default bindings.
    pub fn reset_all(&mut self) {
        self.bindings = self.defaults.clone();
    }

    /// Get the current binding for an action.
    pub fn get(&self, action: &A) -> Option<&DynamicShortcut> {
        self.bindings.get(action)
    }

    /// Get the default binding for an action.
    pub fn get_default(&self, action: &A) -> Option<&DynamicShortcut> {
        self.defaults.get(action)
    }

    /// Check if an action's binding has been modified from its default.
    pub fn is_modified(&self, action: &A) -> bool {
        match (self.bindings.get(action), self.defaults.get(action)) {
            (Some(current), Some(default)) => current != default,
            _ => false,
        }
    }

    /// Find the action bound to a given shortcut.
    ///
    /// Useful for displaying "already bound to X" messages in a keybinding UI.
    pub fn find_action(&self, shortcut: &DynamicShortcut) -> Option<&A> {
        self.bindings
            .iter()
            .find(|(_, s)| *s == shortcut)
            .map(|(a, _)| a)
    }

    /// Find all pairs of actions that share the same shortcut.
    ///
    /// Returns an empty Vec if there are no conflicts.
    pub fn find_conflicts(&self) -> Vec<(&A, &A)> {
        let mut conflicts = Vec::new();
        let actions: Vec<_> = self.bindings.keys().collect();

        for i in 0..actions.len() {
            for j in (i + 1)..actions.len() {
                if self.bindings.get(actions[i]) == self.bindings.get(actions[j]) {
                    conflicts.push((actions[i], actions[j]));
                }
            }
        }

        conflicts
    }

    /// Get an iterator over all (action, shortcut) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&A, &DynamicShortcut)> {
        self.bindings.iter()
    }

    /// Get the number of registered bindings.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if there are no bindings.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Remove a binding entirely.
    ///
    /// This removes both the current binding and the default.
    pub fn remove(&mut self, action: &A) -> Option<DynamicShortcut> {
        self.defaults.remove(action);
        self.bindings.remove(action)
    }

    /// Check if the given shortcut was triggered and consume it.
    ///
    /// Returns Some(action) if a bound action was triggered, None otherwise.
    pub fn check_triggered(&self, ctx: &Context) -> Option<&A> {
        for (action, shortcut) in &self.bindings {
            if shortcut.consume(ctx) {
                return Some(action);
            }
        }
        None
    }
}

/// Helper to get modifier key names.
/// Always uses text names (Cmd, Ctrl, etc.) instead of symbols (⌘, ⌃)
/// to avoid font rendering issues.
fn modifier_names() -> egui::ModifierNames<'static> {
    // Always use NAMES to avoid symbol rendering issues with icon fonts
    egui::ModifierNames::NAMES
}

/// A group of shortcuts where any one can trigger the action.
///
/// Useful for supporting multiple shortcuts for the same action,
/// like both "Cmd+Z" and "Ctrl+Z" for undo on different platforms.
#[derive(Clone, Debug, Default)]
pub struct ShortcutGroup {
    shortcuts: Vec<DynamicShortcut>,
}

impl ShortcutGroup {
    /// Create a new empty shortcut group.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a shortcut to the group.
    pub fn with(mut self, shortcut: impl Into<DynamicShortcut>) -> Self {
        self.shortcuts.push(shortcut.into());
        self
    }

    /// Add a shortcut to the group.
    pub fn add(&mut self, shortcut: impl Into<DynamicShortcut>) {
        self.shortcuts.push(shortcut.into());
    }

    /// Check if any shortcut in the group matches.
    pub fn matches(&self, ctx: &Context) -> bool {
        self.shortcuts.iter().any(|s| s.matches(ctx))
    }

    /// Consume the first matching shortcut and return whether any matched.
    pub fn consume(&self, ctx: &Context) -> bool {
        for shortcut in &self.shortcuts {
            if shortcut.consume(ctx) {
                return true;
            }
        }
        false
    }
}

impl InputBinding for ShortcutGroup {
    fn matches(&self, ctx: &Context) -> bool {
        ShortcutGroup::matches(self, ctx)
    }

    fn consume(&self, ctx: &Context) -> bool {
        ShortcutGroup::consume(self, ctx)
    }

    fn display(&self) -> String {
        self.shortcuts
            .iter()
            .map(|s| s.display())
            .collect::<Vec<_>>()
            .join(" / ")
    }

    fn as_keyboard_shortcut(&self) -> Option<KeyboardShortcut> {
        self.shortcuts.first().and_then(|s| s.as_keyboard_shortcut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum TestAction {
        Save,
        Undo,
        Redo,
        Copy,
    }

    #[test]
    fn test_dynamic_shortcut_creation() {
        let shortcut = DynamicShortcut::new(Modifiers::COMMAND, Key::S);
        assert_eq!(shortcut.modifiers, Modifiers::COMMAND);
        assert_eq!(shortcut.key, Key::S);
    }

    #[test]
    fn test_dynamic_shortcut_from_keyboard_shortcut() {
        let ks = KeyboardShortcut::new(Modifiers::CTRL, Key::Z);
        let ds = DynamicShortcut::from(ks);
        assert_eq!(ds.modifiers, Modifiers::CTRL);
        assert_eq!(ds.key, Key::Z);
    }

    #[test]
    fn test_action_bindings_defaults() {
        let bindings = ActionBindings::new()
            .with_default(
                TestAction::Save,
                DynamicShortcut::new(Modifiers::COMMAND, Key::S),
            )
            .with_default(
                TestAction::Undo,
                DynamicShortcut::new(Modifiers::COMMAND, Key::Z),
            );

        assert_eq!(bindings.len(), 2);
        assert_eq!(
            bindings.get(&TestAction::Save),
            Some(&DynamicShortcut::new(Modifiers::COMMAND, Key::S))
        );
    }

    #[test]
    fn test_action_bindings_rebind() {
        let mut bindings = ActionBindings::new().with_default(
            TestAction::Save,
            DynamicShortcut::new(Modifiers::COMMAND, Key::S),
        );

        // Rebind to a different shortcut
        let old = bindings.rebind(
            &TestAction::Save,
            DynamicShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::S),
        );

        assert_eq!(
            old,
            Some(DynamicShortcut::new(Modifiers::COMMAND, Key::S))
        );
        assert_eq!(
            bindings.get(&TestAction::Save),
            Some(&DynamicShortcut::new(
                Modifiers::CTRL.plus(Modifiers::SHIFT),
                Key::S
            ))
        );
        assert!(bindings.is_modified(&TestAction::Save));
    }

    #[test]
    fn test_action_bindings_reset() {
        let mut bindings = ActionBindings::new().with_default(
            TestAction::Save,
            DynamicShortcut::new(Modifiers::COMMAND, Key::S),
        );

        // Modify and then reset
        bindings.rebind(
            &TestAction::Save,
            DynamicShortcut::new(Modifiers::CTRL, Key::S),
        );
        assert!(bindings.is_modified(&TestAction::Save));

        bindings.reset(&TestAction::Save);
        assert!(!bindings.is_modified(&TestAction::Save));
        assert_eq!(
            bindings.get(&TestAction::Save),
            Some(&DynamicShortcut::new(Modifiers::COMMAND, Key::S))
        );
    }

    #[test]
    fn test_find_action() {
        let bindings = ActionBindings::new()
            .with_default(
                TestAction::Save,
                DynamicShortcut::new(Modifiers::COMMAND, Key::S),
            )
            .with_default(
                TestAction::Undo,
                DynamicShortcut::new(Modifiers::COMMAND, Key::Z),
            );

        let found = bindings.find_action(&DynamicShortcut::new(Modifiers::COMMAND, Key::S));
        assert_eq!(found, Some(&TestAction::Save));

        let not_found = bindings.find_action(&DynamicShortcut::new(Modifiers::COMMAND, Key::X));
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_find_conflicts() {
        let mut bindings = ActionBindings::new()
            .with_default(
                TestAction::Save,
                DynamicShortcut::new(Modifiers::COMMAND, Key::S),
            )
            .with_default(
                TestAction::Undo,
                DynamicShortcut::new(Modifiers::COMMAND, Key::Z),
            );

        // No conflicts initially
        assert!(bindings.find_conflicts().is_empty());

        // Create a conflict
        bindings.rebind(
            &TestAction::Undo,
            DynamicShortcut::new(Modifiers::COMMAND, Key::S),
        );

        let conflicts = bindings.find_conflicts();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_shortcut_group() {
        let group = ShortcutGroup::new()
            .with(DynamicShortcut::new(Modifiers::COMMAND, Key::Z))
            .with(DynamicShortcut::new(Modifiers::CTRL, Key::Z));

        // Display format is platform-dependent, just verify it contains the separator
        let display = group.display();
        assert!(display.contains(" / "), "Expected separator in: {}", display);
        assert!(display.contains("Z"), "Expected key Z in: {}", display);
    }
}
