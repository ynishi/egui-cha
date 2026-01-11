//! Heatmap Grid for multi-agent state visualization
//!
//! A color-coded grid for visualizing the state of multiple agents/processes.
//! Supports real-time updates, click events, hover tooltips, and flash animations.
//!
//! # Example
//! ```ignore
//! // Simple usage with state array
//! HeatmapGrid::new(10, 10)
//!     .data(&model.agent_states)
//!     .cell_size(24.0)
//!     .show_with(ctx, |row, col| Msg::SelectAgent(row, col));
//!
//! // Callback-based for complex state
//! HeatmapGrid::new(10, 10)
//!     .cell(|row, col| {
//!         let agent = &model.agents[row * 10 + col];
//!         HeatmapCell::new(agent.state)
//!             .label(&agent.id[..4])
//!             .flash(agent.just_updated)
//!     })
//!     .show(ui);
//! ```

use crate::atoms::ResponseExt;
use crate::Theme;
use egui::{Color32, Sense, Ui, Vec2};
use egui_cha::ViewCtx;

/// Cell state for color mapping
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CellState {
    /// Gray - No activity / idle
    #[default]
    Idle,
    /// Blue - Processing / forward pass running
    Processing,
    /// Green - Successfully delegated
    Delegated,
    /// Orange - Escalated to higher tier
    Escalated,
    /// Red - Error state
    Error,
    /// Custom color
    Custom(Color32),
}

impl CellState {
    /// Get the color for this state from theme
    pub fn color(&self, theme: &Theme) -> Color32 {
        match self {
            CellState::Idle => theme.text_muted.gamma_multiply(0.5),
            CellState::Processing => theme.state_info,
            CellState::Delegated => theme.state_success,
            CellState::Escalated => theme.state_warning,
            CellState::Error => theme.state_danger,
            CellState::Custom(c) => *c,
        }
    }
}

/// Data for a single heatmap cell (for callback-based usage)
#[derive(Debug, Clone)]
pub struct HeatmapCell {
    pub state: CellState,
    pub label: Option<String>,
    pub tooltip: Option<String>,
    pub flash: bool,
}

impl HeatmapCell {
    /// Create a new cell with the given state
    pub fn new(state: CellState) -> Self {
        Self {
            state,
            label: None,
            tooltip: None,
            flash: false,
        }
    }

    /// Create an idle cell
    pub fn idle() -> Self {
        Self::new(CellState::Idle)
    }

    /// Set label text (displayed in cell if space allows)
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set tooltip text (shown on hover)
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Enable flash animation (for just-updated cells)
    pub fn flash(mut self, flash: bool) -> Self {
        self.flash = flash;
        self
    }
}

impl Default for HeatmapCell {
    fn default() -> Self {
        Self::idle()
    }
}

impl From<CellState> for HeatmapCell {
    fn from(state: CellState) -> Self {
        Self::new(state)
    }
}

/// Heatmap grid for visualizing multi-agent states
pub struct HeatmapGrid<'a, F = fn(usize, usize) -> HeatmapCell>
where
    F: Fn(usize, usize) -> HeatmapCell,
{
    rows: usize,
    cols: usize,
    states: Option<&'a [CellState]>,
    cell_fn: Option<F>,
    cell_size: Option<f32>,
    spacing: Option<f32>,
    show_labels: bool,
    tooltip_fn: Option<Box<dyn Fn(usize, usize) -> String + 'a>>,
}

impl<'a> HeatmapGrid<'a, fn(usize, usize) -> HeatmapCell> {
    /// Create a new heatmap grid with specified dimensions
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows: rows.max(1),
            cols: cols.max(1),
            states: None,
            cell_fn: None,
            cell_size: None,
            spacing: None,
            show_labels: false,
            tooltip_fn: None,
        }
    }
}

impl<'a, F> HeatmapGrid<'a, F>
where
    F: Fn(usize, usize) -> HeatmapCell,
{
    /// Set cell states from a flat array (row-major order)
    pub fn data(mut self, states: &'a [CellState]) -> Self {
        self.states = Some(states);
        self
    }

    /// Set cell size in pixels (default: theme.spacing_md + theme.spacing_xs)
    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell_size = Some(size);
        self
    }

    /// Set spacing between cells (default: theme.spacing_xs / 3.0)
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Show labels in cells (requires sufficient cell_size)
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set tooltip generator function
    pub fn on_hover(mut self, f: impl Fn(usize, usize) -> String + 'a) -> Self {
        self.tooltip_fn = Some(Box::new(f));
        self
    }
}

impl<'a> HeatmapGrid<'a, fn(usize, usize) -> HeatmapCell> {
    /// Set cell data provider function (for complex state)
    pub fn cell<F2>(self, f: F2) -> HeatmapGrid<'a, F2>
    where
        F2: Fn(usize, usize) -> HeatmapCell,
    {
        HeatmapGrid {
            rows: self.rows,
            cols: self.cols,
            states: self.states,
            cell_fn: Some(f),
            cell_size: self.cell_size,
            spacing: self.spacing,
            show_labels: self.show_labels,
            tooltip_fn: self.tooltip_fn,
        }
    }
}

impl<'a, F> HeatmapGrid<'a, F>
where
    F: Fn(usize, usize) -> HeatmapCell,
{
    /// Show grid and return clicked cell (row, col), if any
    pub fn show(self, ui: &mut Ui) -> Option<(usize, usize)> {
        self.show_internal(ui)
    }

    /// TEA-style: Show grid, emit Msg when cell is clicked
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_click: impl Fn(usize, usize) -> Msg,
    ) {
        if let Some((row, col)) = self.show_internal(ctx.ui) {
            ctx.emit(on_click(row, col));
        }
    }

    fn show_internal(self, ui: &mut Ui) -> Option<(usize, usize)> {
        let theme = Theme::current(ui.ctx());
        let time = ui.input(|i| i.time) as f32;

        // Get size/spacing from builder or use theme defaults
        let cell_size = self
            .cell_size
            .unwrap_or(theme.spacing_md + theme.spacing_xs);
        let spacing = self.spacing.unwrap_or(theme.spacing_xs / 3.0);

        let total_width =
            self.cols as f32 * cell_size + (self.cols.saturating_sub(1)) as f32 * spacing;
        let total_height =
            self.rows as f32 * cell_size + (self.rows.saturating_sub(1)) as f32 * spacing;

        let (grid_rect, _response) =
            ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::hover());

        if !ui.is_rect_visible(grid_rect) {
            return None;
        }

        let mut clicked: Option<(usize, usize)> = None;
        let mut needs_repaint = false;

        // Render cells
        for row in 0..self.rows {
            for col in 0..self.cols {
                let idx = row * self.cols + col;

                // Get cell data
                let cell_data = if let Some(ref cell_fn) = self.cell_fn {
                    cell_fn(row, col)
                } else if let Some(states) = self.states {
                    if idx < states.len() {
                        HeatmapCell::new(states[idx])
                    } else {
                        HeatmapCell::idle()
                    }
                } else {
                    HeatmapCell::idle()
                };

                // Calculate cell rect
                let x = grid_rect.left() + col as f32 * (cell_size + spacing);
                let y = grid_rect.top() + row as f32 * (cell_size + spacing);
                let cell_rect = egui::Rect::from_min_size(egui::pos2(x, y), Vec2::splat(cell_size));

                // Interact
                let cell_response = ui.interact(cell_rect, ui.id().with(idx), Sense::click());

                if cell_response.clicked() {
                    clicked = Some((row, col));
                }

                // Determine color with flash animation
                let base_color = cell_data.state.color(&theme);
                let color = if cell_data.flash {
                    needs_repaint = true;
                    let flash_intensity = (time * 6.0).sin() * 0.3 + 0.7;
                    Color32::from_rgba_unmultiplied(
                        (base_color.r() as f32 * flash_intensity) as u8,
                        (base_color.g() as f32 * flash_intensity) as u8,
                        (base_color.b() as f32 * flash_intensity) as u8,
                        base_color.a(),
                    )
                } else {
                    base_color
                };

                // Draw cell background
                let rounding = theme.radius_sm * 0.5;
                ui.painter().rect_filled(cell_rect, rounding, color);

                // Draw hover border
                if cell_response.hovered() {
                    ui.painter().rect_stroke(
                        cell_rect,
                        rounding,
                        egui::Stroke::new(2.0, theme.primary),
                        egui::StrokeKind::Inside,
                    );
                }

                // Draw label if enabled and cell is large enough
                if self.show_labels && cell_size >= 20.0 {
                    if let Some(ref label) = cell_data.label {
                        let font_size = (cell_size * 0.4).min(theme.font_size_xs);
                        ui.painter().text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(font_size),
                            theme.text_primary,
                        );
                    }
                }

                // Show tooltip on hover
                if cell_response.hovered() {
                    let tooltip_text = if let Some(ref t) = cell_data.tooltip {
                        t.clone()
                    } else if let Some(ref tooltip_fn) = self.tooltip_fn {
                        tooltip_fn(row, col)
                    } else {
                        format!("[{}, {}]", row, col)
                    };
                    cell_response.with_tooltip(tooltip_text);
                }
            }
        }

        // Request repaint if any cell is flashing
        if needs_repaint {
            ui.ctx().request_repaint();
        }

        clicked
    }
}
