//! Clip Grid atom - Ableton Live style clip launcher grid
//!
//! A grid for launching clips/phrases with visual feedback for
//! current, queued, and idle states.
//!
//! # Example
//! ```ignore
//! ClipGrid::new(&clips, 4) // 4 columns
//!     .current(model.current_clip)
//!     .queued(&model.queue)
//!     .show_with(ctx, |idx| Msg::QueueClip(idx));
//! ```

use crate::Theme;
use egui::{Color32, Sense, Ui, Vec2};
use egui_cha::ViewCtx;

/// Clip state for visual representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipState {
    /// Idle / not playing
    #[default]
    Idle,
    /// Queued to play next
    Queued,
    /// Currently playing
    Playing,
    /// Selected (UI focus)
    Selected,
}

/// A single clip/phrase cell data
#[derive(Debug, Clone)]
pub struct ClipCell {
    pub name: String,
    pub color: Option<Color32>,
    pub state: ClipState,
}

impl ClipCell {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: None,
            state: ClipState::Idle,
        }
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_state(mut self, state: ClipState) -> Self {
        self.state = state;
        self
    }
}

/// Ableton Live style clip launcher grid
pub struct ClipGrid<'a> {
    clips: &'a [ClipCell],
    columns: usize,
    cell_size: Vec2,
    spacing: f32,
    current: Option<usize>,
    queued: &'a [usize],
    show_index: bool,
}

impl<'a> ClipGrid<'a> {
    /// Create a new clip grid
    pub fn new(clips: &'a [ClipCell], columns: usize) -> Self {
        Self {
            clips,
            columns: columns.max(1),
            cell_size: Vec2::new(80.0, 60.0),
            spacing: 4.0,
            current: None,
            queued: &[],
            show_index: false,
        }
    }

    /// Set cell size
    pub fn cell_size(mut self, width: f32, height: f32) -> Self {
        self.cell_size = Vec2::new(width, height);
        self
    }

    /// Set spacing between cells
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the currently playing clip index
    pub fn current(mut self, index: Option<usize>) -> Self {
        self.current = index;
        self
    }

    /// Set queued clip indices
    pub fn queued(mut self, indices: &'a [usize]) -> Self {
        self.queued = indices;
        self
    }

    /// Show clip index numbers
    pub fn show_index(mut self, show: bool) -> Self {
        self.show_index = show;
        self
    }

    /// TEA-style: Show grid, emit Msg when clip is clicked
    pub fn show_with<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, on_click: impl Fn(usize) -> Msg) {
        let clicked = self.show_internal(ctx.ui);
        if let Some(idx) = clicked {
            ctx.emit(on_click(idx));
        }
    }

    /// Show grid and return clicked clip index
    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<usize> {
        let theme = Theme::current(ui.ctx());
        let time = ui.input(|i| i.time) as f32;
        let mut clicked_idx: Option<usize> = None;

        let rows = (self.clips.len() + self.columns - 1) / self.columns;
        let total_width =
            self.columns as f32 * self.cell_size.x + (self.columns - 1) as f32 * self.spacing;
        let total_height =
            rows as f32 * self.cell_size.y + (rows.saturating_sub(1)) as f32 * self.spacing;

        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // First pass: collect cell info and handle interactions
        struct CellInfo {
            rect: egui::Rect,
            state: ClipState,
            hovered: bool,
            name: String,
            base_color: Color32,
            idx: usize,
        }

        let mut cells: Vec<CellInfo> = Vec::with_capacity(self.clips.len());

        for (idx, clip) in self.clips.iter().enumerate() {
            let col = idx % self.columns;
            let row = idx / self.columns;

            let cell_x = rect.min.x + col as f32 * (self.cell_size.x + self.spacing);
            let cell_y = rect.min.y + row as f32 * (self.cell_size.y + self.spacing);
            let cell_rect = egui::Rect::from_min_size(egui::pos2(cell_x, cell_y), self.cell_size);

            // Determine state (priority: current > queued > clip.state)
            let state = if self.current == Some(idx) {
                ClipState::Playing
            } else if self.queued.contains(&idx) {
                ClipState::Queued
            } else {
                clip.state
            };

            // Allocate interactive area for this cell
            let cell_response = ui.allocate_rect(cell_rect, Sense::click());

            if cell_response.clicked() {
                clicked_idx = Some(idx);
            }

            cells.push(CellInfo {
                rect: cell_rect,
                state,
                hovered: cell_response.hovered(),
                name: clip.name.clone(),
                base_color: clip.color.unwrap_or(theme.primary),
                idx,
            });
        }

        // Second pass: draw all cells
        let painter = ui.painter();

        for cell in &cells {
            // Colors based on state
            let (bg_color, border_color, text_color) = match cell.state {
                ClipState::Playing => {
                    // Pulsing animation for playing
                    let pulse = (time * 4.0).sin() * 0.15 + 0.85;
                    let pulsed = Color32::from_rgba_unmultiplied(
                        (cell.base_color.r() as f32 * pulse) as u8,
                        (cell.base_color.g() as f32 * pulse) as u8,
                        (cell.base_color.b() as f32 * pulse) as u8,
                        255,
                    );
                    (pulsed, theme.state_success, theme.primary_text)
                }
                ClipState::Queued => {
                    let dimmed = Color32::from_rgba_unmultiplied(
                        cell.base_color.r(),
                        cell.base_color.g(),
                        cell.base_color.b(),
                        180,
                    );
                    (dimmed, theme.state_warning, theme.text_primary)
                }
                ClipState::Selected => (cell.base_color, theme.border_focus, theme.primary_text),
                ClipState::Idle => {
                    let idle_bg = if cell.hovered {
                        Color32::from_rgba_unmultiplied(
                            cell.base_color.r(),
                            cell.base_color.g(),
                            cell.base_color.b(),
                            100,
                        )
                    } else {
                        theme.bg_secondary
                    };
                    (idle_bg, theme.border, theme.text_secondary)
                }
            };

            // Draw cell background
            painter.rect_filled(cell.rect, theme.radius_sm, bg_color);

            // Draw border
            let border_width = if matches!(cell.state, ClipState::Playing | ClipState::Queued) {
                2.0
            } else {
                theme.border_width
            };
            painter.rect_stroke(
                cell.rect,
                theme.radius_sm,
                egui::Stroke::new(border_width, border_color),
                egui::StrokeKind::Inside,
            );

            // Draw clip name
            let text_pos = cell.rect.center();
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                &cell.name,
                egui::FontId::proportional(theme.font_size_xs),
                text_color,
            );

            // Draw index if enabled
            if self.show_index {
                let idx_pos = cell.rect.left_top() + Vec2::new(4.0, 4.0);
                painter.text(
                    idx_pos,
                    egui::Align2::LEFT_TOP,
                    format!("{}", cell.idx + 1),
                    egui::FontId::proportional(theme.font_size_xs * 0.8),
                    Color32::from_rgba_unmultiplied(
                        text_color.r(),
                        text_color.g(),
                        text_color.b(),
                        150,
                    ),
                );
            }

            // Draw playing indicator (triangle)
            if matches!(cell.state, ClipState::Playing) {
                let indicator_size = 8.0;
                let center =
                    cell.rect.right_top() + Vec2::new(-indicator_size - 4.0, indicator_size + 4.0);
                let points = vec![
                    egui::pos2(
                        center.x - indicator_size / 2.0,
                        center.y - indicator_size / 2.0,
                    ),
                    egui::pos2(
                        center.x - indicator_size / 2.0,
                        center.y + indicator_size / 2.0,
                    ),
                    egui::pos2(center.x + indicator_size / 2.0, center.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    theme.state_success,
                    egui::Stroke::NONE,
                ));
            }

            // Draw queued indicator (circle)
            if matches!(cell.state, ClipState::Queued) {
                let indicator_pos = cell.rect.right_top() + Vec2::new(-8.0, 8.0);
                painter.circle_filled(indicator_pos, 4.0, theme.state_warning);
            }
        }

        // Request repaint if we have a playing clip (for animation)
        if self.current.is_some() {
            ui.ctx().request_repaint();
        }

        clicked_idx
    }
}
