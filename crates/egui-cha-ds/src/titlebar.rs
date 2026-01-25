//! Custom window titlebar component
//!
//! Provides a customizable titlebar for frameless/transparent windows.
//!
//! # Features
//!
//! - Window drag to move
//! - Close, minimize, maximize buttons
//! - Custom content area (title, icons, etc.)
//! - Platform-aware button placement (macOS left, Windows/Linux right)
//!
//! # Example
//!
//! ```ignore
//! use egui_cha_ds::titlebar::{TitleBar, TitleBarStyle};
//!
//! TitleBar::new("My App")
//!     .style(TitleBarStyle::default())
//!     .show(ui, |ui| {
//!         // Optional custom content
//!         ui.label("Custom content");
//!     });
//! ```

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};

/// Titlebar button type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBarButton {
    Close,
    Minimize,
    Maximize,
}

/// Titlebar button action result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TitleBarResponse {
    pub close_clicked: bool,
    pub minimize_clicked: bool,
    pub maximize_clicked: bool,
    pub dragged: bool,
}

/// Titlebar style configuration
#[derive(Debug, Clone)]
pub struct TitleBarStyle {
    /// Height of the titlebar
    pub height: f32,
    /// Background color (None = transparent)
    pub background: Option<Color32>,
    /// Title text color
    pub title_color: Color32,
    /// Button size
    pub button_size: f32,
    /// Button spacing
    pub button_spacing: f32,
    /// Padding from edges
    pub padding: f32,
    /// Show window control buttons
    pub show_buttons: bool,
    /// Button style
    pub button_style: TitleBarButtonStyle,
}

impl Default for TitleBarStyle {
    fn default() -> Self {
        Self {
            height: 32.0,
            background: None,
            title_color: Color32::from_rgb(200, 200, 200),
            button_size: 12.0,
            button_spacing: 8.0,
            padding: 12.0,
            show_buttons: true,
            button_style: TitleBarButtonStyle::default(),
        }
    }
}

impl TitleBarStyle {
    /// Create style from theme
    pub fn from_theme(theme: &crate::Theme) -> Self {
        Self {
            height: theme.titlebar_height,
            background: Some(theme.bg_secondary.linear_multiply(theme.glass_opacity)),
            title_color: theme.text_primary,
            button_size: 12.0,
            button_spacing: 8.0,
            padding: theme.spacing_sm,
            show_buttons: true,
            button_style: TitleBarButtonStyle::default(),
        }
    }

    /// Create transparent style from theme (for vibrancy windows)
    pub fn transparent_from_theme(theme: &crate::Theme) -> Self {
        Self {
            height: theme.titlebar_height,
            background: None,
            title_color: theme.text_primary,
            button_size: 12.0,
            button_spacing: 8.0,
            padding: theme.spacing_sm,
            show_buttons: true,
            button_style: TitleBarButtonStyle::default(),
        }
    }

    /// Transparent style (for vibrancy windows)
    pub fn transparent() -> Self {
        Self {
            background: None,
            ..Default::default()
        }
    }

    /// Compact style
    pub fn compact() -> Self {
        Self {
            height: 28.0,
            button_size: 10.0,
            padding: 8.0,
            ..Default::default()
        }
    }
}

/// Button visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TitleBarButtonStyle {
    /// macOS-style traffic lights (colored circles)
    #[default]
    TrafficLights,
    /// Windows-style icons
    WindowsIcons,
    /// Minimal dots
    Minimal,
}

/// Custom titlebar component
pub struct TitleBar<'a> {
    title: &'a str,
    style: TitleBarStyle,
    buttons: Vec<TitleBarButton>,
    id: Option<egui::Id>,
}

impl<'a> TitleBar<'a> {
    /// Create a new titlebar with title
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            style: TitleBarStyle::default(),
            buttons: vec![
                TitleBarButton::Close,
                TitleBarButton::Minimize,
                TitleBarButton::Maximize,
            ],
            id: None,
        }
    }

    /// Set a unique ID for this titlebar (required when multiple titlebars in same UI)
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Some(egui::Id::new(id));
        self
    }

    /// Set titlebar style
    pub fn style(mut self, style: TitleBarStyle) -> Self {
        self.style = style;
        self
    }

    /// Set which buttons to show
    pub fn buttons(mut self, buttons: Vec<TitleBarButton>) -> Self {
        self.buttons = buttons;
        self
    }

    /// Only show close button
    pub fn close_only(mut self) -> Self {
        self.buttons = vec![TitleBarButton::Close];
        self
    }

    /// Hide all buttons
    pub fn no_buttons(mut self) -> Self {
        self.buttons = vec![];
        self.style.show_buttons = false;
        self
    }

    /// Show the titlebar
    ///
    /// Returns response indicating which buttons were clicked
    pub fn show(self, ui: &mut Ui) -> TitleBarResponse {
        self.show_with_content(ui, |_| {})
    }

    /// Show the titlebar with custom content
    pub fn show_with_content<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> TitleBarResponse {
        let mut response = TitleBarResponse::default();

        // Use provided ID or generate from title
        let base_id = self.id.unwrap_or_else(|| ui.id().with(self.title));

        // Allocate titlebar area
        let (rect, drag_response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), self.style.height),
            Sense::click_and_drag(),
        );

        // Handle drag for window movement
        if drag_response.drag_started() {
            response.dragged = true;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // Draw background
        if let Some(bg) = self.style.background {
            ui.painter().rect_filled(rect, 0.0, bg);
        }

        // Determine button position based on platform
        let buttons_on_left = cfg!(target_os = "macos");

        // Calculate positions
        let button_area_width = if self.style.show_buttons && !self.buttons.is_empty() {
            self.buttons.len() as f32 * (self.style.button_size + self.style.button_spacing)
        } else {
            0.0
        };

        // Draw buttons
        if self.style.show_buttons && !self.buttons.is_empty() {
            let buttons_start = if buttons_on_left {
                rect.min + Vec2::new(self.style.padding, self.style.height / 2.0)
            } else {
                rect.max
                    - Vec2::new(
                        self.style.padding + button_area_width,
                        -self.style.height / 2.0,
                    )
            };

            for (i, button) in self.buttons.iter().enumerate() {
                let offset = i as f32 * (self.style.button_size + self.style.button_spacing);
                let center = buttons_start + Vec2::new(offset + self.style.button_size / 2.0, 0.0);

                let button_rect =
                    Rect::from_center_size(center, Vec2::splat(self.style.button_size));

                let button_response = ui.interact(
                    button_rect,
                    base_id.with(format!("btn_{:?}", button)),
                    Sense::click(),
                );

                // Draw button
                self.draw_button(ui, button_rect, *button, button_response.hovered());

                // Handle click
                if button_response.clicked() {
                    match button {
                        TitleBarButton::Close => {
                            response.close_clicked = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        TitleBarButton::Minimize => {
                            response.minimize_clicked = true;
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        TitleBarButton::Maximize => {
                            response.maximize_clicked = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(
                                !ui.input(|i| i.viewport().maximized.unwrap_or(false)),
                            ));
                        }
                    }
                }
            }
        }

        // Draw title
        let title_x = if buttons_on_left {
            rect.min.x + self.style.padding + button_area_width + self.style.button_spacing
        } else {
            rect.min.x + self.style.padding
        };

        let title_rect = Rect::from_min_max(
            Pos2::new(title_x, rect.min.y),
            Pos2::new(
                if buttons_on_left {
                    rect.max.x - self.style.padding
                } else {
                    rect.max.x - self.style.padding - button_area_width - self.style.button_spacing
                },
                rect.max.y,
            ),
        );

        // Title text
        ui.painter().text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            self.title,
            egui::FontId::proportional(14.0),
            self.style.title_color,
        );

        // Custom content area (between title and buttons on non-macOS, or after title on macOS)
        let content_rect = if buttons_on_left {
            // macOS: content after title
            Rect::from_min_max(
                Pos2::new(title_rect.max.x, rect.min.y),
                Pos2::new(rect.max.x - self.style.padding, rect.max.y),
            )
        } else {
            // Windows/Linux: content between padding and buttons
            Rect::from_min_max(
                Pos2::new(
                    rect.min.x + self.style.padding + 100.0, // After title
                    rect.min.y,
                ),
                Pos2::new(
                    rect.max.x - self.style.padding - button_area_width - self.style.button_spacing,
                    rect.max.y,
                ),
            )
        };

        if content_rect.width() > 20.0 {
            let mut content_ui = ui.child_ui(
                content_rect,
                egui::Layout::left_to_right(egui::Align::Center),
                None,
            );
            add_contents(&mut content_ui);
        }

        response
    }

    /// Draw a single button
    fn draw_button(&self, ui: &Ui, rect: Rect, button: TitleBarButton, hovered: bool) {
        let painter = ui.painter();
        match self.style.button_style {
            TitleBarButtonStyle::TrafficLights => {
                // macOS-style colored circles
                let (color, hover_color) = match button {
                    TitleBarButton::Close => (
                        Color32::from_rgb(255, 95, 87),
                        Color32::from_rgb(255, 59, 48),
                    ),
                    TitleBarButton::Minimize => (
                        Color32::from_rgb(255, 189, 46),
                        Color32::from_rgb(255, 204, 0),
                    ),
                    TitleBarButton::Maximize => (
                        Color32::from_rgb(39, 201, 63),
                        Color32::from_rgb(40, 205, 65),
                    ),
                };

                let color = if hovered { hover_color } else { color };
                painter.circle_filled(rect.center(), rect.width() / 2.0, color);

                // Draw icon on hover
                if hovered {
                    let icon_color = Color32::from_rgba_unmultiplied(0, 0, 0, 180);
                    let center = rect.center();
                    let size = rect.width() * 0.3;

                    match button {
                        TitleBarButton::Close => {
                            // X icon
                            painter.line_segment(
                                [center - Vec2::splat(size), center + Vec2::splat(size)],
                                egui::Stroke::new(1.5, icon_color),
                            );
                            painter.line_segment(
                                [
                                    center + Vec2::new(-size, size),
                                    center + Vec2::new(size, -size),
                                ],
                                egui::Stroke::new(1.5, icon_color),
                            );
                        }
                        TitleBarButton::Minimize => {
                            // - icon
                            painter.line_segment(
                                [center - Vec2::new(size, 0.0), center + Vec2::new(size, 0.0)],
                                egui::Stroke::new(1.5, icon_color),
                            );
                        }
                        TitleBarButton::Maximize => {
                            // + or expand icon
                            painter.line_segment(
                                [center - Vec2::new(size, 0.0), center + Vec2::new(size, 0.0)],
                                egui::Stroke::new(1.5, icon_color),
                            );
                            painter.line_segment(
                                [center - Vec2::new(0.0, size), center + Vec2::new(0.0, size)],
                                egui::Stroke::new(1.5, icon_color),
                            );
                        }
                    }
                }
            }

            TitleBarButtonStyle::WindowsIcons => {
                // Windows-style icons
                let bg_color = if hovered {
                    match button {
                        TitleBarButton::Close => Color32::from_rgb(232, 17, 35),
                        _ => Color32::from_rgba_unmultiplied(255, 255, 255, 30),
                    }
                } else {
                    Color32::TRANSPARENT
                };

                let icon_color = if hovered && button == TitleBarButton::Close {
                    Color32::WHITE
                } else {
                    self.style.title_color
                };

                // Background
                painter.rect_filled(rect, 0.0, bg_color);

                let center = rect.center();
                let size = rect.width() * 0.25;

                match button {
                    TitleBarButton::Close => {
                        painter.line_segment(
                            [center - Vec2::splat(size), center + Vec2::splat(size)],
                            egui::Stroke::new(1.0, icon_color),
                        );
                        painter.line_segment(
                            [
                                center + Vec2::new(-size, size),
                                center + Vec2::new(size, -size),
                            ],
                            egui::Stroke::new(1.0, icon_color),
                        );
                    }
                    TitleBarButton::Minimize => {
                        painter.line_segment(
                            [center - Vec2::new(size, 0.0), center + Vec2::new(size, 0.0)],
                            egui::Stroke::new(1.0, icon_color),
                        );
                    }
                    TitleBarButton::Maximize => {
                        painter.rect_stroke(
                            Rect::from_center_size(center, Vec2::splat(size * 2.0)),
                            0.0,
                            egui::Stroke::new(1.0, icon_color),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }

            TitleBarButtonStyle::Minimal => {
                // Simple dots
                let color = if hovered {
                    self.style.title_color
                } else {
                    self.style.title_color.linear_multiply(0.5)
                };
                painter.circle_filled(rect.center(), rect.width() / 4.0, color);
            }
        }
    }
}

/// Simple titlebar that just provides drag area
///
/// Use when you want custom button placement or no buttons at all.
pub fn drag_area(ui: &mut Ui, height: f32) -> Response {
    let (_rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        Sense::click_and_drag(),
    );

    if response.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    response
}
