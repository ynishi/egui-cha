//! Router for page navigation in TEA applications
//!
//! # Example
//! ```ignore
//! use egui_cha::router::Router;
//!
//! #[derive(Clone, PartialEq, Default)]
//! enum Page {
//!     #[default]
//!     Home,
//!     Settings,
//!     Profile(u64),
//! }
//!
//! struct Model {
//!     router: Router<Page>,
//!     // page-specific models...
//! }
//!
//! enum Msg {
//!     Router(RouterMsg<Page>),
//!     // ...
//! }
//!
//! fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
//!     match msg {
//!         Msg::Router(router_msg) => {
//!             model.router.handle(router_msg);
//!         }
//!         // ...
//!     }
//!     Cmd::none()
//! }
//!
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     match model.router.current() {
//!         Page::Home => { /* render home */ }
//!         Page::Settings => { /* render settings */ }
//!         Page::Profile(id) => { /* render profile */ }
//!     }
//! }
//! ```

use std::collections::VecDeque;

/// Router for managing page navigation with history
#[derive(Debug, Clone)]
pub struct Router<P> {
    current: P,
    history: VecDeque<P>,
    forward_stack: Vec<P>,
    max_history: usize,
}

/// Messages for router operations
#[derive(Debug, Clone, PartialEq)]
pub enum RouterMsg<P> {
    /// Navigate to a new page
    Navigate(P),
    /// Go back in history
    Back,
    /// Go forward in history
    Forward,
    /// Replace current page without adding to history
    Replace(P),
    /// Clear all history
    ClearHistory,
}

impl<P: Clone + PartialEq> Router<P> {
    /// Create a new router with an initial page
    pub fn new(initial: P) -> Self {
        Self {
            current: initial,
            history: VecDeque::new(),
            forward_stack: Vec::new(),
            max_history: 50,
        }
    }

    /// Set maximum history size
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Get the current page
    pub fn current(&self) -> &P {
        &self.current
    }

    /// Check if current page matches
    pub fn is_at(&self, page: &P) -> bool {
        &self.current == page
    }

    /// Navigate to a new page
    pub fn navigate(&mut self, page: P) {
        if self.current == page {
            return; // Already on this page
        }

        // Push current to history
        self.history.push_back(self.current.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // Clear forward stack on new navigation
        self.forward_stack.clear();

        self.current = page;
    }

    /// Replace current page without affecting history
    pub fn replace(&mut self, page: P) {
        self.current = page;
    }

    /// Go back in history
    pub fn back(&mut self) -> bool {
        if let Some(prev) = self.history.pop_back() {
            self.forward_stack.push(self.current.clone());
            self.current = prev;
            true
        } else {
            false
        }
    }

    /// Go forward in history
    pub fn forward(&mut self) -> bool {
        if let Some(next) = self.forward_stack.pop() {
            self.history.push_back(self.current.clone());
            self.current = next;
            true
        } else {
            false
        }
    }

    /// Check if can go back
    pub fn can_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// Check if can go forward
    pub fn can_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    /// Get history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.forward_stack.clear();
    }

    /// Handle a router message
    pub fn handle(&mut self, msg: RouterMsg<P>) {
        match msg {
            RouterMsg::Navigate(page) => self.navigate(page),
            RouterMsg::Back => {
                self.back();
            }
            RouterMsg::Forward => {
                self.forward();
            }
            RouterMsg::Replace(page) => self.replace(page),
            RouterMsg::ClearHistory => self.clear_history(),
        }
    }
}

impl<P: Default + Clone + PartialEq> Default for Router<P> {
    fn default() -> Self {
        Self::new(P::default())
    }
}

// ============================================================
// ViewCtx integration
// ============================================================

use crate::ViewCtx;

impl<'a, Msg> ViewCtx<'a, Msg> {
    /// Navigate to a page (convenience method)
    pub fn navigate<P>(&mut self, page: P, to_msg: impl FnOnce(RouterMsg<P>) -> Msg) {
        self.emit(to_msg(RouterMsg::Navigate(page)));
    }

    /// Go back in router history
    pub fn router_back<P>(&mut self, to_msg: impl FnOnce(RouterMsg<P>) -> Msg) {
        self.emit(to_msg(RouterMsg::Back));
    }

    /// Go forward in router history
    pub fn router_forward<P>(&mut self, to_msg: impl FnOnce(RouterMsg<P>) -> Msg) {
        self.emit(to_msg(RouterMsg::Forward));
    }
}

// ============================================================
// Navigation helpers
// ============================================================

/// A navigation link that renders as a button
pub struct NavLink<'a, P> {
    label: &'a str,
    page: P,
    active_style: bool,
}

impl<'a, P: Clone + PartialEq> NavLink<'a, P> {
    pub fn new(label: &'a str, page: P) -> Self {
        Self {
            label,
            page,
            active_style: true,
        }
    }

    /// Disable active styling
    pub fn no_active_style(mut self) -> Self {
        self.active_style = false;
        self
    }

    /// Show the nav link
    pub fn show<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        router: &Router<P>,
        to_msg: impl FnOnce(RouterMsg<P>) -> Msg,
    ) -> bool {
        let is_active = router.is_at(&self.page);

        let response = if self.active_style && is_active {
            // Active style - could customize this more
            ctx.ui
                .add(egui::Button::new(self.label).fill(egui::Color32::from_rgb(59, 130, 246)))
        } else {
            ctx.ui.button(self.label)
        };

        if response.clicked() && !is_active {
            ctx.emit(to_msg(RouterMsg::Navigate(self.page)));
            true
        } else {
            false
        }
    }
}

/// Back button helper
pub struct BackButton<'a> {
    label: &'a str,
}

impl<'a> BackButton<'a> {
    pub fn new() -> Self {
        Self { label: "Back" }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn show<P, Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        router: &Router<P>,
        to_msg: impl FnOnce(RouterMsg<P>) -> Msg,
    ) -> bool
    where
        P: Clone + PartialEq,
    {
        let enabled = router.can_back();
        let response = ctx.ui.add_enabled(enabled, egui::Button::new(self.label));

        if response.clicked() && enabled {
            ctx.emit(to_msg(RouterMsg::Back));
            true
        } else {
            false
        }
    }
}

impl<'a> Default for BackButton<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug, Default)]
    enum TestPage {
        #[default]
        Home,
        Settings,
        Profile(u64),
    }

    #[test]
    fn test_basic_navigation() {
        let mut router = Router::new(TestPage::Home);

        assert!(router.is_at(&TestPage::Home));

        router.navigate(TestPage::Settings);
        assert!(router.is_at(&TestPage::Settings));
        assert!(router.can_back());
    }

    #[test]
    fn test_back_forward() {
        let mut router = Router::new(TestPage::Home);

        router.navigate(TestPage::Settings);
        router.navigate(TestPage::Profile(42));

        assert!(router.back());
        assert!(router.is_at(&TestPage::Settings));

        assert!(router.forward());
        assert!(router.is_at(&TestPage::Profile(42)));
    }

    #[test]
    fn test_navigate_clears_forward() {
        let mut router = Router::new(TestPage::Home);

        router.navigate(TestPage::Settings);
        router.back();

        // New navigation should clear forward stack
        router.navigate(TestPage::Profile(1));
        assert!(!router.can_forward());
    }

    #[test]
    fn test_navigate_same_page() {
        let mut router = Router::new(TestPage::Home);

        router.navigate(TestPage::Home); // Same page
        assert!(!router.can_back()); // Should not add to history
    }

    #[test]
    fn test_replace() {
        let mut router = Router::new(TestPage::Home);

        router.navigate(TestPage::Settings);
        router.replace(TestPage::Profile(1));

        assert!(router.is_at(&TestPage::Profile(1)));
        assert_eq!(router.history_len(), 1); // Only Home in history

        router.back();
        assert!(router.is_at(&TestPage::Home));
    }

    #[test]
    fn test_handle_msg() {
        let mut router = Router::new(TestPage::Home);

        router.handle(RouterMsg::Navigate(TestPage::Settings));
        assert!(router.is_at(&TestPage::Settings));

        router.handle(RouterMsg::Back);
        assert!(router.is_at(&TestPage::Home));
    }
}
