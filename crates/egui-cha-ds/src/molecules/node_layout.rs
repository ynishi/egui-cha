//! Node Layout module - Infinite canvas pane layout
//!
//! A free-form pane layout system with infinite canvas, pan/zoom support,
//! and optional locking.
//!
//! # Features
//! - Infinite canvas with pan/zoom
//! - Free positioning of panes
//! - Lock mode to prevent changes
//! - Custom pane content via closure
//!
//! # Example
//! ```ignore
//! // In your Model
//! struct Model {
//!     layout: NodeLayout,
//! }
//!
//! // In view
//! NodeLayoutArea::new(&mut model.layout, |ui, pane| {
//!     match pane.id.as_str() {
//!         "effects" => render_effects(ui),
//!         "layers" => render_layers(ui),
//!         _ => {}
//!     }
//! })
//! .locked(false)
//! .show(ui);
//! ```

use crate::atoms::icons;
use crate::Theme;
use egui::{
    emath::TSTransform, Color32, CornerRadius, FontFamily, Pos2, Rect, RichText, Scene, Sense,
    Stroke, Ui, Vec2,
};
use std::collections::HashMap;

/// A pane in the node layout
#[derive(Clone, Debug)]
pub struct LayoutPane {
    /// Unique identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Optional title icon (Phosphor icon codepoint)
    pub title_icon: Option<&'static str>,
    /// Position in graph space
    pub position: Pos2,
    /// Desired size (width, height)
    pub size: Vec2,
    /// Size before maximize (for restore)
    pub pre_maximize_size: Option<Vec2>,
    /// Position before maximize (for restore)
    pub pre_maximize_position: Option<Pos2>,
    /// Whether the pane can be closed
    pub closable: bool,
    /// Whether the pane is currently collapsed (title bar only)
    pub collapsed: bool,
    /// Whether the pane is maximized (fills canvas)
    pub maximized: bool,
    /// Whether the pane can be resized
    pub resizable: bool,
    /// Minimum size constraint
    pub min_size: Vec2,
    /// Whether the pane is locked (no drag/resize)
    pub locked: bool,
}

impl LayoutPane {
    /// Create a new pane
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            title_icon: None,
            position: Pos2::ZERO,
            size: Vec2::new(300.0, 200.0),
            pre_maximize_size: None,
            pre_maximize_position: None,
            closable: false,
            collapsed: false,
            maximized: false,
            resizable: true,
            min_size: Vec2::new(100.0, 60.0),
            locked: false,
        }
    }

    /// Set the size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set initial position
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Pos2::new(x, y);
        self
    }

    /// Set closable
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Vec2::new(width, height);
        self
    }

    /// Set locked state
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Set title icon (Phosphor icon codepoint)
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.title_icon = Some(icon);
        self
    }
}

/// Events emitted by NodeLayoutArea
#[derive(Debug, Clone)]
pub enum NodeLayoutEvent {
    /// Pane was moved
    PaneMoved { id: String, position: Pos2 },
    /// Pane was resized
    PaneResized { id: String, size: Vec2 },
    /// Pane was collapsed/expanded (title bar only)
    PaneCollapsed { id: String, collapsed: bool },
    /// Pane was maximized/restored
    PaneMaximized { id: String, maximized: bool },
    /// Pane was locked/unlocked
    PaneLocked { id: String, locked: bool },
    /// Pane was closed
    PaneClosed(String),
}

/// Resize direction
#[derive(Clone, Copy, Debug, PartialEq)]
enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Internal state persisted across frames
#[derive(Clone)]
struct LayoutState {
    /// Transform from graph space to screen space
    to_screen: TSTransform,
    /// Whether the transform has been initialized with the rect position
    initialized: bool,
    /// Currently dragging pane (by title)
    dragging: Option<String>,
    /// Currently resizing pane
    resizing: Option<(String, ResizeEdge)>,
    /// Draw order (front to back)
    draw_order: Vec<String>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            to_screen: TSTransform::IDENTITY,
            initialized: false,
            dragging: None,
            resizing: None,
            draw_order: Vec::new(),
        }
    }
}

/// Node layout container
pub struct NodeLayout {
    panes: Vec<LayoutPane>,
    /// Map from pane id to index for quick lookup
    id_to_index: HashMap<String, usize>,
}

impl Default for NodeLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeLayout {
    /// Create a new empty layout
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            id_to_index: HashMap::new(),
        }
    }

    /// Add a pane at a position
    pub fn add_pane(&mut self, pane: LayoutPane, position: Pos2) -> &mut Self {
        let mut pane = pane;
        pane.position = position;
        let id = pane.id.clone();
        let index = self.panes.len();
        self.panes.push(pane);
        self.id_to_index.insert(id, index);
        self
    }

    /// Remove a pane by id
    pub fn remove_pane(&mut self, id: &str) -> Option<LayoutPane> {
        if let Some(index) = self.id_to_index.remove(id) {
            let pane = self.panes.remove(index);
            // Rebuild index map
            self.id_to_index.clear();
            for (i, p) in self.panes.iter().enumerate() {
                self.id_to_index.insert(p.id.clone(), i);
            }
            Some(pane)
        } else {
            None
        }
    }

    /// Get a pane by id
    pub fn get_pane(&self, id: &str) -> Option<&LayoutPane> {
        self.id_to_index.get(id).map(|&i| &self.panes[i])
    }

    /// Get a mutable pane by id
    pub fn get_pane_mut(&mut self, id: &str) -> Option<&mut LayoutPane> {
        if let Some(&i) = self.id_to_index.get(id) {
            Some(&mut self.panes[i])
        } else {
            None
        }
    }

    /// Iterate over all panes
    pub fn panes(&self) -> impl Iterator<Item = &LayoutPane> {
        self.panes.iter()
    }

    /// Iterate over all panes mutably
    pub fn panes_mut(&mut self) -> impl Iterator<Item = &mut LayoutPane> {
        self.panes.iter_mut()
    }
}

/// Node layout area widget
pub struct NodeLayoutArea<'a, F> {
    layout: &'a mut NodeLayout,
    content_fn: F,
    locked: bool,
    title_height: f32,
    content_padding: Option<f32>,
    grid_size: f32,
    grid_alpha: u8,
    min_scale: f32,
    max_scale: f32,
}

impl<'a, F> NodeLayoutArea<'a, F>
where
    F: FnMut(&mut Ui, &LayoutPane),
{
    /// Create a new node layout area
    pub fn new(layout: &'a mut NodeLayout, content_fn: F) -> Self {
        Self {
            layout,
            content_fn,
            locked: false,
            title_height: 24.0,
            content_padding: None, // Uses theme.spacing_sm by default
            grid_size: 50.0,
            grid_alpha: 30,
            min_scale: 0.25,
            max_scale: 2.0,
        }
    }

    /// Set locked state (prevents dragging and pan/zoom)
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Set title bar height
    pub fn title_height(mut self, height: f32) -> Self {
        self.title_height = height;
        self
    }

    /// Set content padding (defaults to theme.spacing_sm)
    pub fn content_padding(mut self, padding: f32) -> Self {
        self.content_padding = Some(padding);
        self
    }

    /// Set grid size
    pub fn grid_size(mut self, size: f32) -> Self {
        self.grid_size = size;
        self
    }

    /// Set grid line alpha (0-255)
    pub fn grid_alpha(mut self, alpha: u8) -> Self {
        self.grid_alpha = alpha;
        self
    }

    /// Set zoom range (min, max scale)
    pub fn zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_scale = min;
        self.max_scale = max;
        self
    }

    /// Show the layout
    pub fn show(mut self, ui: &mut Ui) -> Vec<NodeLayoutEvent> {
        let theme = Theme::current(ui.ctx());
        let mut events = Vec::new();

        // Get available rect
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());

        // Load state
        let state_id = ui.id().with("node_layout_state");
        let mut state: LayoutState = ui
            .ctx()
            .data(|d| d.get_temp(state_id))
            .unwrap_or_default();

        // Ensure draw order contains all panes
        self.sync_draw_order(&mut state);

        // Initialize transform on first frame to map graph origin to rect top-left
        if !state.initialized {
            state.to_screen = TSTransform::from_translation(rect.min.to_vec2());
            state.initialized = true;
        }

        // Handle pan/zoom (only if not locked)
        let mut to_screen = state.to_screen;
        if !self.locked {
            let mut scene_response = response.clone();
            Scene::new()
                .zoom_range(self.min_scale..=self.max_scale)
                .register_pan_and_zoom(ui, &mut scene_response, &mut to_screen);
        }

        let from_screen = to_screen.inverse();

        // Calculate viewport in graph space
        let viewport = from_screen * rect;

        // Draw background (clipped to rect)
        let bg_painter = ui.painter_at(rect);
        bg_painter.rect_filled(rect, 0.0, theme.bg_secondary);

        // Draw grid (in screen space, clipped to rect)
        self.draw_grid_screen(&bg_painter, rect, self.grid_size, self.grid_alpha, &to_screen, &theme);

        // Use clipped painter for panes (clips to canvas area)
        let painter = ui.painter_at(rect).clone();

        // Collect pane interactions
        let mut pane_to_top: Option<String> = None;
        let mut pane_moved: Option<(String, Vec2)> = None;
        let mut pane_resized: Option<(String, Vec2)> = None;
        let mut pane_collapsed: Option<(String, bool)> = None;
        let mut pane_maximized: Option<(String, bool)> = None;
        let mut pane_locked: Option<(String, bool)> = None;
        let mut pane_closed: Option<String> = None;

        // Button size for title bar
        let button_size = self.title_height * 0.7;
        let button_padding = self.title_height * 0.15;

        // Resize edge detection threshold (in screen pixels)
        let resize_edge_size = 6.0;

        // Draw panes in order (back to front)
        // Collect IDs first to avoid borrow issues during iteration
        let draw_order: Vec<_> = state.draw_order.iter().rev().cloned().collect();
        for pane_id in draw_order {
            let Some(pane) = self.layout.get_pane(&pane_id) else {
                continue;
            };

            // Calculate effective pane rect (considering maximized state)
            let (pane_rect, is_maximized) = if pane.maximized {
                // Maximized: fill the viewport
                (viewport, true)
            } else {
                (Rect::from_min_size(pane.position, pane.size), false)
            };

            // For collapsed panes, use only title height
            let effective_pane_rect = if pane.collapsed && !is_maximized {
                Rect::from_min_size(pane_rect.min, Vec2::new(pane_rect.width(), self.title_height))
            } else {
                pane_rect
            };

            // Skip if not visible in viewport (unless maximized)
            if !is_maximized && !viewport.intersects(effective_pane_rect) {
                continue;
            }

            // Transform to screen space
            let screen_pane_rect = to_screen * effective_pane_rect;

            // Draw pane frame (no clipping for infinite canvas)
            let frame_stroke = if state.dragging.as_ref() == Some(&pane_id) {
                Stroke::new(theme.border_width * 2.0, theme.primary)
            } else {
                Stroke::new(theme.border_width, theme.border)
            };

            painter.rect_filled(screen_pane_rect, theme.radius_sm, theme.bg_primary);
            painter.rect_stroke(
                screen_pane_rect,
                theme.radius_sm,
                frame_stroke,
                egui::StrokeKind::Outside,
            );

            // Draw title bar
            let scaled_title_height = self.title_height * to_screen.scaling;
            let screen_title_rect = Rect::from_min_size(
                screen_pane_rect.min,
                Vec2::new(screen_pane_rect.width(), scaled_title_height),
            );

            let radius = theme.radius_sm as u8;
            let title_rounding = if pane.collapsed {
                // All corners rounded when collapsed
                CornerRadius::same(radius)
            } else {
                CornerRadius {
                    nw: radius,
                    ne: radius,
                    sw: 0,
                    se: 0,
                }
            };
            painter.rect_filled(screen_title_rect, title_rounding, theme.bg_tertiary);

            // Calculate button positions (right side of title bar)
            let scaled_button_size = button_size * to_screen.scaling;
            let scaled_button_padding = button_padding * to_screen.scaling;
            let mut button_x = screen_title_rect.max.x - scaled_button_padding - scaled_button_size;

            // Helper to draw icon button
            let icon_font = egui::FontId::new(scaled_button_size * 0.7, FontFamily::Name("icons".into()));

            // Close button (if closable)
            if pane.closable {
                let close_rect = Rect::from_min_size(
                    Pos2::new(button_x, screen_title_rect.min.y + scaled_button_padding),
                    Vec2::splat(scaled_button_size),
                );
                let close_response = ui.interact(
                    close_rect,
                    ui.id().with(&pane_id).with("close"),
                    Sense::click(),
                );
                let close_color = if close_response.hovered() {
                    theme.state_danger
                } else {
                    theme.text_secondary
                };
                painter.text(
                    close_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icons::X,
                    icon_font.clone(),
                    close_color,
                );
                if close_response.clicked() {
                    pane_closed = Some(pane_id.clone());
                }
                button_x -= scaled_button_size + scaled_button_padding * 0.5;
            }

            // Maximize button
            let max_rect = Rect::from_min_size(
                Pos2::new(button_x, screen_title_rect.min.y + scaled_button_padding),
                Vec2::splat(scaled_button_size),
            );
            let max_response = ui.interact(
                max_rect,
                ui.id().with(&pane_id).with("maximize"),
                Sense::click(),
            );
            let max_color = if max_response.hovered() {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            let max_icon = if pane.maximized {
                icons::CORNERS_IN
            } else {
                icons::CORNERS_OUT
            };
            painter.text(
                max_rect.center(),
                egui::Align2::CENTER_CENTER,
                max_icon,
                icon_font.clone(),
                max_color,
            );
            if max_response.clicked() {
                pane_maximized = Some((pane_id.clone(), !pane.maximized));
            }
            button_x -= scaled_button_size + scaled_button_padding * 0.5;

            // Collapse button
            let collapse_rect = Rect::from_min_size(
                Pos2::new(button_x, screen_title_rect.min.y + scaled_button_padding),
                Vec2::splat(scaled_button_size),
            );
            let collapse_response = ui.interact(
                collapse_rect,
                ui.id().with(&pane_id).with("collapse"),
                Sense::click(),
            );
            let collapse_color = if collapse_response.hovered() {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            let collapse_icon = if pane.collapsed {
                icons::CARET_DOWN
            } else {
                icons::CARET_UP
            };
            painter.text(
                collapse_rect.center(),
                egui::Align2::CENTER_CENTER,
                collapse_icon,
                icon_font.clone(),
                collapse_color,
            );
            if collapse_response.clicked() {
                pane_collapsed = Some((pane_id.clone(), !pane.collapsed));
            }
            button_x -= scaled_button_size + scaled_button_padding * 0.5;

            // Lock button
            let lock_rect = Rect::from_min_size(
                Pos2::new(button_x, screen_title_rect.min.y + scaled_button_padding),
                Vec2::splat(scaled_button_size),
            );
            let lock_response = ui.interact(
                lock_rect,
                ui.id().with(&pane_id).with("lock"),
                Sense::click(),
            );
            let lock_color = if lock_response.hovered() {
                theme.text_primary
            } else if pane.locked {
                theme.primary
            } else {
                theme.text_secondary
            };
            let lock_icon = if pane.locked {
                icons::LOCK
            } else {
                icons::LOCK_OPEN
            };
            painter.text(
                lock_rect.center(),
                egui::Align2::CENTER_CENTER,
                lock_icon,
                icon_font.clone(),
                lock_color,
            );
            if lock_response.clicked() {
                pane_locked = Some((pane_id.clone(), !pane.locked));
            }

            // Draw title (icon + text, with space for buttons)
            let title_text_max_x = button_x - scaled_button_padding;
            let font_size = (theme.font_size_sm * to_screen.scaling).max(theme.font_size_xs);
            let text_padding = theme.spacing_sm * to_screen.scaling;
            let title_text_rect = Rect::from_min_max(
                screen_title_rect.min + Vec2::new(text_padding, 0.0),
                Pos2::new(title_text_max_x, screen_title_rect.max.y),
            );
            let clipped_painter = ui.painter().with_clip_rect(title_text_rect);

            // Draw title icon if present
            let mut title_x = screen_title_rect.left_center().x + text_padding;
            if let Some(icon_char) = pane.title_icon {
                let icon_font = egui::FontId::new(font_size, FontFamily::Name("icons".into()));
                let icon_galley = clipped_painter.layout_no_wrap(
                    icon_char.to_string(),
                    icon_font.clone(),
                    theme.text_secondary,
                );
                clipped_painter.galley(
                    Pos2::new(title_x, screen_title_rect.center().y - icon_galley.size().y * 0.5),
                    icon_galley.clone(),
                    theme.text_secondary,
                );
                title_x += icon_galley.size().x + text_padding * 0.5;
            }

            // Draw title text
            clipped_painter.text(
                Pos2::new(title_x, screen_title_rect.center().y),
                egui::Align2::LEFT_CENTER,
                &pane.title,
                egui::FontId::proportional(font_size),
                theme.text_primary,
            );

            // Title drag interaction (only in area before buttons)
            let title_drag_rect = Rect::from_min_max(
                screen_title_rect.min,
                Pos2::new(title_text_max_x, screen_title_rect.max.y),
            );
            let pane_is_locked = pane.locked;
            let title_response = ui.interact(
                title_drag_rect,
                ui.id().with(&pane_id).with("title_drag"),
                if self.locked || is_maximized || pane_is_locked {
                    Sense::hover()
                } else {
                    Sense::click_and_drag()
                },
            );

            if title_response.clicked() || title_response.drag_started() {
                pane_to_top = Some(pane_id.clone());
            }

            if !self.locked && !is_maximized && !pane_is_locked && title_response.dragged() {
                if state.dragging.is_none() && state.resizing.is_none() {
                    state.dragging = Some(pane_id.clone());
                }

                if state.dragging.as_ref() == Some(&pane_id) {
                    // Convert screen delta to graph delta
                    let screen_delta = title_response.drag_delta();
                    let graph_delta = screen_delta / to_screen.scaling;
                    pane_moved = Some((pane_id.clone(), graph_delta));
                }
            }

            if title_response.drag_stopped() && state.dragging.as_ref() == Some(&pane_id) {
                state.dragging = None;
            }

            // Resize handling (only if not collapsed, not maximized, resizable, and not locked)
            // Use direct pointer input to avoid stealing events from title bar
            if !pane.collapsed && !is_maximized && pane.resizable && !self.locked && !pane_is_locked {
                // Detect resize edges (check if pointer is near the pane border)
                let detect_edge = |pos: Pos2| -> Option<ResizeEdge> {
                    // Must be within expanded pane rect
                    let expanded = screen_pane_rect.expand(resize_edge_size);
                    if !expanded.contains(pos) {
                        return None;
                    }

                    let in_left = pos.x < screen_pane_rect.min.x + resize_edge_size;
                    let in_right = pos.x > screen_pane_rect.max.x - resize_edge_size;
                    let in_top = pos.y < screen_pane_rect.min.y + resize_edge_size;
                    let in_bottom = pos.y > screen_pane_rect.max.y - resize_edge_size;

                    match (in_left, in_right, in_top, in_bottom) {
                        (true, _, true, _) => Some(ResizeEdge::TopLeft),
                        (true, _, _, true) => Some(ResizeEdge::BottomLeft),
                        (_, true, true, _) => Some(ResizeEdge::TopRight),
                        (_, true, _, true) => Some(ResizeEdge::BottomRight),
                        (true, _, _, _) => Some(ResizeEdge::Left),
                        (_, true, _, _) => Some(ResizeEdge::Right),
                        (_, _, true, _) => Some(ResizeEdge::Top),
                        (_, _, _, true) => Some(ResizeEdge::Bottom),
                        _ => None,
                    }
                };

                // Get pointer state directly (avoids ui.interact stealing events)
                let pointer = ui.input(|i| i.pointer.clone());

                // Update cursor based on hover position
                if let Some(hover_pos) = pointer.hover_pos() {
                    if let Some(edge) = detect_edge(hover_pos) {
                        let cursor = match edge {
                            ResizeEdge::Left | ResizeEdge::Right => egui::CursorIcon::ResizeHorizontal,
                            ResizeEdge::Top | ResizeEdge::Bottom => egui::CursorIcon::ResizeVertical,
                            ResizeEdge::TopLeft | ResizeEdge::BottomRight => egui::CursorIcon::ResizeNwSe,
                            ResizeEdge::TopRight | ResizeEdge::BottomLeft => egui::CursorIcon::ResizeNeSw,
                        };
                        ui.ctx().set_cursor_icon(cursor);
                    }
                }

                // Handle resize drag start
                if pointer.any_pressed() && state.resizing.is_none() && state.dragging.is_none() {
                    if let Some(pos) = pointer.press_origin() {
                        if let Some(edge) = detect_edge(pos) {
                            state.resizing = Some((pane_id.clone(), edge));
                            pane_to_top = Some(pane_id.clone());
                        }
                    }
                }

                // Handle resize drag
                if let Some((ref resize_id, edge)) = state.resizing.clone() {
                    if resize_id == &pane_id {
                        if pointer.is_decidedly_dragging() {
                            let delta = pointer.delta() / to_screen.scaling;
                            let min_size = pane.min_size;

                            // Calculate new size and position based on edge
                            let mut new_pos = pane.position;
                            let mut new_size = pane.size;

                            match edge {
                                ResizeEdge::Right => {
                                    new_size.x = (new_size.x + delta.x).max(min_size.x);
                                }
                                ResizeEdge::Bottom => {
                                    new_size.y = (new_size.y + delta.y).max(min_size.y);
                                }
                                ResizeEdge::Left => {
                                    let new_width = (new_size.x - delta.x).max(min_size.x);
                                    new_pos.x += new_size.x - new_width;
                                    new_size.x = new_width;
                                }
                                ResizeEdge::Top => {
                                    let new_height = (new_size.y - delta.y).max(min_size.y);
                                    new_pos.y += new_size.y - new_height;
                                    new_size.y = new_height;
                                }
                                ResizeEdge::TopLeft => {
                                    let new_width = (new_size.x - delta.x).max(min_size.x);
                                    let new_height = (new_size.y - delta.y).max(min_size.y);
                                    new_pos.x += new_size.x - new_width;
                                    new_pos.y += new_size.y - new_height;
                                    new_size = Vec2::new(new_width, new_height);
                                }
                                ResizeEdge::TopRight => {
                                    let new_height = (new_size.y - delta.y).max(min_size.y);
                                    new_pos.y += new_size.y - new_height;
                                    new_size.x = (new_size.x + delta.x).max(min_size.x);
                                    new_size.y = new_height;
                                }
                                ResizeEdge::BottomLeft => {
                                    let new_width = (new_size.x - delta.x).max(min_size.x);
                                    new_pos.x += new_size.x - new_width;
                                    new_size.x = new_width;
                                    new_size.y = (new_size.y + delta.y).max(min_size.y);
                                }
                                ResizeEdge::BottomRight => {
                                    new_size.x = (new_size.x + delta.x).max(min_size.x);
                                    new_size.y = (new_size.y + delta.y).max(min_size.y);
                                }
                            }

                            if new_pos != pane.position {
                                pane_moved = Some((pane_id.clone(), new_pos - pane.position));
                            }
                            if new_size != pane.size {
                                pane_resized = Some((pane_id.clone(), new_size));
                            }
                        }

                        // Check if drag ended
                        if pointer.any_released() {
                            state.resizing = None;
                        }
                    }
                }
            }

            // Draw content area (only if not collapsed)
            if !pane.collapsed {
                let base_padding = self.content_padding.unwrap_or(theme.spacing_sm);
                let content_padding = base_padding * to_screen.scaling;
                let screen_content_rect = Rect::from_min_max(
                    screen_pane_rect.min + Vec2::new(content_padding, scaled_title_height + content_padding),
                    screen_pane_rect.max - Vec2::new(content_padding, content_padding),
                );

                // Clip content to canvas area
                let clipped_content_rect = screen_content_rect.intersect(rect);

                // Only draw content if there's visible space
                let min_size = theme.spacing_sm * to_screen.scaling;
                if clipped_content_rect.height() > min_size && clipped_content_rect.width() > min_size {
                    // Create child UI for content
                    let mut child_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(screen_content_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    child_ui.set_clip_rect(clipped_content_rect);
                    child_ui.spacing_mut().item_spacing = egui::vec2(theme.spacing_xs, theme.spacing_xs);

                    let pane_ref = self.layout.get_pane(&pane_id).unwrap();
                    (self.content_fn)(&mut child_ui, pane_ref);
                }
            }
        }

        // Apply pane moved
        if let Some((id, delta)) = pane_moved {
            if let Some(pane) = self.layout.get_pane_mut(&id) {
                pane.position += delta;
                events.push(NodeLayoutEvent::PaneMoved {
                    id: id.clone(),
                    position: pane.position,
                });
            }
        }

        // Apply pane resized
        if let Some((id, new_size)) = pane_resized {
            if let Some(pane) = self.layout.get_pane_mut(&id) {
                pane.size = new_size;
                events.push(NodeLayoutEvent::PaneResized {
                    id: id.clone(),
                    size: new_size,
                });
            }
        }

        // Apply pane collapsed
        if let Some((id, collapsed)) = pane_collapsed {
            if let Some(pane) = self.layout.get_pane_mut(&id) {
                pane.collapsed = collapsed;
                events.push(NodeLayoutEvent::PaneCollapsed {
                    id: id.clone(),
                    collapsed,
                });
            }
        }

        // Apply pane maximized
        if let Some((id, maximized)) = pane_maximized {
            if let Some(pane) = self.layout.get_pane_mut(&id) {
                if maximized {
                    // Store current size/position for restore
                    pane.pre_maximize_size = Some(pane.size);
                    pane.pre_maximize_position = Some(pane.position);
                } else {
                    // Restore previous size/position
                    if let Some(size) = pane.pre_maximize_size.take() {
                        pane.size = size;
                    }
                    if let Some(pos) = pane.pre_maximize_position.take() {
                        pane.position = pos;
                    }
                }
                pane.maximized = maximized;
                events.push(NodeLayoutEvent::PaneMaximized {
                    id: id.clone(),
                    maximized,
                });
            }
        }

        // Apply pane locked
        if let Some((id, locked)) = pane_locked {
            if let Some(pane) = self.layout.get_pane_mut(&id) {
                pane.locked = locked;
                events.push(NodeLayoutEvent::PaneLocked {
                    id: id.clone(),
                    locked,
                });
            }
        }

        // Apply pane closed
        if let Some(id) = pane_closed {
            self.layout.remove_pane(&id);
            events.push(NodeLayoutEvent::PaneClosed(id));
        }

        // Bring pane to top
        if let Some(id) = pane_to_top {
            if let Some(pos) = state.draw_order.iter().position(|x| x == &id) {
                state.draw_order.remove(pos);
                state.draw_order.insert(0, id);
            }
        }

        // Save state
        state.to_screen = to_screen;
        ui.ctx().data_mut(|d| d.insert_temp(state_id, state));

        events
    }

    fn sync_draw_order(&self, state: &mut LayoutState) {
        // Remove panes that no longer exist
        state
            .draw_order
            .retain(|id| self.layout.id_to_index.contains_key(id));

        // Add new panes
        for pane in &self.layout.panes {
            if !state.draw_order.contains(&pane.id) {
                state.draw_order.push(pane.id.clone());
            }
        }
    }

    fn draw_grid_screen(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        grid_size: f32,
        grid_alpha: u8,
        to_screen: &TSTransform,
        theme: &Theme,
    ) {
        let grid_color = Color32::from_rgba_unmultiplied(
            theme.border.r(),
            theme.border.g(),
            theme.border.b(),
            grid_alpha,
        );

        let from_screen = to_screen.inverse();
        let viewport = from_screen * rect;

        // Calculate screen-space grid size
        let screen_grid_size = grid_size * to_screen.scaling;

        // Don't draw grid if too small or too large
        if screen_grid_size < 10.0 || screen_grid_size > 500.0 {
            return;
        }

        // Vertical lines
        let start_x = (viewport.min.x / grid_size).floor() * grid_size;
        let mut x = start_x;
        while x <= viewport.max.x {
            let screen_x = to_screen.translation.x + x * to_screen.scaling;
            if screen_x >= rect.min.x && screen_x <= rect.max.x {
                painter.line_segment(
                    [Pos2::new(screen_x, rect.min.y), Pos2::new(screen_x, rect.max.y)],
                    Stroke::new(theme.stroke_width, grid_color),
                );
            }
            x += grid_size;
        }

        // Horizontal lines
        let start_y = (viewport.min.y / grid_size).floor() * grid_size;
        let mut y = start_y;
        while y <= viewport.max.y {
            let screen_y = to_screen.translation.y + y * to_screen.scaling;
            if screen_y >= rect.min.y && screen_y <= rect.max.y {
                painter.line_segment(
                    [Pos2::new(rect.min.x, screen_y), Pos2::new(rect.max.x, screen_y)],
                    Stroke::new(theme.stroke_width, grid_color),
                );
            }
            y += grid_size;
        }
    }
}
