//! GradientEditor atom - Gradient color stop editor for VJ applications
//!
//! A component for creating and editing color gradients with draggable stops.
//! Used for color mapping, LUTs, and visual effects in VJ software.
//!
//! # Features
//! - Add/remove color stops
//! - Drag stops to reposition
//! - Double-click to edit stop color
//! - Gradient preview
//! - Linear/Radial mode indicators
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! GradientEditor::new(&gradient)
//!     .show_with(ctx, |event| match event {
//!         GradientEvent::AddStop(pos, color) => Msg::AddStop(pos, color),
//!         GradientEvent::MoveStop(idx, pos) => Msg::MoveStop(idx, pos),
//!         GradientEvent::RemoveStop(idx) => Msg::RemoveStop(idx),
//!         GradientEvent::SetStopColor(idx, color) => Msg::SetColor(idx, color),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// A color stop in the gradient
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color32,
}

impl GradientStop {
    pub fn new(position: f32, color: Color32) -> Self {
        Self {
            position: position.clamp(0.0, 1.0),
            color,
        }
    }
}

/// Gradient data
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
}

impl Gradient {
    pub fn new() -> Self {
        Self {
            stops: vec![
                GradientStop::new(0.0, Color32::BLACK),
                GradientStop::new(1.0, Color32::WHITE),
            ],
        }
    }

    pub fn from_stops(mut stops: Vec<GradientStop>) -> Self {
        stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        Self { stops }
    }

    pub fn sample(&self, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);

        if self.stops.is_empty() {
            return Color32::BLACK;
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        let mut left = &self.stops[0];
        let mut right = &self.stops[self.stops.len() - 1];

        for i in 0..self.stops.len() - 1 {
            if self.stops[i].position <= t && self.stops[i + 1].position >= t {
                left = &self.stops[i];
                right = &self.stops[i + 1];
                break;
            }
        }

        let range = right.position - left.position;
        if range < 0.0001 {
            return left.color;
        }

        let factor = (t - left.position) / range;
        Color32::from_rgba_unmultiplied(
            lerp_u8(left.color.r(), right.color.r(), factor),
            lerp_u8(left.color.g(), right.color.g(), factor),
            lerp_u8(left.color.b(), right.color.b(), factor),
            lerp_u8(left.color.a(), right.color.a(), factor),
        )
    }

    pub fn add_stop(&mut self, position: f32) {
        let color = self.sample(position);
        self.stops.push(GradientStop::new(position, color));
        self.stops
            .sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    }

    pub fn add_stop_with_color(&mut self, position: f32, color: Color32) {
        self.stops.push(GradientStop::new(position, color));
        self.stops
            .sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    }

    pub fn remove_stop(&mut self, index: usize) {
        if self.stops.len() > 2 && index < self.stops.len() {
            self.stops.remove(index);
        }
    }

    pub fn move_stop(&mut self, index: usize, new_position: f32) {
        if let Some(stop) = self.stops.get_mut(index) {
            stop.position = new_position.clamp(0.0, 1.0);
        }
        self.stops
            .sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self::new()
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}

/// Events emitted by GradientEditor
#[derive(Debug, Clone)]
pub enum GradientEvent {
    AddStop(f32),
    MoveStop { index: usize, position: f32 },
    RemoveStop(usize),
    SetStopColor { index: usize, color: Color32 },
    SelectStop(Option<usize>),
}

/// Gradient direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientDirection {
    #[default]
    Horizontal,
    Vertical,
}

/// Gradient editor widget
pub struct GradientEditor<'a> {
    gradient: &'a Gradient,
    width: f32,
    height: f32,
    direction: GradientDirection,
    selected_stop: Option<usize>,
    show_stop_values: bool,
    editable: bool,
}

impl<'a> GradientEditor<'a> {
    pub fn new(gradient: &'a Gradient) -> Self {
        Self {
            gradient,
            width: 300.0,
            height: 40.0,
            direction: GradientDirection::Horizontal,
            selected_stop: None,
            show_stop_values: true,
            editable: true,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn direction(mut self, direction: GradientDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected_stop = index;
        self
    }

    pub fn show_values(mut self, show: bool) -> Self {
        self.show_stop_values = show;
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(GradientEvent) -> Msg,
    ) {
        let event = self.show_internal(ctx.ui);
        if let Some(e) = event {
            ctx.emit(on_event(e));
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<GradientEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<GradientEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<GradientEvent> = None;

        let stop_handle_size = theme.spacing_md;
        let stop_area_height = stop_handle_size + theme.spacing_xs;
        let values_height = if self.show_stop_values {
            theme.font_size_xs + theme.spacing_xs
        } else {
            0.0
        };

        let total_height = self.height + stop_area_height + values_height + theme.spacing_xs;

        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(self.width, total_height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Gradient bar area
        let bar_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, rect.min.y),
            Vec2::new(self.width, self.height),
        );

        // First pass: collect interactions
        let bar_response = if self.editable {
            let resp = ui.allocate_rect(bar_rect, Sense::click());
            Some((resp.double_clicked(), resp.interact_pointer_pos()))
        } else {
            None
        };

        // Stop handle interactions
        struct StopInfo {
            idx: usize,
            stop_x: f32,
            hovered: bool,
            dragged: bool,
            drag_pos: Option<Pos2>,
            clicked: bool,
            secondary_clicked: bool,
        }

        let stop_y = bar_rect.max.y + theme.spacing_xs;
        let mut stop_infos: Vec<StopInfo> = Vec::with_capacity(self.gradient.stops.len());

        for (idx, stop) in self.gradient.stops.iter().enumerate() {
            let stop_x = match self.direction {
                GradientDirection::Horizontal => bar_rect.min.x + stop.position * bar_rect.width(),
                GradientDirection::Vertical => bar_rect.center().x,
            };

            let handle_pos = Pos2::new(stop_x, stop_y);
            let handle_rect =
                Rect::from_center_size(handle_pos, Vec2::new(stop_handle_size, stop_handle_size));

            if self.editable {
                let resp = ui.allocate_rect(handle_rect.expand(4.0), Sense::click_and_drag());
                stop_infos.push(StopInfo {
                    idx,
                    stop_x,
                    hovered: resp.hovered(),
                    dragged: resp.dragged(),
                    drag_pos: resp.interact_pointer_pos(),
                    clicked: resp.clicked(),
                    secondary_clicked: resp.secondary_clicked(),
                });
            } else {
                stop_infos.push(StopInfo {
                    idx,
                    stop_x,
                    hovered: false,
                    dragged: false,
                    drag_pos: None,
                    clicked: false,
                    secondary_clicked: false,
                });
            }
        }

        // Second pass: draw everything
        let painter = ui.painter();

        // Draw checkerboard for transparency
        let checker_size = 8.0;
        let cols = (bar_rect.width() / checker_size) as usize + 1;
        let rows = (bar_rect.height() / checker_size) as usize + 1;
        for row in 0..rows {
            for col in 0..cols {
                let is_dark = (row + col) % 2 == 0;
                let color = if is_dark {
                    Color32::from_gray(60)
                } else {
                    Color32::from_gray(100)
                };
                let check_rect = Rect::from_min_size(
                    Pos2::new(
                        bar_rect.min.x + col as f32 * checker_size,
                        bar_rect.min.y + row as f32 * checker_size,
                    ),
                    Vec2::splat(checker_size),
                )
                .intersect(bar_rect);
                painter.rect_filled(check_rect, 0.0, color);
            }
        }

        // Draw gradient
        let steps = 64;
        for i in 0..steps {
            let t1 = i as f32 / steps as f32;
            let t2 = (i + 1) as f32 / steps as f32;

            let (x1, x2) = match self.direction {
                GradientDirection::Horizontal => (
                    bar_rect.min.x + t1 * bar_rect.width(),
                    bar_rect.min.x + t2 * bar_rect.width(),
                ),
                GradientDirection::Vertical => (bar_rect.min.x, bar_rect.max.x),
            };

            let (y1, y2) = match self.direction {
                GradientDirection::Horizontal => (bar_rect.min.y, bar_rect.max.y),
                GradientDirection::Vertical => (
                    bar_rect.min.y + t1 * bar_rect.height(),
                    bar_rect.min.y + t2 * bar_rect.height(),
                ),
            };

            let color = self.gradient.sample(t1);
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(x1, y1), Pos2::new(x2 + 1.0, y2)),
                0.0,
                color,
            );
        }

        // Border
        painter.rect_stroke(
            bar_rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        // Handle bar click to add stop
        if let Some((double_clicked, pos)) = bar_response {
            if double_clicked {
                if let Some(pos) = pos {
                    let t = match self.direction {
                        GradientDirection::Horizontal => {
                            (pos.x - bar_rect.min.x) / bar_rect.width()
                        }
                        GradientDirection::Vertical => (pos.y - bar_rect.min.y) / bar_rect.height(),
                    };
                    event = Some(GradientEvent::AddStop(t.clamp(0.0, 1.0)));
                }
            }
        }

        // Draw stop handles
        for (info, stop) in stop_infos.iter().zip(self.gradient.stops.iter()) {
            let is_selected = self.selected_stop == Some(info.idx);

            // Draw triangle pointing up
            let tri_height = stop_handle_size * 0.6;
            let tri_half_width = stop_handle_size * 0.5;
            let tri_top = Pos2::new(info.stop_x, bar_rect.max.y);
            let tri_left = Pos2::new(info.stop_x - tri_half_width, bar_rect.max.y + tri_height);
            let tri_right = Pos2::new(info.stop_x + tri_half_width, bar_rect.max.y + tri_height);

            let is_hovered = info.hovered || info.dragged;

            painter.add(egui::Shape::convex_polygon(
                vec![tri_top, tri_left, tri_right],
                stop.color,
                Stroke::NONE,
            ));

            let outline_color = if is_selected {
                theme.primary
            } else if is_hovered {
                theme.text_primary
            } else {
                theme.border
            };
            painter.add(egui::Shape::closed_line(
                vec![tri_top, tri_left, tri_right],
                Stroke::new(if is_selected { 2.0 } else { 1.0 }, outline_color),
            ));

            // Color swatch below triangle
            let swatch_rect = Rect::from_min_size(
                Pos2::new(info.stop_x - stop_handle_size / 2.0, tri_right.y + 2.0),
                Vec2::new(stop_handle_size, stop_handle_size / 2.0),
            );
            painter.rect_filled(swatch_rect, theme.radius_sm, stop.color);
            painter.rect_stroke(
                swatch_rect,
                theme.radius_sm,
                Stroke::new(1.0, outline_color),
                egui::StrokeKind::Outside,
            );

            // Position value on hover
            if self.show_stop_values && is_hovered {
                let value_text = format!("{:.0}%", stop.position * 100.0);
                painter.text(
                    Pos2::new(info.stop_x, swatch_rect.max.y + theme.spacing_xs),
                    egui::Align2::CENTER_TOP,
                    &value_text,
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_muted,
                );
            }
        }

        // Connection lines between stops
        if self.gradient.stops.len() > 1 {
            let line_y = stop_y + stop_handle_size * 0.3;
            for i in 0..self.gradient.stops.len() - 1 {
                let x1 = bar_rect.min.x + self.gradient.stops[i].position * bar_rect.width();
                let x2 = bar_rect.min.x + self.gradient.stops[i + 1].position * bar_rect.width();
                painter.line_segment(
                    [
                        Pos2::new(x1 + stop_handle_size / 2.0, line_y),
                        Pos2::new(x2 - stop_handle_size / 2.0, line_y),
                    ],
                    Stroke::new(1.0, theme.border),
                );
            }
        }

        // Info text
        let info_text = format!("{} stops", self.gradient.stops.len());
        painter.text(
            Pos2::new(rect.max.x - theme.spacing_sm, rect.min.y + theme.spacing_xs),
            egui::Align2::RIGHT_TOP,
            &info_text,
            egui::FontId::proportional(theme.font_size_xs),
            theme.text_muted,
        );

        // Process stop handle events
        for info in stop_infos.iter() {
            if event.is_some() {
                break;
            }

            if info.clicked {
                event = Some(GradientEvent::SelectStop(Some(info.idx)));
            } else if info.dragged {
                if let Some(pos) = info.drag_pos {
                    let new_pos = match self.direction {
                        GradientDirection::Horizontal => {
                            (pos.x - bar_rect.min.x) / bar_rect.width()
                        }
                        GradientDirection::Vertical => (pos.y - bar_rect.min.y) / bar_rect.height(),
                    };
                    event = Some(GradientEvent::MoveStop {
                        index: info.idx,
                        position: new_pos.clamp(0.0, 1.0),
                    });
                }
            } else if info.secondary_clicked && self.gradient.stops.len() > 2 {
                event = Some(GradientEvent::RemoveStop(info.idx));
            }
        }

        event
    }
}
