//! Sample Pad - MPC-style sample trigger pad grid
//!
//! A grid of trigger pads for launching samples, commonly seen in MPC, Maschine, and similar hardware.
//!
//! # Example
//! ```ignore
//! SamplePad::new(4, 4)  // 4x4 grid
//!     .labels(&["Kick", "Snare", "HiHat", "Clap", ...])
//!     .active(&[0, 5])  // Currently playing pads
//!     .show_with(ctx, |event| match event {
//!         PadEvent::Trigger(idx) => Msg::TriggerSample(idx),
//!         PadEvent::Select(idx) => Msg::SelectPad(idx),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Events from pad interactions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PadEvent {
    /// Pad triggered (primary click)
    Trigger(usize),
    /// Pad selected (secondary click or shift+click)
    Select(usize),
}

/// A single pad cell data
#[derive(Debug, Clone, Default)]
pub struct PadCell {
    /// Label for the pad
    pub label: String,
    /// Custom color (uses theme.primary if None)
    pub color: Option<Color32>,
    /// Velocity (0.0 - 1.0), affects visual brightness
    pub velocity: f32,
    /// Whether this pad is assigned/has content
    pub assigned: bool,
}

impl PadCell {
    /// Create an empty pad
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a pad with a label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: None,
            velocity: 1.0,
            assigned: true,
        }
    }

    /// Set custom color
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Set velocity (0.0 - 1.0)
    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity.clamp(0.0, 1.0);
        self
    }
}

/// MPC-style sample trigger pad grid
pub struct SamplePad<'a> {
    cols: usize,
    rows: usize,
    pads: Option<&'a [PadCell]>,
    labels: Option<&'a [&'a str]>,
    active: &'a [usize],
    selected: Option<usize>,
    pad_size: f32,
    spacing: f32,
    show_index: bool,
    velocity_sensitive: bool,
}

impl<'a> SamplePad<'a> {
    /// Create a new pad grid
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols: cols.max(1),
            rows: rows.max(1),
            pads: None,
            labels: None,
            active: &[],
            selected: None,
            pad_size: 60.0,
            spacing: 4.0,
            show_index: false,
            velocity_sensitive: false,
        }
    }

    /// Provide pad cell data
    pub fn pads(mut self, pads: &'a [PadCell]) -> Self {
        self.pads = Some(pads);
        self
    }

    /// Simple labels (creates basic PadCells internally)
    pub fn labels(mut self, labels: &'a [&'a str]) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Set currently active (playing) pad indices
    pub fn active(mut self, indices: &'a [usize]) -> Self {
        self.active = indices;
        self
    }

    /// Set selected pad index
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    /// Set pad size
    pub fn pad_size(mut self, size: f32) -> Self {
        self.pad_size = size;
        self
    }

    /// Set spacing between pads
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Show pad index numbers
    pub fn show_index(mut self, show: bool) -> Self {
        self.show_index = show;
        self
    }

    /// Enable velocity sensitivity (visual feedback based on click position)
    pub fn velocity_sensitive(mut self, enabled: bool) -> Self {
        self.velocity_sensitive = enabled;
        self
    }

    /// TEA-style: Show pad grid and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(PadEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show pad grid, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<PadEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<PadEvent> {
        let theme = Theme::current(ui.ctx());
        let time = ui.input(|i| i.time) as f32;
        let mut event = None;

        let total_pads = self.cols * self.rows;
        let total_width = self.cols as f32 * self.pad_size
            + (self.cols.saturating_sub(1)) as f32 * self.spacing;
        let total_height = self.rows as f32 * self.pad_size
            + (self.rows.saturating_sub(1)) as f32 * self.spacing;

        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(total_width, total_height),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // First pass: collect pad info and handle interactions
        struct PadInfo {
            rect: Rect,
            idx: usize,
            is_active: bool,
            is_selected: bool,
            hovered: bool,
            pad_data: PadCell,
            base_color: Color32,
        }

        let mut pads_info: Vec<PadInfo> = Vec::with_capacity(total_pads);

        for idx in 0..total_pads {
            let col = idx % self.cols;
            // MPC-style: bottom-left is pad 1, so invert row order
            let display_row = self.rows - 1 - (idx / self.cols);

            let pad_x = rect.min.x + col as f32 * (self.pad_size + self.spacing);
            let pad_y = rect.min.y + display_row as f32 * (self.pad_size + self.spacing);
            let pad_rect = Rect::from_min_size(
                egui::pos2(pad_x, pad_y),
                Vec2::splat(self.pad_size),
            );

            // Get pad data
            let pad_data = if let Some(pads) = self.pads {
                pads.get(idx).cloned().unwrap_or_default()
            } else if let Some(labels) = self.labels {
                if let Some(label) = labels.get(idx) {
                    PadCell::new(*label)
                } else {
                    PadCell::empty()
                }
            } else {
                PadCell::empty()
            };

            let base_color = pad_data.color.unwrap_or(theme.primary);
            let is_active = self.active.contains(&idx);
            let is_selected = self.selected == Some(idx);

            // Allocate interactive area
            let response = ui.allocate_rect(pad_rect, Sense::click());

            // Handle click events
            if response.clicked() {
                event = Some(PadEvent::Trigger(idx));
            }
            if response.secondary_clicked()
                || (response.clicked() && ui.input(|i| i.modifiers.shift))
            {
                event = Some(PadEvent::Select(idx));
            }

            pads_info.push(PadInfo {
                rect: pad_rect,
                idx,
                is_active,
                is_selected,
                hovered: response.hovered(),
                pad_data,
                base_color,
            });
        }

        // Second pass: draw all pads
        let painter = ui.painter();

        for pad in &pads_info {
            let (bg_color, border_color, text_color) = if pad.is_active {
                // Active/playing - bright with pulse animation
                let pulse = (time * 6.0).sin() * 0.15 + 0.85;
                let pulsed = Color32::from_rgba_unmultiplied(
                    (pad.base_color.r() as f32 * pulse) as u8,
                    (pad.base_color.g() as f32 * pulse) as u8,
                    (pad.base_color.b() as f32 * pulse) as u8,
                    255,
                );
                (pulsed, theme.state_success, theme.primary_text)
            } else if pad.is_selected {
                // Selected - highlighted border
                let dimmed = Color32::from_rgba_unmultiplied(
                    pad.base_color.r(),
                    pad.base_color.g(),
                    pad.base_color.b(),
                    180,
                );
                (dimmed, theme.border_focus, theme.text_primary)
            } else if pad.hovered {
                // Hovered - slightly brighter
                let bright = Color32::from_rgba_unmultiplied(
                    pad.base_color.r(),
                    pad.base_color.g(),
                    pad.base_color.b(),
                    if pad.pad_data.assigned { 200 } else { 100 },
                );
                (bright, theme.border_focus, theme.text_primary)
            } else if pad.pad_data.assigned {
                // Normal assigned pad
                let normal = Color32::from_rgba_unmultiplied(
                    (pad.base_color.r() as f32 * pad.pad_data.velocity) as u8,
                    (pad.base_color.g() as f32 * pad.pad_data.velocity) as u8,
                    (pad.base_color.b() as f32 * pad.pad_data.velocity) as u8,
                    150,
                );
                (normal, theme.border, theme.text_secondary)
            } else {
                // Empty pad
                (theme.bg_secondary, theme.border, theme.text_muted)
            };

            // Draw pad background
            painter.rect_filled(pad.rect, theme.radius_sm, bg_color);

            // Draw border
            let border_width = if pad.is_active || pad.is_selected { 2.0 } else { 1.0 };
            painter.rect_stroke(
                pad.rect,
                theme.radius_sm,
                Stroke::new(border_width, border_color),
                egui::StrokeKind::Inside,
            );

            // Draw label
            if !pad.pad_data.label.is_empty() {
                let text_pos = pad.rect.center();
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    &pad.pad_data.label,
                    egui::FontId::proportional(theme.font_size_xs),
                    text_color,
                );
            }

            // Draw index number
            if self.show_index {
                let idx_pos = pad.rect.left_top() + Vec2::new(4.0, 4.0);
                painter.text(
                    idx_pos,
                    egui::Align2::LEFT_TOP,
                    format!("{}", pad.idx + 1),
                    egui::FontId::proportional(theme.font_size_xs * 0.75),
                    Color32::from_rgba_unmultiplied(
                        text_color.r(),
                        text_color.g(),
                        text_color.b(),
                        120,
                    ),
                );
            }

            // Active indicator (small triangle in corner)
            if pad.is_active {
                let indicator_size = 8.0;
                let center = pad.rect.right_bottom() + Vec2::new(-indicator_size - 2.0, -indicator_size - 2.0);
                let points = vec![
                    egui::pos2(center.x - indicator_size / 2.0, center.y - indicator_size / 2.0),
                    egui::pos2(center.x - indicator_size / 2.0, center.y + indicator_size / 2.0),
                    egui::pos2(center.x + indicator_size / 2.0, center.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    theme.state_success,
                    Stroke::NONE,
                ));
            }
        }

        // Request repaint if we have active pads (for animation)
        if !self.active.is_empty() {
            ui.ctx().request_repaint();
        }

        event
    }
}
