//! WorkspaceCanvas - Flexible window layout system
//!
//! A unified layout system that supports both fixed (Tile) and free layouts
//! with optional locking for performance-critical contexts.
//!
//! # Features
//! - **Tile Mode**: Auto-arrange panes in a grid layout
//! - **Free Mode**: Drag panes freely with snap-to-edge/grid
//! - **Lock/Unlock**: Prevent accidental layout changes (ideal for Live mode)
//! - **Snap System**: Magnetic snapping to edges and other panes
//!
//! # Usage Patterns
//!
//! ## Live Mode (Fixed, Locked)
//! ```ignore
//! WorkspaceCanvas::new(&mut panes)
//!     .layout(LayoutMode::Tile { columns: None })
//!     .locked(true)  // Prevent accidental changes
//!     .show(ui, |ui, pane| { ... });
//! ```
//!
//! ## Lab Mode (Free, Unlocked)
//! ```ignore
//! WorkspaceCanvas::new(&mut panes)
//!     .layout(LayoutMode::Free)
//!     .snap_threshold(8.0)
//!     .show(ui, |ui, pane| { ... });
//! ```

use crate::Theme;
use egui::{Color32, Id, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Layout mode for workspace
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutMode {
    /// Auto-arrange panes in a grid
    /// columns = None means auto-calculate based on pane count
    Tile { columns: Option<usize> },

    /// Free positioning with drag & snap
    Free,
}

impl Default for LayoutMode {
    fn default() -> Self {
        LayoutMode::Tile { columns: None }
    }
}

/// A pane (window) in the workspace
#[derive(Clone, Debug)]
pub struct WorkspacePane {
    /// Unique identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Position (used in Free mode, computed in Tile mode)
    pub position: Pos2,
    /// Size (used in Free mode, computed in Tile mode)
    pub size: Vec2,
    /// Minimum size constraint
    pub min_size: Vec2,
    /// Whether the pane is visible
    pub visible: bool,
    /// Whether the pane is minimized
    pub minimized: bool,
    /// Order in the layout (for Tile mode)
    pub order: usize,
}

impl WorkspacePane {
    /// Create a new pane
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            position: Pos2::new(50.0, 50.0),
            size: Vec2::new(200.0, 150.0),
            min_size: Vec2::new(100.0, 80.0),
            visible: true,
            minimized: false,
            order: 0,
        }
    }

    /// Set initial position
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Pos2::new(x, y);
        self
    }

    /// Set initial size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set minimum size
    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Vec2::new(width, height);
        self
    }

    /// Set order (for Tile mode)
    pub fn with_order(mut self, order: usize) -> Self {
        self.order = order;
        self
    }

    /// Set visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Events emitted by WorkspaceCanvas
#[derive(Clone, Debug)]
pub enum WorkspaceEvent {
    /// Pane was moved
    PaneMoved { id: String, position: Pos2 },
    /// Pane was resized
    PaneResized { id: String, size: Vec2 },
    /// Pane was closed
    PaneClosed(String),
    /// Pane was minimized/restored
    PaneMinimized { id: String, minimized: bool },
    /// Pane order changed (drag reorder in Tile mode)
    PaneReordered { from: usize, to: usize },
    /// Layout mode changed
    LayoutChanged(LayoutMode),
    /// Lock state changed
    LockChanged(bool),
}

/// Snap target for visual feedback
#[derive(Clone, Debug, PartialEq)]
pub enum SnapTarget {
    /// Snapped to another pane's edge
    Pane { id: String, edge: Edge },
    /// Snapped to canvas edge
    CanvasEdge(Edge),
    /// Snapped to grid
    Grid { x: i32, y: i32 },
}

/// Edge direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Internal state for drag operations
#[derive(Clone, Debug, Default)]
struct DragState {
    /// Currently dragging pane ID
    dragging: Option<String>,
    /// Drag start position
    drag_start: Option<Pos2>,
    /// Original pane position at drag start
    original_pos: Option<Pos2>,
    /// Current snap target (for visual feedback)
    snap_target: Option<SnapTarget>,
    /// Resizing pane ID and edge
    resizing: Option<(String, ResizeEdge)>,
}

#[derive(Clone, Copy, Debug)]
enum ResizeEdge {
    Right,
    Bottom,
    BottomRight,
}

/// Internal struct for collecting pane interactions
struct PaneInteraction {
    idx: usize,
    rect: Rect,
    title_rect: Rect,
    close_rect: Option<Rect>,
    minimize_rect: Option<Rect>,
    title_hovered: bool,
    title_dragged: bool,
    close_clicked: bool,
    minimize_clicked: bool,
    resize_edge: Option<ResizeEdge>,
}

/// Flexible workspace canvas with Tile and Free layout modes
pub struct WorkspaceCanvas<'a> {
    panes: &'a mut Vec<WorkspacePane>,
    layout_mode: LayoutMode,
    locked: bool,
    snap_threshold: f32,
    grid_size: Option<f32>,
    show_grid: bool,
    gap: f32,
    title_bar_height: f32,
    show_close_buttons: bool,
    show_minimize_buttons: bool,
}

impl<'a> WorkspaceCanvas<'a> {
    /// Create a new workspace canvas
    pub fn new(panes: &'a mut Vec<WorkspacePane>) -> Self {
        Self {
            panes,
            layout_mode: LayoutMode::default(),
            locked: false,
            snap_threshold: 8.0,
            grid_size: None,
            show_grid: false,
            gap: 4.0,
            title_bar_height: 24.0,
            show_close_buttons: true,
            show_minimize_buttons: true,
        }
    }

    /// Set layout mode
    pub fn layout(mut self, mode: LayoutMode) -> Self {
        self.layout_mode = mode;
        self
    }

    /// Set locked state (prevents all layout changes)
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Set snap threshold (distance for magnetic snapping)
    pub fn snap_threshold(mut self, threshold: f32) -> Self {
        self.snap_threshold = threshold;
        self
    }

    /// Set grid size for snapping (None = no grid)
    pub fn grid(mut self, size: Option<f32>) -> Self {
        self.grid_size = size;
        self
    }

    /// Show/hide grid lines
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Set gap between panes in Tile mode
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set title bar height
    pub fn title_bar_height(mut self, height: f32) -> Self {
        self.title_bar_height = height;
        self
    }

    /// Show/hide close buttons
    pub fn show_close_buttons(mut self, show: bool) -> Self {
        self.show_close_buttons = show;
        self
    }

    /// Show/hide minimize buttons
    pub fn show_minimize_buttons(mut self, show: bool) -> Self {
        self.show_minimize_buttons = show;
        self
    }

    /// Show workspace and render pane contents
    pub fn show<F>(self, ui: &mut Ui, mut content: F) -> Vec<WorkspaceEvent>
    where
        F: FnMut(&mut Ui, &WorkspacePane),
    {
        self.show_internal(ui, &mut content)
    }

    /// TEA-style: Show workspace and emit events
    pub fn show_with<Msg, F>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        mut content: F,
        on_event: impl Fn(WorkspaceEvent) -> Msg,
    ) where
        F: FnMut(&mut Ui, &WorkspacePane),
    {
        let events = self.show_internal(ctx.ui, &mut content);
        for event in events {
            ctx.emit(on_event(event));
        }
    }

    fn show_internal<F>(self, ui: &mut Ui, content: &mut F) -> Vec<WorkspaceEvent>
    where
        F: FnMut(&mut Ui, &WorkspacePane),
    {
        let theme = Theme::current(ui.ctx());
        let mut events = Vec::new();

        // Get available rect
        let available_rect = ui.available_rect_before_wrap();
        let canvas_id = Id::new("workspace_canvas");

        // Load/save drag state
        let mut drag_state: DragState = ui
            .ctx()
            .data_mut(|d| d.get_temp(canvas_id).unwrap_or_default());

        // Allocate the canvas area
        let (rect, response) = ui.allocate_exact_size(available_rect.size(), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return events;
        }

        // Get visible panes sorted by order
        let mut visible_panes: Vec<_> = self
            .panes
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible && !p.minimized)
            .collect();
        visible_panes.sort_by_key(|(_, p)| p.order);

        // Calculate layout based on mode
        let pane_rects: Vec<(usize, Rect)> = match self.layout_mode {
            LayoutMode::Tile { columns } => {
                self.calculate_tile_layout(&visible_panes, rect, columns)
            }
            LayoutMode::Free => visible_panes
                .iter()
                .map(|(idx, pane)| (*idx, Rect::from_min_size(pane.position, pane.size)))
                .collect(),
        };

        // Collect interaction info
        let mut interactions: Vec<PaneInteraction> = Vec::new();

        for (idx, pane_rect) in &pane_rects {
            let pane = &self.panes[*idx];

            // Title bar rect
            let title_rect = Rect::from_min_size(
                pane_rect.min,
                Vec2::new(pane_rect.width(), self.title_bar_height),
            );

            // Button rects
            let button_size = self.title_bar_height - 8.0;
            let mut button_x = pane_rect.max.x - 4.0;

            let close_rect = if self.show_close_buttons {
                button_x -= button_size;
                Some(Rect::from_min_size(
                    Pos2::new(button_x, pane_rect.min.y + 4.0),
                    Vec2::splat(button_size),
                ))
            } else {
                None
            };

            let minimize_rect = if self.show_minimize_buttons {
                button_x -= button_size + 2.0;
                Some(Rect::from_min_size(
                    Pos2::new(button_x, pane_rect.min.y + 4.0),
                    Vec2::splat(button_size),
                ))
            } else {
                None
            };

            // Allocate interaction areas
            let title_response = ui.allocate_rect(title_rect, Sense::click_and_drag());
            let close_response = close_rect.map(|r| ui.allocate_rect(r, Sense::click()));
            let minimize_response = minimize_rect.map(|r| ui.allocate_rect(r, Sense::click()));

            // Check resize edge hover (only in Free mode and unlocked)
            let resize_edge = if !self.locked && matches!(self.layout_mode, LayoutMode::Free) {
                self.check_resize_edge(ui, *pane_rect)
            } else {
                None
            };

            interactions.push(PaneInteraction {
                idx: *idx,
                rect: *pane_rect,
                title_rect,
                close_rect,
                minimize_rect,
                title_hovered: title_response.hovered(),
                title_dragged: title_response.dragged() && !self.locked,
                close_clicked: close_response.map_or(false, |r| r.clicked()),
                minimize_clicked: minimize_response.map_or(false, |r| r.clicked()),
                resize_edge,
            });
        }

        // Process interactions (before drawing)
        for interaction in &interactions {
            let pane = &self.panes[interaction.idx];

            // Handle close
            if interaction.close_clicked {
                events.push(WorkspaceEvent::PaneClosed(pane.id.clone()));
            }

            // Handle minimize
            if interaction.minimize_clicked {
                events.push(WorkspaceEvent::PaneMinimized {
                    id: pane.id.clone(),
                    minimized: !pane.minimized,
                });
            }

            // Handle drag (Free mode)
            if interaction.title_dragged && matches!(self.layout_mode, LayoutMode::Free) {
                if drag_state.dragging.is_none() {
                    drag_state.dragging = Some(pane.id.clone());
                    drag_state.original_pos = Some(pane.position);
                }

                if drag_state.dragging.as_ref() == Some(&pane.id) {
                    let delta = ui.input(|i| i.pointer.delta());
                    let new_pos = pane.position + delta;

                    // Apply snapping
                    let (snapped_pos, snap_target) = self.apply_snap(
                        new_pos,
                        pane.size,
                        rect,
                        &pane_rects,
                        interaction.idx,
                    );

                    drag_state.snap_target = snap_target;

                    events.push(WorkspaceEvent::PaneMoved {
                        id: pane.id.clone(),
                        position: snapped_pos,
                    });
                }
            }

            // Draw pane content (before we borrow painter)
            let content_rect = Rect::from_min_max(
                Pos2::new(interaction.rect.min.x, interaction.rect.min.y + self.title_bar_height),
                interaction.rect.max,
            );
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
            content(&mut child_ui, pane);
        }

        // Now get painter for all drawing operations
        let painter = ui.painter();

        // Draw background
        painter.rect_filled(rect, 0.0, theme.bg_primary);

        // Draw grid if enabled
        if self.show_grid {
            if let Some(grid_size) = self.grid_size {
                self.draw_grid(painter, rect, grid_size, &theme);
            }
        }

        // Draw panes
        for interaction in &interactions {
            let pane = &self.panes[interaction.idx];
            self.draw_pane(
                painter,
                interaction,
                pane,
                &theme,
                &drag_state,
                self.locked,
            );
        }

        // Draw snap guides
        if let Some(ref target) = drag_state.snap_target {
            self.draw_snap_guide(painter, target, rect, &theme);
        }

        // Clear drag state if not dragging
        if !ui.input(|i| i.pointer.any_down()) {
            drag_state.dragging = None;
            drag_state.snap_target = None;
            drag_state.resizing = None;
        }

        // Save drag state
        ui.ctx().data_mut(|d| d.insert_temp(canvas_id, drag_state));

        // Draw lock indicator
        if self.locked {
            self.draw_lock_indicator(painter, rect, &theme);
        }

        events
    }

    fn calculate_tile_layout(
        &self,
        visible_panes: &[(usize, &WorkspacePane)],
        rect: Rect,
        columns: Option<usize>,
    ) -> Vec<(usize, Rect)> {
        if visible_panes.is_empty() {
            return Vec::new();
        }

        let count = visible_panes.len();
        let cols = columns.unwrap_or_else(|| {
            // Auto-calculate columns: sqrt-ish for balanced grid
            match count {
                1 => 1,
                2 => 2,
                3..=4 => 2,
                5..=6 => 3,
                _ => ((count as f32).sqrt().ceil() as usize).max(2),
            }
        });

        let rows = (count + cols - 1) / cols;

        let available_width = rect.width() - self.gap * (cols + 1) as f32;
        let available_height = rect.height() - self.gap * (rows + 1) as f32;

        let cell_width = available_width / cols as f32;
        let cell_height = available_height / rows as f32;

        visible_panes
            .iter()
            .enumerate()
            .map(|(i, (pane_idx, _))| {
                let col = i % cols;
                let row = i / cols;

                let x = rect.min.x + self.gap + col as f32 * (cell_width + self.gap);
                let y = rect.min.y + self.gap + row as f32 * (cell_height + self.gap);

                (*pane_idx, Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_width, cell_height)))
            })
            .collect()
    }

    fn check_resize_edge(&self, ui: &mut Ui, rect: Rect) -> Option<ResizeEdge> {
        let pointer_pos = ui.input(|i| i.pointer.hover_pos())?;
        let edge_size = 8.0;

        let right_edge = Rect::from_min_max(
            Pos2::new(rect.max.x - edge_size, rect.min.y),
            rect.max,
        );
        let bottom_edge = Rect::from_min_max(
            Pos2::new(rect.min.x, rect.max.y - edge_size),
            rect.max,
        );
        let corner = Rect::from_min_max(
            Pos2::new(rect.max.x - edge_size, rect.max.y - edge_size),
            rect.max,
        );

        if corner.contains(pointer_pos) {
            Some(ResizeEdge::BottomRight)
        } else if right_edge.contains(pointer_pos) {
            Some(ResizeEdge::Right)
        } else if bottom_edge.contains(pointer_pos) {
            Some(ResizeEdge::Bottom)
        } else {
            None
        }
    }

    fn apply_snap(
        &self,
        pos: Pos2,
        size: Vec2,
        canvas_rect: Rect,
        pane_rects: &[(usize, Rect)],
        current_idx: usize,
    ) -> (Pos2, Option<SnapTarget>) {
        let mut snapped_pos = pos;
        let mut snap_target = None;

        // Snap to canvas edges
        if (pos.x - canvas_rect.min.x).abs() < self.snap_threshold {
            snapped_pos.x = canvas_rect.min.x + self.gap;
            snap_target = Some(SnapTarget::CanvasEdge(Edge::Left));
        }
        if (pos.x + size.x - canvas_rect.max.x).abs() < self.snap_threshold {
            snapped_pos.x = canvas_rect.max.x - size.x - self.gap;
            snap_target = Some(SnapTarget::CanvasEdge(Edge::Right));
        }
        if (pos.y - canvas_rect.min.y).abs() < self.snap_threshold {
            snapped_pos.y = canvas_rect.min.y + self.gap;
            snap_target = Some(SnapTarget::CanvasEdge(Edge::Top));
        }
        if (pos.y + size.y - canvas_rect.max.y).abs() < self.snap_threshold {
            snapped_pos.y = canvas_rect.max.y - size.y - self.gap;
            snap_target = Some(SnapTarget::CanvasEdge(Edge::Bottom));
        }

        // Snap to other panes
        for (idx, other_rect) in pane_rects {
            if *idx == current_idx {
                continue;
            }

            let pane = &self.panes[*idx];

            // Snap right edge to left edge of other
            if (pos.x + size.x - other_rect.min.x).abs() < self.snap_threshold {
                snapped_pos.x = other_rect.min.x - size.x - self.gap;
                snap_target = Some(SnapTarget::Pane {
                    id: pane.id.clone(),
                    edge: Edge::Left,
                });
            }
            // Snap left edge to right edge of other
            if (pos.x - other_rect.max.x).abs() < self.snap_threshold {
                snapped_pos.x = other_rect.max.x + self.gap;
                snap_target = Some(SnapTarget::Pane {
                    id: pane.id.clone(),
                    edge: Edge::Right,
                });
            }
            // Snap bottom edge to top edge of other
            if (pos.y + size.y - other_rect.min.y).abs() < self.snap_threshold {
                snapped_pos.y = other_rect.min.y - size.y - self.gap;
                snap_target = Some(SnapTarget::Pane {
                    id: pane.id.clone(),
                    edge: Edge::Top,
                });
            }
            // Snap top edge to bottom edge of other
            if (pos.y - other_rect.max.y).abs() < self.snap_threshold {
                snapped_pos.y = other_rect.max.y + self.gap;
                snap_target = Some(SnapTarget::Pane {
                    id: pane.id.clone(),
                    edge: Edge::Bottom,
                });
            }
        }

        // Grid snap
        if let Some(grid_size) = self.grid_size {
            if snap_target.is_none() {
                let grid_x = (snapped_pos.x / grid_size).round() as i32;
                let grid_y = (snapped_pos.y / grid_size).round() as i32;
                snapped_pos.x = grid_x as f32 * grid_size;
                snapped_pos.y = grid_y as f32 * grid_size;
                snap_target = Some(SnapTarget::Grid { x: grid_x, y: grid_y });
            }
        }

        (snapped_pos, snap_target)
    }

    fn draw_pane(
        &self,
        painter: &egui::Painter,
        interaction: &PaneInteraction,
        pane: &WorkspacePane,
        theme: &Theme,
        drag_state: &DragState,
        locked: bool,
    ) {
        let is_dragging = drag_state.dragging.as_ref() == Some(&pane.id);

        // Pane background
        let bg_color = if is_dragging {
            theme.bg_tertiary
        } else {
            theme.bg_secondary
        };
        painter.rect_filled(interaction.rect, theme.radius_md, bg_color);

        // Title bar
        let title_bg = if interaction.title_hovered && !locked {
            theme.bg_tertiary
        } else {
            theme.bg_primary
        };
        painter.rect_filled(interaction.title_rect, theme.radius_md, title_bg);

        // Title text
        painter.text(
            Pos2::new(
                interaction.title_rect.min.x + theme.spacing_sm,
                interaction.title_rect.center().y,
            ),
            egui::Align2::LEFT_CENTER,
            &pane.title,
            egui::FontId::proportional(theme.font_size_sm),
            theme.text_primary,
        );

        // Lock icon if locked
        if locked {
            painter.text(
                Pos2::new(
                    interaction.title_rect.min.x + theme.spacing_xs,
                    interaction.title_rect.min.y + theme.spacing_xs,
                ),
                egui::Align2::LEFT_TOP,
                "🔒",
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );
        }

        // Close button
        if let Some(close_rect) = interaction.close_rect {
            let close_color = if interaction.close_clicked {
                theme.state_danger
            } else {
                theme.text_muted
            };
            painter.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "×",
                egui::FontId::proportional(theme.font_size_md),
                close_color,
            );
        }

        // Minimize button
        if let Some(minimize_rect) = interaction.minimize_rect {
            painter.text(
                minimize_rect.center(),
                egui::Align2::CENTER_CENTER,
                "−",
                egui::FontId::proportional(theme.font_size_md),
                theme.text_muted,
            );
        }

        // Border
        let border_color = if is_dragging {
            theme.primary
        } else {
            theme.border
        };
        painter.rect_stroke(
            interaction.rect,
            theme.radius_md,
            Stroke::new(theme.border_width, border_color),
            egui::StrokeKind::Inside,
        );

        // Resize handles (only in Free mode and unlocked)
        if !locked && matches!(self.layout_mode, LayoutMode::Free) {
            if let Some(edge) = interaction.resize_edge {
                let handle_color = theme.primary.gamma_multiply(0.5);
                let handle_size = 6.0;

                let handle_pos = match edge {
                    ResizeEdge::Right => Pos2::new(interaction.rect.max.x - 3.0, interaction.rect.center().y),
                    ResizeEdge::Bottom => Pos2::new(interaction.rect.center().x, interaction.rect.max.y - 3.0),
                    ResizeEdge::BottomRight => Pos2::new(interaction.rect.max.x - 3.0, interaction.rect.max.y - 3.0),
                };

                painter.circle_filled(handle_pos, handle_size, handle_color);
            }
        }
    }

    fn draw_snap_guide(
        &self,
        painter: &egui::Painter,
        target: &SnapTarget,
        canvas_rect: Rect,
        theme: &Theme,
    ) {
        let guide_color = theme.primary.gamma_multiply(0.7);
        let guide_stroke = Stroke::new(2.0, guide_color);

        match target {
            SnapTarget::CanvasEdge(edge) => {
                let (start, end) = match edge {
                    Edge::Left => (
                        Pos2::new(canvas_rect.min.x + self.gap, canvas_rect.min.y),
                        Pos2::new(canvas_rect.min.x + self.gap, canvas_rect.max.y),
                    ),
                    Edge::Right => (
                        Pos2::new(canvas_rect.max.x - self.gap, canvas_rect.min.y),
                        Pos2::new(canvas_rect.max.x - self.gap, canvas_rect.max.y),
                    ),
                    Edge::Top => (
                        Pos2::new(canvas_rect.min.x, canvas_rect.min.y + self.gap),
                        Pos2::new(canvas_rect.max.x, canvas_rect.min.y + self.gap),
                    ),
                    Edge::Bottom => (
                        Pos2::new(canvas_rect.min.x, canvas_rect.max.y - self.gap),
                        Pos2::new(canvas_rect.max.x, canvas_rect.max.y - self.gap),
                    ),
                };
                painter.line_segment([start, end], guide_stroke);
            }
            SnapTarget::Pane { .. } => {
                // Draw connection indicator between panes
                // (Could be enhanced with more visual feedback)
            }
            SnapTarget::Grid { x, y } => {
                if let Some(grid_size) = self.grid_size {
                    let pos = Pos2::new(*x as f32 * grid_size, *y as f32 * grid_size);
                    painter.circle_filled(pos, 4.0, guide_color);
                }
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, grid_size: f32, theme: &Theme) {
        let grid_color = Color32::from_rgba_unmultiplied(
            theme.border.r(),
            theme.border.g(),
            theme.border.b(),
            30,
        );
        let grid_stroke = Stroke::new(0.5, grid_color);

        // Vertical lines
        let mut x = rect.min.x;
        while x < rect.max.x {
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                grid_stroke,
            );
            x += grid_size;
        }

        // Horizontal lines
        let mut y = rect.min.y;
        while y < rect.max.y {
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                grid_stroke,
            );
            y += grid_size;
        }
    }

    fn draw_lock_indicator(&self, painter: &egui::Painter, rect: Rect, theme: &Theme) {
        // Subtle lock indicator in corner
        let indicator_rect = Rect::from_min_size(
            Pos2::new(rect.max.x - 40.0, rect.min.y + 4.0),
            Vec2::new(36.0, 20.0),
        );

        painter.rect_filled(
            indicator_rect,
            theme.radius_sm,
            Color32::from_rgba_unmultiplied(0, 0, 0, 100),
        );

        painter.text(
            indicator_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🔒 Lock",
            egui::FontId::proportional(theme.font_size_xs),
            theme.text_muted,
        );
    }
}
