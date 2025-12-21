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
    emath::TSTransform, Color32, CornerRadius, FontFamily, Pos2, Rect, Scene, Sense, Stroke, Ui,
    Vec2,
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
    /// Lock level (None, Light, Full)
    pub lock_level: LockLevel,
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
            lock_level: LockLevel::None,
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

    /// Set lock level
    pub fn lock_level(mut self, level: LockLevel) -> Self {
        self.lock_level = level;
        self
    }

    /// Set title icon (Phosphor icon codepoint)
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.title_icon = Some(icon);
        self
    }
}

/// Lock level for panes and canvas
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LockLevel {
    /// No lock - all operations allowed
    #[default]
    None,
    /// Light lock - no move/resize, but collapse/maximize/close allowed
    Light,
    /// Full lock - all operations disabled
    Full,
}

impl LockLevel {
    /// Cycle to next lock level (None -> Light -> Full -> None)
    pub fn cycle(self) -> Self {
        match self {
            LockLevel::None => LockLevel::Light,
            LockLevel::Light => LockLevel::Full,
            LockLevel::Full => LockLevel::None,
        }
    }

    /// Check if move/resize is allowed
    pub fn allows_move_resize(self) -> bool {
        matches!(self, LockLevel::None)
    }

    /// Check if collapse/maximize/close is allowed
    pub fn allows_window_controls(self) -> bool {
        !matches!(self, LockLevel::Full)
    }
}

/// Strategy for auto-arranging panes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrangeStrategy {
    /// Just resolve overlaps with minimal movement
    #[default]
    ResolveOverlaps,
    /// Arrange in a grid layout (like WorkspaceCanvas Tile mode)
    Grid { columns: Option<usize> },
    /// Cascade windows diagonally
    Cascade,
    /// Stack horizontally
    Horizontal,
    /// Stack vertically
    Vertical,
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
    /// Pane lock level changed
    PaneLockChanged { id: String, lock_level: LockLevel },
    /// Pane was closed
    PaneClosed(String),
    /// Panes were auto-arranged
    AutoArranged {
        strategy: ArrangeStrategy,
        moved_pane_ids: Vec<String>,
    },
    /// Canvas lock level changed (from menu bar)
    CanvasLockChanged(LockLevel),
    /// Zoom to fit all panes
    ZoomToFit,
    /// Reset zoom to 100%
    ZoomReset,
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

    /// Check if any panes are overlapping (considering gap)
    pub fn has_overlaps(&self, gap: f32) -> bool {
        let rects: Vec<Rect> = self
            .panes
            .iter()
            .filter(|p| !p.collapsed && !p.maximized)
            .map(|p| Rect::from_min_size(p.position, p.size))
            .collect();
        super::layout_helpers::has_overlaps(&rects, gap)
    }

    /// Resolve overlapping panes by pushing them apart.
    ///
    /// Returns the IDs of panes that were moved.
    pub fn resolve_overlaps(&mut self, gap: f32) -> Vec<String> {
        // Collect non-collapsed, non-maximized panes with their indices
        let pane_data: Vec<(usize, Rect)> = self
            .panes
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.collapsed && !p.maximized)
            .map(|(i, p)| (i, Rect::from_min_size(p.position, p.size)))
            .collect();

        if pane_data.is_empty() {
            return Vec::new();
        }

        let rects: Vec<Rect> = pane_data.iter().map(|(_, r)| *r).collect();
        let result = super::layout_helpers::resolve_overlaps(&rects, gap, 100);

        if !result.changed {
            return Vec::new();
        }

        // Apply new positions
        let mut moved_ids = Vec::new();
        for (i, new_pos) in result.positions.iter().enumerate() {
            let pane_idx = pane_data[i].0;
            let pane = &mut self.panes[pane_idx];
            if pane.position != *new_pos {
                pane.position = *new_pos;
                moved_ids.push(pane.id.clone());
            }
        }

        moved_ids
    }

    /// Resolve overlaps while keeping panes close to their original positions.
    ///
    /// `anchor_strength` controls how strongly panes are pulled back (0.0 - 1.0).
    /// Returns the IDs of panes that were moved.
    pub fn resolve_overlaps_anchored(&mut self, gap: f32, anchor_strength: f32) -> Vec<String> {
        let pane_data: Vec<(usize, Rect)> = self
            .panes
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.collapsed && !p.maximized)
            .map(|(i, p)| (i, Rect::from_min_size(p.position, p.size)))
            .collect();

        if pane_data.is_empty() {
            return Vec::new();
        }

        let rects: Vec<Rect> = pane_data.iter().map(|(_, r)| *r).collect();
        let result =
            super::layout_helpers::resolve_overlaps_with_anchors(&rects, gap, anchor_strength, 100);

        if !result.changed {
            return Vec::new();
        }

        let mut moved_ids = Vec::new();
        for (i, new_pos) in result.positions.iter().enumerate() {
            let pane_idx = pane_data[i].0;
            let pane = &mut self.panes[pane_idx];
            if pane.position != *new_pos {
                pane.position = *new_pos;
                moved_ids.push(pane.id.clone());
            }
        }

        moved_ids
    }

    /// Count overlapping pane pairs
    pub fn count_overlaps(&self, gap: f32) -> usize {
        let rects: Vec<Rect> = self
            .panes
            .iter()
            .filter(|p| !p.collapsed && !p.maximized)
            .map(|p| Rect::from_min_size(p.position, p.size))
            .collect();
        super::layout_helpers::count_overlaps(&rects, gap)
    }

    /// Auto-arrange panes using the specified strategy.
    ///
    /// # Arguments
    /// * `strategy` - The arrangement strategy to use
    /// * `gap` - Gap between panes
    /// * `origin` - Optional origin point (defaults to current bounding box min or (0,0))
    /// * `z_order_ids` - Optional Z-order for Cascade (back-to-front pane IDs).
    ///                   First ID = back (top-left), last ID = front (bottom-right).
    ///
    /// # Returns
    /// IDs of panes that were moved
    pub fn auto_arrange(
        &mut self,
        strategy: ArrangeStrategy,
        gap: f32,
        origin: Option<Pos2>,
        z_order_ids: Option<&[String]>,
    ) -> Vec<String> {
        use super::layout_helpers;

        // Note: Sorting is now done in layout_helpers based on current positions
        // (not by ID), so the spatial arrangement is preserved.
        // Exception: Cascade uses z_order_ids if provided.

        // Collect non-collapsed, non-maximized panes
        let pane_data: Vec<(usize, Rect)> = self
            .panes
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.collapsed && !p.maximized)
            .map(|(i, p)| (i, Rect::from_min_size(p.position, p.size)))
            .collect();

        if pane_data.is_empty() {
            return Vec::new();
        }

        let rects: Vec<Rect> = pane_data.iter().map(|(_, r)| *r).collect();

        // Determine origin
        let origin = origin.unwrap_or_else(|| {
            layout_helpers::bounding_box(&rects)
                .map(|b| b.min)
                .unwrap_or(Pos2::ZERO)
        });

        // Apply the appropriate arrangement
        let result = match strategy {
            ArrangeStrategy::ResolveOverlaps => {
                layout_helpers::resolve_overlaps(&rects, gap, 100).into()
            }
            ArrangeStrategy::Grid { columns } => {
                layout_helpers::arrange_grid_proportional(&rects, columns, origin, gap)
            }
            ArrangeStrategy::Cascade => {
                let offset = Vec2::new(30.0, 30.0);

                // Build cascade order from z_order_ids if provided
                let cascade_order = z_order_ids.and_then(|ids| {
                    // Map z_order IDs (back-to-front) to pane_data indices
                    // Reverse to get back-to-front order (first = top-left)
                    let order: Vec<usize> = ids
                        .iter()
                        .rev() // Reverse: draw_order is front-to-back, we want back-to-front
                        .filter_map(|id| {
                            pane_data
                                .iter()
                                .position(|(idx, _)| self.panes[*idx].id == *id)
                        })
                        .collect();

                    if order.len() == pane_data.len() {
                        Some(order)
                    } else {
                        None // Fallback to default if not all panes matched
                    }
                });

                match cascade_order {
                    Some(order) => {
                        layout_helpers::arrange_cascade(&rects, origin, offset, Some(&order))
                    }
                    None => layout_helpers::arrange_cascade(&rects, origin, offset, None),
                }
            }
            ArrangeStrategy::Horizontal => {
                layout_helpers::arrange_horizontal(&rects, origin, gap, false)
            }
            ArrangeStrategy::Vertical => {
                layout_helpers::arrange_vertical(&rects, origin, gap, false)
            }
        };

        if !result.changed {
            return Vec::new();
        }

        // Apply new positions
        let mut moved_ids = Vec::new();
        for (i, new_pos) in result.positions.iter().enumerate() {
            let pane_idx = pane_data[i].0;
            let pane = &mut self.panes[pane_idx];
            if pane.position != *new_pos {
                pane.position = *new_pos;
                moved_ids.push(pane.id.clone());
            }
        }

        moved_ids
    }

    /// Get the bounding box of all visible panes
    pub fn bounding_box(&self) -> Option<Rect> {
        let rects: Vec<Rect> = self
            .panes
            .iter()
            .filter(|p| !p.collapsed && !p.maximized)
            .map(|p| Rect::from_min_size(p.position, p.size))
            .collect();
        super::layout_helpers::bounding_box(&rects)
    }
}

/// Node layout area widget
pub struct NodeLayoutArea<'a, F> {
    layout: &'a mut NodeLayout,
    content_fn: F,
    lock_level: LockLevel,
    title_height: f32,
    content_padding: Option<f32>,
    grid_size: f32,
    grid_alpha: u8,
    min_scale: f32,
    max_scale: f32,
    /// Whether to show the menu bar
    show_menu_bar: bool,
    /// Menu bar height
    menu_bar_height: f32,
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
            lock_level: LockLevel::None,
            title_height: 24.0,
            content_padding: None, // Uses theme.spacing_sm by default
            grid_size: 50.0,
            grid_alpha: 30,
            min_scale: 0.25,
            max_scale: 2.0,
            show_menu_bar: false,
            menu_bar_height: 28.0,
        }
    }

    /// Show/hide the menu bar
    pub fn show_menu_bar(mut self, show: bool) -> Self {
        self.show_menu_bar = show;
        self
    }

    /// Set menu bar height
    pub fn menu_bar_height(mut self, height: f32) -> Self {
        self.menu_bar_height = height;
        self
    }

    /// Set lock level (controls what operations are allowed)
    pub fn lock_level(mut self, level: LockLevel) -> Self {
        self.lock_level = level;
        self
    }

    /// Set locked state (shorthand for Full lock)
    pub fn locked(mut self, locked: bool) -> Self {
        self.lock_level = if locked {
            LockLevel::Full
        } else {
            LockLevel::None
        };
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
        let full_rect = ui.available_rect_before_wrap();

        // Calculate menu bar and canvas rects
        let (menu_rect, canvas_rect) = if self.show_menu_bar {
            let menu_rect = Rect::from_min_size(
                full_rect.min,
                Vec2::new(full_rect.width(), self.menu_bar_height),
            );
            let canvas_rect = Rect::from_min_max(
                Pos2::new(full_rect.min.x, full_rect.min.y + self.menu_bar_height),
                full_rect.max,
            );
            (Some(menu_rect), canvas_rect)
        } else {
            (None, full_rect)
        };

        // Use canvas_rect for the rest
        let rect = canvas_rect;

        // Load state (before menu bar so we can pass draw_order)
        let state_id = ui.id().with("node_layout_state");
        let mut state: LayoutState = ui.ctx().data(|d| d.get_temp(state_id)).unwrap_or_default();

        // Ensure draw order contains all panes
        self.sync_draw_order(&mut state);

        // Draw menu bar FIRST (before allocating canvas) so it can receive input
        if let Some(menu_rect) = menu_rect {
            self.draw_menu_bar(ui, menu_rect, &theme, &mut events, &state.draw_order);
        }

        // Initialize transform on first frame to map graph origin to rect top-left
        if !state.initialized {
            state.to_screen = TSTransform::from_translation(rect.min.to_vec2());
            state.initialized = true;
        }

        // Handle pan/zoom (only if canvas allows move/resize)
        let mut to_screen = state.to_screen;
        if self.lock_level.allows_move_resize() {
            // Create a response for the canvas area only
            let canvas_response = ui.allocate_rect(rect, Sense::drag());
            let mut scene_response = canvas_response;
            Scene::new()
                .zoom_range(self.min_scale..=self.max_scale)
                .register_pan_and_zoom(ui, &mut scene_response, &mut to_screen);
        }

        // Handle zoom events from menu bar
        for event in &events {
            match event {
                NodeLayoutEvent::ZoomToFit => {
                    if let Some(bounds) = self.layout.bounding_box() {
                        // Add margin around the bounding box
                        let margin = 20.0;
                        let bounds = bounds.expand(margin);

                        // Calculate scale to fit bounds in canvas
                        let scale_x = rect.width() / bounds.width();
                        let scale_y = rect.height() / bounds.height();
                        let scale = scale_x.min(scale_y).clamp(self.min_scale, self.max_scale);

                        // Calculate translation to center the bounds
                        let bounds_center = bounds.center();
                        let rect_center = rect.center();

                        to_screen = TSTransform::from_translation(
                            rect_center.to_vec2() - bounds_center.to_vec2() * scale,
                        ) * TSTransform::from_scaling(scale);
                    }
                }
                NodeLayoutEvent::ZoomReset => {
                    // Reset to 100% zoom, keeping content at origin
                    to_screen = TSTransform::from_translation(rect.min.to_vec2());
                }
                _ => {}
            }
        }

        let from_screen = to_screen.inverse();

        // Calculate viewport in graph space
        let viewport = from_screen * rect;

        // Draw background (clipped to rect)
        let bg_painter = ui.painter_at(rect);
        bg_painter.rect_filled(rect, 0.0, theme.bg_secondary);

        // Draw grid (in screen space, clipped to rect)
        self.draw_grid_screen(
            &bg_painter,
            rect,
            self.grid_size,
            self.grid_alpha,
            &to_screen,
            &theme,
        );

        // Use clipped painter for panes (clips to canvas area)
        let painter = ui.painter_at(rect).clone();

        // Collect pane interactions
        let mut pane_to_top: Option<String> = None;
        let mut pane_moved: Option<(String, Vec2)> = None;
        let mut pane_resized: Option<(String, Vec2)> = None;
        let mut pane_collapsed: Option<(String, bool)> = None;
        let mut pane_maximized: Option<(String, bool)> = None;
        let mut pane_lock_changed: Option<(String, LockLevel)> = None;
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
                Rect::from_min_size(
                    pane_rect.min,
                    Vec2::new(pane_rect.width(), self.title_height),
                )
            } else {
                pane_rect
            };

            // Transform to screen space
            let screen_pane_rect = to_screen * effective_pane_rect;

            // Skip if not visible in canvas rect (screen space check)
            if !is_maximized && !rect.intersects(screen_pane_rect) {
                continue;
            }

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
            let icon_font =
                egui::FontId::new(scaled_button_size * 0.7, FontFamily::Name("icons".into()));

            // Check if window controls are allowed (not Full locked)
            let pane_allows_window_ctrl = pane.lock_level.allows_window_controls();
            let canvas_allows_window_ctrl = self.lock_level.allows_window_controls();
            let window_ctrl_enabled = pane_allows_window_ctrl && canvas_allows_window_ctrl;

            // Close button (if closable and not full-locked)
            if pane.closable {
                let close_rect = Rect::from_min_size(
                    Pos2::new(button_x, screen_title_rect.min.y + scaled_button_padding),
                    Vec2::splat(scaled_button_size),
                );
                let close_response = ui.interact(
                    close_rect,
                    ui.id().with(&pane_id).with("close"),
                    if window_ctrl_enabled {
                        Sense::click()
                    } else {
                        Sense::hover()
                    },
                );
                let close_color = if !window_ctrl_enabled {
                    theme.text_muted
                } else if close_response.hovered() {
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
                if window_ctrl_enabled && close_response.clicked() {
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
                if window_ctrl_enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            let max_color = if !window_ctrl_enabled {
                theme.text_muted
            } else if max_response.hovered() {
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
            if window_ctrl_enabled && max_response.clicked() {
                pane_maximized = Some((pane_id.clone(), !pane.maximized));
                // Bring to front when maximizing
                if !pane.maximized {
                    pane_to_top = Some(pane_id.clone());
                }
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
                if window_ctrl_enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            let collapse_color = if !window_ctrl_enabled {
                theme.text_muted
            } else if collapse_response.hovered() {
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
            if window_ctrl_enabled && collapse_response.clicked() {
                pane_collapsed = Some((pane_id.clone(), !pane.collapsed));
            }
            button_x -= scaled_button_size + scaled_button_padding * 0.5;

            // Lock button (cycles: None -> Light -> Full -> None)
            // Disabled when canvas is Full locked
            let lock_button_enabled = canvas_allows_window_ctrl;
            let lock_rect = Rect::from_min_size(
                Pos2::new(button_x, screen_title_rect.min.y + scaled_button_padding),
                Vec2::splat(scaled_button_size),
            );
            let lock_response = ui.interact(
                lock_rect,
                ui.id().with(&pane_id).with("lock"),
                if lock_button_enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            let (lock_icon, lock_color) = if !lock_button_enabled {
                (icons::LOCK, theme.text_muted)
            } else {
                match pane.lock_level {
                    LockLevel::None => (
                        icons::LOCK_OPEN,
                        if lock_response.hovered() {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        },
                    ),
                    LockLevel::Light => (
                        icons::LOCK,
                        if lock_response.hovered() {
                            theme.text_primary
                        } else {
                            theme.primary
                        },
                    ),
                    LockLevel::Full => (
                        icons::LOCK,
                        if lock_response.hovered() {
                            theme.text_primary
                        } else {
                            theme.state_danger
                        },
                    ),
                }
            };
            painter.text(
                lock_rect.center(),
                egui::Align2::CENTER_CENTER,
                lock_icon,
                icon_font.clone(),
                lock_color,
            );
            if lock_button_enabled && lock_response.clicked() {
                pane_lock_changed = Some((pane_id.clone(), pane.lock_level.cycle()));
            }

            // Draw title (icon + text, with space for buttons)
            let title_text_max_x = button_x - scaled_button_padding;
            let font_size = (theme.font_size_sm * to_screen.scaling).max(theme.font_size_xs);
            let text_padding = theme.spacing_sm * to_screen.scaling;
            let title_text_rect = Rect::from_min_max(
                screen_title_rect.min + Vec2::new(text_padding, 0.0),
                Pos2::new(title_text_max_x, screen_title_rect.max.y),
            );
            // Clip to both title area and canvas rect
            let clipped_painter = ui.painter().with_clip_rect(title_text_rect.intersect(rect));

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
                    Pos2::new(
                        title_x,
                        screen_title_rect.center().y - icon_galley.size().y * 0.5,
                    ),
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

            // Compute effective lock: use stricter of canvas and pane lock levels
            let pane_lock = pane.lock_level;
            let can_move_resize =
                self.lock_level.allows_move_resize() && pane_lock.allows_move_resize();
            let _can_window_control =
                self.lock_level.allows_window_controls() && pane_lock.allows_window_controls();

            let title_response = ui.interact(
                title_drag_rect,
                ui.id().with(&pane_id).with("title_drag"),
                if !can_move_resize || is_maximized {
                    Sense::hover()
                } else {
                    Sense::click_and_drag()
                },
            );

            if title_response.clicked() || title_response.drag_started() {
                pane_to_top = Some(pane_id.clone());
            }

            if can_move_resize && !is_maximized && title_response.dragged() {
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

            // Resize handling (only if not collapsed, not maximized, resizable, and can move/resize)
            // Use direct pointer input to avoid stealing events from title bar
            if !pane.collapsed && !is_maximized && pane.resizable && can_move_resize {
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
                            ResizeEdge::Left | ResizeEdge::Right => {
                                egui::CursorIcon::ResizeHorizontal
                            }
                            ResizeEdge::Top | ResizeEdge::Bottom => {
                                egui::CursorIcon::ResizeVertical
                            }
                            ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                                egui::CursorIcon::ResizeNwSe
                            }
                            ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                                egui::CursorIcon::ResizeNeSw
                            }
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
                    screen_pane_rect.min
                        + Vec2::new(content_padding, scaled_title_height + content_padding),
                    screen_pane_rect.max - Vec2::new(content_padding, content_padding),
                );

                // Clip content to canvas area
                let clipped_content_rect = screen_content_rect.intersect(rect);

                // Only draw content if there's visible space
                let min_size = theme.spacing_sm * to_screen.scaling;
                if clipped_content_rect.height() > min_size
                    && clipped_content_rect.width() > min_size
                {
                    // Create child UI for content
                    let mut child_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(screen_content_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    child_ui.set_clip_rect(clipped_content_rect);
                    child_ui.spacing_mut().item_spacing =
                        egui::vec2(theme.spacing_xs, theme.spacing_xs);

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

        // Apply pane lock level change
        if let Some((id, new_level)) = pane_lock_changed {
            if let Some(pane) = self.layout.get_pane_mut(&id) {
                pane.lock_level = new_level;
                events.push(NodeLayoutEvent::PaneLockChanged {
                    id: id.clone(),
                    lock_level: new_level,
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

    /// Draw the menu bar
    fn draw_menu_bar(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        theme: &Theme,
        events: &mut Vec<NodeLayoutEvent>,
        draw_order: &[String],
    ) {
        use crate::atoms::icons;

        // Draw background
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, theme.bg_primary);
        painter.hline(
            rect.x_range(),
            rect.max.y,
            egui::Stroke::new(1.0, theme.border),
        );

        // Inset rect for better vertical centering
        let inner_rect = rect.shrink2(Vec2::new(theme.spacing_sm, 2.0));

        // Use allocate_ui_at_rect to properly handle input
        ui.allocate_ui_at_rect(inner_rect, |child_ui| {
            child_ui.horizontal_centered(|child_ui| {
                child_ui.style_mut().spacing.item_spacing = Vec2::new(4.0, 0.0);

                // Helper to create icon text
                let icon_text = |icon: &str| -> egui::RichText {
                    egui::RichText::new(icon).family(FontFamily::Name("icons".into()))
                };

                // Lock button with Phosphor icon
                let (lock_icon, lock_tooltip) = match self.lock_level {
                    LockLevel::None => (icons::LOCK_OPEN, "Unlocked - Click to lock position"),
                    LockLevel::Light => (icons::LOCK_KEY, "Position locked - Click to fully lock"),
                    LockLevel::Full => (icons::LOCK, "Fully locked - Click to unlock"),
                };

                if child_ui
                    .add(egui::Button::new(icon_text(lock_icon)).min_size(Vec2::new(24.0, 20.0)))
                    .on_hover_text(lock_tooltip)
                    .clicked()
                {
                    let new_level = match self.lock_level {
                        LockLevel::None => LockLevel::Light,
                        LockLevel::Light => LockLevel::Full,
                        LockLevel::Full => LockLevel::None,
                    };
                    self.lock_level = new_level;
                    events.push(NodeLayoutEvent::CanvasLockChanged(new_level));
                }

                child_ui.separator();

                // Arrange dropdown with Phosphor icon
                let gap = super::layout_helpers::DEFAULT_GAP;

                child_ui.menu_button(icon_text(icons::SLIDERS_HORIZONTAL), |ui| {
                    use crate::atoms::ListItem;

                    if ListItem::new("Grid")
                        .icon(icons::GRID_FOUR)
                        .compact()
                        .show(ui)
                        .clicked()
                    {
                        let moved = self.layout.auto_arrange(
                            ArrangeStrategy::Grid { columns: None },
                            gap,
                            None,
                            None,
                        );
                        if !moved.is_empty() {
                            events.push(NodeLayoutEvent::AutoArranged {
                                strategy: ArrangeStrategy::Grid { columns: None },
                                moved_pane_ids: moved,
                            });
                        }
                        ui.close();
                    }
                    if ListItem::new("Horizontal")
                        .icon(icons::ARROWS_OUT_LINE_HORIZONTAL)
                        .compact()
                        .show(ui)
                        .clicked()
                    {
                        let moved =
                            self.layout
                                .auto_arrange(ArrangeStrategy::Horizontal, gap, None, None);
                        if !moved.is_empty() {
                            events.push(NodeLayoutEvent::AutoArranged {
                                strategy: ArrangeStrategy::Horizontal,
                                moved_pane_ids: moved,
                            });
                        }
                        ui.close();
                    }
                    if ListItem::new("Vertical")
                        .icon(icons::ARROWS_OUT_LINE_VERTICAL)
                        .compact()
                        .show(ui)
                        .clicked()
                    {
                        let moved =
                            self.layout
                                .auto_arrange(ArrangeStrategy::Vertical, gap, None, None);
                        if !moved.is_empty() {
                            events.push(NodeLayoutEvent::AutoArranged {
                                strategy: ArrangeStrategy::Vertical,
                                moved_pane_ids: moved,
                            });
                        }
                        ui.close();
                    }
                    if ListItem::new("Cascade")
                        .icon(icons::SQUARES_FOUR)
                        .compact()
                        .show(ui)
                        .clicked()
                    {
                        // Use Z-order: back pane at top-left, front pane at bottom-right
                        let moved = self.layout.auto_arrange(
                            ArrangeStrategy::Cascade,
                            gap,
                            None,
                            Some(draw_order),
                        );
                        if !moved.is_empty() {
                            events.push(NodeLayoutEvent::AutoArranged {
                                strategy: ArrangeStrategy::Cascade,
                                moved_pane_ids: moved,
                            });
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ListItem::new("Resolve Overlaps")
                        .icon(icons::BROOM)
                        .compact()
                        .show(ui)
                        .clicked()
                    {
                        let moved = self.layout.auto_arrange(
                            ArrangeStrategy::ResolveOverlaps,
                            gap,
                            None,
                            None,
                        );
                        if !moved.is_empty() {
                            events.push(NodeLayoutEvent::AutoArranged {
                                strategy: ArrangeStrategy::ResolveOverlaps,
                                moved_pane_ids: moved,
                            });
                        }
                        ui.close();
                    }
                });

                child_ui.separator();

                // Zoom buttons with Phosphor icons
                if child_ui
                    .add(
                        egui::Button::new(icon_text(icons::FRAME_CORNERS))
                            .min_size(Vec2::new(28.0, 20.0)),
                    )
                    .on_hover_text("Zoom to fit all panes")
                    .clicked()
                {
                    events.push(NodeLayoutEvent::ZoomToFit);
                }

                if child_ui
                    .add(egui::Button::new("100%").min_size(Vec2::new(40.0, 20.0)))
                    .on_hover_text("Reset zoom to 100%")
                    .clicked()
                {
                    events.push(NodeLayoutEvent::ZoomReset);
                }

                // Show pane count on the right
                child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(theme.spacing_sm);
                    let visible_count = self.layout.panes.iter().filter(|p| !p.collapsed).count();
                    let total_count = self.layout.panes.len();
                    ui.label(
                        egui::RichText::new(format!("{}/{} panes", visible_count, total_count))
                            .color(theme.text_secondary)
                            .small(),
                    );
                });
            }); // end horizontal_centered
        }); // end allocate_ui_at_rect
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
                    [
                        Pos2::new(screen_x, rect.min.y),
                        Pos2::new(screen_x, rect.max.y),
                    ],
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
                    [
                        Pos2::new(rect.min.x, screen_y),
                        Pos2::new(rect.max.x, screen_y),
                    ],
                    Stroke::new(theme.stroke_width, grid_color),
                );
            }
            y += grid_size;
        }
    }
}
