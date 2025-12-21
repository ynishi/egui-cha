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

use crate::Theme;
use egui::{
    emath::TSTransform, Color32, CornerRadius, Pos2, Rect, Scene, Sense, Stroke, Ui, Vec2,
};
use std::collections::HashMap;

/// A pane in the node layout
#[derive(Clone, Debug)]
pub struct LayoutPane {
    /// Unique identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Position in graph space
    pub position: Pos2,
    /// Desired size (width, height)
    pub size: Vec2,
    /// Whether the pane can be closed
    pub closable: bool,
    /// Whether the pane is currently collapsed
    pub collapsed: bool,
}

impl LayoutPane {
    /// Create a new pane
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            position: Pos2::ZERO,
            size: Vec2::new(300.0, 200.0),
            closable: false,
            collapsed: false,
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
}

/// Events emitted by NodeLayoutArea
#[derive(Debug, Clone)]
pub enum NodeLayoutEvent {
    /// Pane was moved
    PaneMoved { id: String, position: Pos2 },
    /// Pane was closed
    PaneClosed(String),
    /// Pane was collapsed/expanded
    PaneCollapsed { id: String, collapsed: bool },
}

/// Internal state persisted across frames
#[derive(Clone)]
struct LayoutState {
    /// Transform from graph space to screen space
    to_screen: TSTransform,
    /// Whether the transform has been initialized with the rect position
    initialized: bool,
    /// Currently dragging pane
    dragging: Option<String>,
    /// Draw order (front to back)
    draw_order: Vec<String>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            to_screen: TSTransform::IDENTITY,
            initialized: false,
            dragging: None,
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

        // Draw panes in order (back to front)
        // Collect IDs first to avoid borrow issues during iteration
        let draw_order: Vec<_> = state.draw_order.iter().rev().cloned().collect();
        for pane_id in draw_order {
            let Some(pane) = self.layout.get_pane(&pane_id) else {
                continue;
            };

            let pane_rect = Rect::from_min_size(pane.position, pane.size);

            // Skip if not visible in viewport
            if !viewport.intersects(pane_rect) {
                continue;
            }

            // Transform to screen space
            let screen_pane_rect = to_screen * pane_rect;

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
            let title_rounding = CornerRadius {
                nw: radius,
                ne: radius,
                sw: 0,
                se: 0,
            };
            painter.rect_filled(screen_title_rect, title_rounding, theme.bg_tertiary);

            // Draw title text
            let font_size = (theme.font_size_sm * to_screen.scaling).max(theme.font_size_xs);
            let text_padding = theme.spacing_sm * to_screen.scaling;
            painter.text(
                screen_title_rect.left_center() + Vec2::new(text_padding, 0.0),
                egui::Align2::LEFT_CENTER,
                &pane.title,
                egui::FontId::proportional(font_size),
                theme.text_primary,
            );

            // Title drag interaction
            let title_response = ui.interact(
                screen_title_rect,
                ui.id().with(&pane_id).with("title"),
                if self.locked {
                    Sense::hover()
                } else {
                    Sense::click_and_drag()
                },
            );

            if title_response.clicked() || title_response.drag_started() {
                pane_to_top = Some(pane_id.clone());
            }

            if !self.locked && title_response.dragged() {
                if state.dragging.is_none() {
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

            // Draw content area
            let screen_content_rect = Rect::from_min_max(
                screen_pane_rect.min + Vec2::new(0.0, scaled_title_height),
                screen_pane_rect.max,
            );

            // Clip content to canvas area
            let clipped_content_rect = screen_content_rect.intersect(rect);

            // Only draw content if there's visible space
            if clipped_content_rect.height() > theme.spacing_xs && clipped_content_rect.width() > theme.spacing_xs {
                // Create child UI for content
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(screen_content_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                child_ui.set_clip_rect(clipped_content_rect);

                // Add some padding
                child_ui.add_space(theme.spacing_xs);

                let pane_ref = self.layout.get_pane(&pane_id).unwrap();
                (self.content_fn)(&mut child_ui, pane_ref);
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
