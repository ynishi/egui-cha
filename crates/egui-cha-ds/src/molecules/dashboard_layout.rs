//! Dashboard Layout - Three-column layout with top bar
//!
//! A composable layout for dashboard-style applications with:
//! - Fixed-height top bar
//! - Collapsible left/right sidebars
//! - Main content area
//!
//! Works within any `&mut Ui` - can be used inside panels, windows, or anywhere.
//!
//! # Example
//!
//! ```ignore
//! DashboardLayout::new()
//!     .top_bar(48.0, |ui| {
//!         ui.horizontal(|ui| {
//!             Text::h2("MyApp").show(ui);
//!             ui.separator();
//!             // Nav tabs...
//!         });
//!     })
//!     .left_sidebar(SidebarConfig::new(200.0).title("Navigation"), |ui| {
//!         // Sidebar content...
//!     })
//!     .right_sidebar(SidebarConfig::new(280.0).title("Details"), |ui| {
//!         // Details panel...
//!     })
//!     .main(|ui| {
//!         // Main content...
//!     })
//!     .show(ui);
//! ```

use crate::theme::Theme;
use egui::{Color32, Pos2, Rect, Ui, Vec2};

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration for the top bar
#[derive(Clone, Debug)]
pub struct TopBarConfig {
    pub height: f32,
    pub background: Option<Color32>,
}

impl TopBarConfig {
    pub fn new(height: f32) -> Self {
        Self {
            height,
            background: None,
        }
    }

    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }
}

impl Default for TopBarConfig {
    fn default() -> Self {
        Self::new(48.0)
    }
}

/// Configuration for sidebars
#[derive(Clone, Debug)]
pub struct SidebarConfig {
    pub width: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub title: Option<String>,
    pub collapsible: bool,
    pub collapsed: bool,
    pub resizable: bool,
    pub background: Option<Color32>,
}

impl SidebarConfig {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            min_width: 100.0,
            max_width: 500.0,
            title: None,
            collapsible: false,
            collapsed: false,
            resizable: false, // Disabled by default for Ui-based layout
            background: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }
}

/// Events emitted by DashboardLayout
#[derive(Clone, Debug, PartialEq)]
pub enum DashboardEvent {
    /// Left sidebar collapsed/expanded
    LeftSidebarToggle(bool),
    /// Right sidebar collapsed/expanded
    RightSidebarToggle(bool),
    /// Left sidebar resized
    LeftSidebarResize(f32),
    /// Right sidebar resized
    RightSidebarResize(f32),
}

/// Persistent state for DashboardLayout
#[derive(Clone, Debug)]
pub struct DashboardState {
    pub left_collapsed: bool,
    pub right_collapsed: bool,
    pub left_width: f32,
    pub right_width: f32,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            left_collapsed: false,
            right_collapsed: false,
            left_width: 200.0,
            right_width: 280.0,
        }
    }
}

impl DashboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_left_width(mut self, width: f32) -> Self {
        self.left_width = width;
        self
    }

    pub fn with_right_width(mut self, width: f32) -> Self {
        self.right_width = width;
        self
    }

    /// Update state based on event
    pub fn handle_event(&mut self, event: &DashboardEvent) {
        match event {
            DashboardEvent::LeftSidebarToggle(collapsed) => {
                self.left_collapsed = *collapsed;
            }
            DashboardEvent::RightSidebarToggle(collapsed) => {
                self.right_collapsed = *collapsed;
            }
            DashboardEvent::LeftSidebarResize(width) => {
                self.left_width = *width;
            }
            DashboardEvent::RightSidebarResize(width) => {
                self.right_width = *width;
            }
        }
    }
}

// ============================================================================
// DashboardLayout Builder
// ============================================================================

type BoxedUiFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// Dashboard layout builder
///
/// Creates a three-column layout within any Ui region.
pub struct DashboardLayout<'a> {
    top_bar: Option<(TopBarConfig, BoxedUiFn<'a>)>,
    left_sidebar: Option<(SidebarConfig, BoxedUiFn<'a>)>,
    right_sidebar: Option<(SidebarConfig, BoxedUiFn<'a>)>,
    main_content: Option<BoxedUiFn<'a>>,
    state: Option<&'a mut DashboardState>,
}

impl<'a> DashboardLayout<'a> {
    pub fn new() -> Self {
        Self {
            top_bar: None,
            left_sidebar: None,
            right_sidebar: None,
            main_content: None,
            state: None,
        }
    }

    /// Set persistent state for the layout
    pub fn state(mut self, state: &'a mut DashboardState) -> Self {
        self.state = Some(state);
        self
    }

    /// Add a top bar with fixed height
    pub fn top_bar(mut self, height: f32, content: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.top_bar = Some((TopBarConfig::new(height), Box::new(content)));
        self
    }

    /// Add a top bar with configuration
    pub fn top_bar_with_config(
        mut self,
        config: TopBarConfig,
        content: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.top_bar = Some((config, Box::new(content)));
        self
    }

    /// Add a left sidebar with fixed width
    pub fn left_sidebar(mut self, width: f32, content: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.left_sidebar = Some((SidebarConfig::new(width), Box::new(content)));
        self
    }

    /// Add a left sidebar with configuration
    pub fn left_sidebar_with_config(
        mut self,
        config: SidebarConfig,
        content: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.left_sidebar = Some((config, Box::new(content)));
        self
    }

    /// Add a right sidebar with fixed width
    pub fn right_sidebar(mut self, width: f32, content: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.right_sidebar = Some((SidebarConfig::new(width), Box::new(content)));
        self
    }

    /// Add a right sidebar with configuration
    pub fn right_sidebar_with_config(
        mut self,
        config: SidebarConfig,
        content: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.right_sidebar = Some((config, Box::new(content)));
        self
    }

    /// Add main content area
    pub fn main(mut self, content: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.main_content = Some(Box::new(content));
        self
    }

    /// Show the layout within the given Ui region
    pub fn show(self, ui: &mut Ui) -> Option<DashboardEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<DashboardEvent> = None;

        // Get available rect
        let available_rect = ui.available_rect_before_wrap();
        let mut current_y = available_rect.min.y;

        // Calculate collapsed state
        let left_collapsed = self
            .state
            .as_ref()
            .map(|s| s.left_collapsed)
            .unwrap_or(false);
        let right_collapsed = self
            .state
            .as_ref()
            .map(|s| s.right_collapsed)
            .unwrap_or(false);

        // ========================================
        // Top Bar
        // ========================================
        if let Some((config, content)) = self.top_bar {
            let top_bar_rect = Rect::from_min_size(
                Pos2::new(available_rect.min.x, current_y),
                Vec2::new(available_rect.width(), config.height),
            );

            // Use bg_tertiary for better contrast, or custom background
            let bg = config.background.unwrap_or(theme.bg_tertiary);

            // Draw top bar frame
            ui.painter().rect_filled(top_bar_rect, 0.0, bg);

            // Draw bottom border
            ui.painter().hline(
                top_bar_rect.x_range(),
                top_bar_rect.max.y,
                egui::Stroke::new(1.0, theme.border),
            );

            // Create child UI for top bar content
            let mut top_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(top_bar_rect.shrink2(Vec2::new(theme.spacing_md, theme.spacing_sm))),
            );
            content(&mut top_ui);

            current_y += config.height;
        }

        // ========================================
        // Body (Left + Main + Right)
        // ========================================
        let body_rect = Rect::from_min_max(
            Pos2::new(available_rect.min.x, current_y),
            available_rect.max,
        );

        let mut left_width = 0.0;
        let mut right_width = 0.0;
        let collapsed_width = 28.0;

        // Calculate sidebar widths
        if let Some((ref config, _)) = self.left_sidebar {
            if left_collapsed {
                if config.collapsible {
                    left_width = collapsed_width;
                }
            } else if !config.collapsed {
                left_width = self
                    .state
                    .as_ref()
                    .map(|s| s.left_width)
                    .unwrap_or(config.width);
            }
        }

        if let Some((ref config, _)) = self.right_sidebar {
            if right_collapsed {
                if config.collapsible {
                    right_width = collapsed_width;
                }
            } else if !config.collapsed {
                right_width = self
                    .state
                    .as_ref()
                    .map(|s| s.right_width)
                    .unwrap_or(config.width);
            }
        }

        let main_width = (body_rect.width() - left_width - right_width).max(100.0);

        // ========================================
        // Left Sidebar
        // ========================================
        if let Some((config, content)) = self.left_sidebar {
            let sidebar_rect = Rect::from_min_size(
                Pos2::new(body_rect.min.x, body_rect.min.y),
                Vec2::new(left_width, body_rect.height()),
            );

            let bg = config.background.unwrap_or(theme.bg_secondary);

            // Draw sidebar background
            ui.painter().rect_filled(sidebar_rect, 0.0, bg);
            // Draw right border
            ui.painter().vline(
                sidebar_rect.max.x,
                sidebar_rect.y_range(),
                egui::Stroke::new(1.0, theme.border),
            );

            if left_collapsed && config.collapsible {
                // Collapsed: just show expand button
                let inner_rect = sidebar_rect.shrink(4.0);
                let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));
                if sidebar_ui.button("▶").clicked() {
                    event = Some(DashboardEvent::LeftSidebarToggle(false));
                }
            } else if !config.collapsed {
                // Normal sidebar
                let inner_rect = sidebar_rect.shrink(theme.spacing_sm);
                let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));

                // Title and collapse button
                if config.title.is_some() || config.collapsible {
                    sidebar_ui.horizontal(|ui| {
                        if let Some(title) = &config.title {
                            ui.heading(title);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if config.collapsible {
                                if ui.small_button("◀").clicked() {
                                    event = Some(DashboardEvent::LeftSidebarToggle(true));
                                }
                            }
                        });
                    });
                    sidebar_ui.separator();
                }

                // Content with scroll
                egui::ScrollArea::vertical()
                    .id_salt("dashboard_left_sidebar")
                    .show(&mut sidebar_ui, |ui| {
                        content(ui);
                    });
            }
        }

        // ========================================
        // Right Sidebar
        // ========================================
        if let Some((config, content)) = self.right_sidebar {
            let sidebar_rect = Rect::from_min_size(
                Pos2::new(body_rect.max.x - right_width, body_rect.min.y),
                Vec2::new(right_width, body_rect.height()),
            );

            let bg = config.background.unwrap_or(theme.bg_secondary);

            // Draw sidebar background
            ui.painter().rect_filled(sidebar_rect, 0.0, bg);
            // Draw left border
            ui.painter().vline(
                sidebar_rect.min.x,
                sidebar_rect.y_range(),
                egui::Stroke::new(1.0, theme.border),
            );

            if right_collapsed && config.collapsible {
                // Collapsed: just show expand button
                let inner_rect = sidebar_rect.shrink(4.0);
                let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));
                if sidebar_ui.button("◀").clicked() {
                    event = Some(DashboardEvent::RightSidebarToggle(false));
                }
            } else if !config.collapsed {
                // Normal sidebar
                let inner_rect = sidebar_rect.shrink(theme.spacing_sm);
                let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));

                // Title and collapse button
                if config.title.is_some() || config.collapsible {
                    sidebar_ui.horizontal(|ui| {
                        if config.collapsible {
                            if ui.small_button("▶").clicked() {
                                event = Some(DashboardEvent::RightSidebarToggle(true));
                            }
                        }
                        if let Some(title) = &config.title {
                            ui.heading(title);
                        }
                    });
                    sidebar_ui.separator();
                }

                // Content with scroll
                egui::ScrollArea::vertical()
                    .id_salt("dashboard_right_sidebar")
                    .show(&mut sidebar_ui, |ui| {
                        content(ui);
                    });
            }
        }

        // ========================================
        // Main Content
        // ========================================
        if let Some(content) = self.main_content {
            let main_rect = Rect::from_min_size(
                Pos2::new(body_rect.min.x + left_width, body_rect.min.y),
                Vec2::new(main_width, body_rect.height()),
            );

            // Draw main background
            ui.painter().rect_filled(main_rect, 0.0, theme.bg_primary);

            let inner_rect = main_rect.shrink(theme.spacing_md);
            let mut main_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));
            content(&mut main_ui);
        }

        // Consume the space
        ui.allocate_rect(available_rect, egui::Sense::hover());

        event
    }
}

impl<'a> Default for DashboardLayout<'a> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create a simple three-column dashboard layout
///
/// # Example
/// ```ignore
/// dashboard_3col(
///     ui,
///     200.0, // left width
///     280.0, // right width
///     |ui| { /* left sidebar */ },
///     |ui| { /* main content */ },
///     |ui| { /* right sidebar */ },
/// );
/// ```
pub fn dashboard_3col(
    ui: &mut Ui,
    left_width: f32,
    right_width: f32,
    left: impl FnOnce(&mut Ui),
    main: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    DashboardLayout::new()
        .left_sidebar(left_width, left)
        .right_sidebar(right_width, right)
        .main(main)
        .show(ui);
}

/// Create a dashboard with top bar and three columns
///
/// # Example
/// ```ignore
/// dashboard_full(
///     ui,
///     48.0,   // top bar height
///     200.0,  // left width
///     280.0,  // right width
///     |ui| { /* top bar */ },
///     |ui| { /* left sidebar */ },
///     |ui| { /* main content */ },
///     |ui| { /* right sidebar */ },
/// );
/// ```
pub fn dashboard_full(
    ui: &mut Ui,
    top_height: f32,
    left_width: f32,
    right_width: f32,
    top: impl FnOnce(&mut Ui),
    left: impl FnOnce(&mut Ui),
    main: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    DashboardLayout::new()
        .top_bar(top_height, top)
        .left_sidebar(left_width, left)
        .right_sidebar(right_width, right)
        .main(main)
        .show(ui);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_config() {
        let config = SidebarConfig::new(200.0)
            .title("Test")
            .collapsible(true)
            .min_width(150.0);

        assert_eq!(config.width, 200.0);
        assert_eq!(config.title, Some("Test".to_string()));
        assert!(config.collapsible);
        assert_eq!(config.min_width, 150.0);
    }

    #[test]
    fn test_dashboard_state() {
        let mut state = DashboardState::new();
        assert!(!state.left_collapsed);

        state.handle_event(&DashboardEvent::LeftSidebarToggle(true));
        assert!(state.left_collapsed);

        state.handle_event(&DashboardEvent::LeftSidebarResize(300.0));
        assert_eq!(state.left_width, 300.0);
    }
}
