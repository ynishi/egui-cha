//! MaskEditor atom - Mask/stencil shape editor for VJ applications
//!
//! A component for creating and editing mask shapes (rectangle, ellipse, polygon).
//! Used for layer masking, spotlight effects, and region selection in VJ software.
//!
//! # Features
//! - Multiple mask types (rectangle, ellipse, polygon, freehand)
//! - Point editing (add, move, delete)
//! - Feather/blur control
//! - Invert mask option
//! - Visual preview
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! MaskEditor::new(&mask)
//!     .show_with(ctx, |event| match event {
//!         MaskEvent::MovePoint(idx, pos) => Msg::MovePoint(idx, pos),
//!         MaskEvent::AddPoint(pos) => Msg::AddPoint(pos),
//!         MaskEvent::DeletePoint(idx) => Msg::DeletePoint(idx),
//!         MaskEvent::SetFeather(val) => Msg::SetFeather(val),
//!         MaskEvent::ToggleInvert => Msg::ToggleInvert,
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Mask shape type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaskShape {
    /// Rectangular mask
    #[default]
    Rectangle,
    /// Elliptical mask
    Ellipse,
    /// Polygon mask (custom points)
    Polygon,
    /// Freehand drawn mask
    Freehand,
}

impl MaskShape {
    pub fn label(&self) -> &'static str {
        match self {
            MaskShape::Rectangle => "Rect",
            MaskShape::Ellipse => "Ellipse",
            MaskShape::Polygon => "Poly",
            MaskShape::Freehand => "Free",
        }
    }

    pub fn all() -> &'static [MaskShape] {
        &[
            MaskShape::Rectangle,
            MaskShape::Ellipse,
            MaskShape::Polygon,
            MaskShape::Freehand,
        ]
    }
}

/// A point in the mask (normalized 0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaskPoint {
    /// X position (0.0-1.0)
    pub x: f32,
    /// Y position (0.0-1.0)
    pub y: f32,
    /// Curve tension (for smooth corners)
    pub tension: f32,
}

impl MaskPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            tension: 0.0,
        }
    }

    pub fn with_tension(mut self, tension: f32) -> Self {
        self.tension = tension.clamp(0.0, 1.0);
        self
    }

    pub fn to_pos(&self, rect: Rect) -> Pos2 {
        Pos2::new(
            rect.min.x + self.x * rect.width(),
            rect.min.y + self.y * rect.height(),
        )
    }

    pub fn from_pos(pos: Pos2, rect: Rect) -> Self {
        Self::new(
            (pos.x - rect.min.x) / rect.width(),
            (pos.y - rect.min.y) / rect.height(),
        )
    }
}

/// Mask data
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    /// Shape type
    pub shape: MaskShape,
    /// Points defining the mask
    pub points: Vec<MaskPoint>,
    /// Feather/blur amount (0.0-1.0)
    pub feather: f32,
    /// Invert mask
    pub inverted: bool,
    /// Mask opacity
    pub opacity: f32,
    /// Mask color (for preview)
    pub color: Color32,
}

impl Mask {
    /// Create a new rectangular mask
    pub fn rectangle() -> Self {
        Self {
            shape: MaskShape::Rectangle,
            points: vec![MaskPoint::new(0.2, 0.2), MaskPoint::new(0.8, 0.8)],
            feather: 0.0,
            inverted: false,
            opacity: 1.0,
            color: Color32::WHITE,
        }
    }

    /// Create a new elliptical mask
    pub fn ellipse() -> Self {
        Self {
            shape: MaskShape::Ellipse,
            points: vec![
                MaskPoint::new(0.5, 0.5), // center
                MaskPoint::new(0.3, 0.3), // radius (stored as point for editing)
            ],
            feather: 0.0,
            inverted: false,
            opacity: 1.0,
            color: Color32::WHITE,
        }
    }

    /// Create a new polygon mask
    pub fn polygon(points: Vec<MaskPoint>) -> Self {
        Self {
            shape: MaskShape::Polygon,
            points,
            feather: 0.0,
            inverted: false,
            opacity: 1.0,
            color: Color32::WHITE,
        }
    }

    /// Create a default triangle polygon
    pub fn triangle() -> Self {
        Self::polygon(vec![
            MaskPoint::new(0.5, 0.2),
            MaskPoint::new(0.2, 0.8),
            MaskPoint::new(0.8, 0.8),
        ])
    }

    pub fn with_feather(mut self, feather: f32) -> Self {
        self.feather = feather.clamp(0.0, 1.0);
        self
    }

    pub fn with_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
}

impl Default for Mask {
    fn default() -> Self {
        Self::rectangle()
    }
}

/// Events emitted by MaskEditor
#[derive(Debug, Clone)]
pub enum MaskEvent {
    /// Point moved to new position
    MovePoint { index: usize, position: MaskPoint },
    /// Point added
    AddPoint(MaskPoint),
    /// Point deleted
    DeletePoint(usize),
    /// Point selected
    SelectPoint(Option<usize>),
    /// Feather changed
    SetFeather(f32),
    /// Opacity changed
    SetOpacity(f32),
    /// Invert toggled
    ToggleInvert,
    /// Shape type changed
    SetShape(MaskShape),
    /// Reset to default
    Reset,
}

/// Mask editor widget
pub struct MaskEditor<'a> {
    mask: &'a Mask,
    size: Vec2,
    selected_point: Option<usize>,
    show_controls: bool,
    show_grid: bool,
    editable: bool,
}

impl<'a> MaskEditor<'a> {
    pub fn new(mask: &'a Mask) -> Self {
        Self {
            mask,
            size: Vec2::new(300.0, 200.0),
            selected_point: None,
            show_controls: true,
            show_grid: true,
            editable: true,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn selected_point(mut self, index: Option<usize>) -> Self {
        self.selected_point = index;
        self
    }

    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn show_with<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, on_event: impl Fn(MaskEvent) -> Msg) {
        let event = self.show_internal(ctx.ui);
        if let Some(e) = event {
            ctx.emit(on_event(e));
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<MaskEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<MaskEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<MaskEvent> = None;

        let controls_height = if self.show_controls {
            theme.spacing_xl + theme.spacing_sm
        } else {
            0.0
        };
        let total_height = self.size.y + controls_height;

        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(self.size.x, total_height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Canvas area
        let canvas_rect = Rect::from_min_size(rect.min, self.size);

        // First pass: collect interactions
        let canvas_response = ui.allocate_rect(canvas_rect, Sense::click());
        let canvas_double_clicked = canvas_response.double_clicked();
        let canvas_click_pos = canvas_response.interact_pointer_pos();

        // Point handle interactions
        struct PointInfo {
            index: usize,
            screen_pos: Pos2,
            hovered: bool,
            dragged: bool,
            drag_pos: Option<Pos2>,
            clicked: bool,
            secondary_clicked: bool,
        }

        let handle_size = theme.spacing_sm;
        let mut point_infos: Vec<PointInfo> = Vec::new();

        if self.editable {
            for (idx, point) in self.mask.points.iter().enumerate() {
                let screen_pos = point.to_pos(canvas_rect);
                let handle_rect =
                    Rect::from_center_size(screen_pos, Vec2::splat(handle_size * 2.0));
                let resp = ui.allocate_rect(handle_rect, Sense::click_and_drag());

                point_infos.push(PointInfo {
                    index: idx,
                    screen_pos,
                    hovered: resp.hovered(),
                    dragged: resp.dragged(),
                    drag_pos: resp.interact_pointer_pos(),
                    clicked: resp.clicked(),
                    secondary_clicked: resp.secondary_clicked(),
                });
            }
        }

        // Controls interactions
        let mut shape_clicked: Option<MaskShape> = None;
        let mut invert_clicked = false;
        let mut feather_drag: Option<f32> = None;

        if self.show_controls {
            let controls_y = canvas_rect.max.y + theme.spacing_xs;

            // Shape buttons
            let button_width = 45.0;
            let mut x = rect.min.x;

            for shape in MaskShape::all() {
                let btn_rect = Rect::from_min_size(
                    Pos2::new(x, controls_y),
                    Vec2::new(button_width, theme.spacing_lg),
                );
                let btn_resp = ui.allocate_rect(btn_rect, Sense::click());
                if btn_resp.clicked() {
                    shape_clicked = Some(*shape);
                }
                x += button_width + theme.spacing_xs;
            }

            // Invert toggle
            let invert_rect = Rect::from_min_size(
                Pos2::new(x + theme.spacing_sm, controls_y),
                Vec2::new(40.0, theme.spacing_lg),
            );
            let invert_resp = ui.allocate_rect(invert_rect, Sense::click());
            if invert_resp.clicked() {
                invert_clicked = true;
            }

            // Feather slider
            let feather_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 80.0, controls_y),
                Vec2::new(70.0, theme.spacing_lg),
            );
            let feather_resp = ui.allocate_rect(feather_rect, Sense::click_and_drag());
            if feather_resp.dragged() || feather_resp.clicked() {
                if let Some(pos) = feather_resp.interact_pointer_pos() {
                    let bar_rect = Rect::from_min_size(
                        Pos2::new(feather_rect.min.x, feather_rect.center().y - 3.0),
                        Vec2::new(feather_rect.width(), 6.0),
                    );
                    feather_drag =
                        Some(((pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0));
                }
            }
        }

        // Second pass: draw everything
        let painter = ui.painter();

        // Background
        painter.rect_filled(canvas_rect, theme.radius_sm, theme.bg_secondary);

        // Grid
        if self.show_grid {
            let grid_stroke = Stroke::new(0.5, theme.border.gamma_multiply(0.5));
            let divisions = 4;

            for i in 1..divisions {
                let t = i as f32 / divisions as f32;
                // Vertical
                let x = canvas_rect.min.x + t * canvas_rect.width();
                painter.line_segment(
                    [
                        Pos2::new(x, canvas_rect.min.y),
                        Pos2::new(x, canvas_rect.max.y),
                    ],
                    grid_stroke,
                );
                // Horizontal
                let y = canvas_rect.min.y + t * canvas_rect.height();
                painter.line_segment(
                    [
                        Pos2::new(canvas_rect.min.x, y),
                        Pos2::new(canvas_rect.max.x, y),
                    ],
                    grid_stroke,
                );
            }

            // Center cross
            let center = canvas_rect.center();
            painter.line_segment(
                [
                    Pos2::new(center.x, canvas_rect.min.y),
                    Pos2::new(center.x, canvas_rect.max.y),
                ],
                Stroke::new(0.5, theme.border),
            );
            painter.line_segment(
                [
                    Pos2::new(canvas_rect.min.x, center.y),
                    Pos2::new(canvas_rect.max.x, center.y),
                ],
                Stroke::new(0.5, theme.border),
            );
        }

        // Draw mask shape preview
        let mask_color = if self.mask.inverted {
            Color32::from_rgba_unmultiplied(255, 100, 100, 100)
        } else {
            Color32::from_rgba_unmultiplied(100, 200, 255, 100)
        };
        let mask_stroke = Stroke::new(2.0, theme.primary);

        match self.mask.shape {
            MaskShape::Rectangle => {
                if self.mask.points.len() >= 2 {
                    let p1 = self.mask.points[0].to_pos(canvas_rect);
                    let p2 = self.mask.points[1].to_pos(canvas_rect);
                    let shape_rect = Rect::from_two_pos(p1, p2);
                    painter.rect_filled(shape_rect, 0.0, mask_color);
                    painter.rect_stroke(shape_rect, 0.0, mask_stroke, egui::StrokeKind::Outside);
                }
            }
            MaskShape::Ellipse => {
                if self.mask.points.len() >= 2 {
                    let center = self.mask.points[0].to_pos(canvas_rect);
                    let radius_pt = self.mask.points[1];
                    let radius = Vec2::new(
                        radius_pt.x * canvas_rect.width(),
                        radius_pt.y * canvas_rect.height(),
                    );

                    // Draw ellipse using segments
                    let segments = 32;
                    let mut points = Vec::with_capacity(segments);
                    for i in 0..segments {
                        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                        points.push(Pos2::new(
                            center.x + angle.cos() * radius.x,
                            center.y + angle.sin() * radius.y,
                        ));
                    }
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        mask_color,
                        Stroke::NONE,
                    ));
                    painter.add(egui::Shape::closed_line(points, mask_stroke));
                }
            }
            MaskShape::Polygon | MaskShape::Freehand => {
                if self.mask.points.len() >= 3 {
                    let points: Vec<Pos2> = self
                        .mask
                        .points
                        .iter()
                        .map(|p| p.to_pos(canvas_rect))
                        .collect();
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        mask_color,
                        Stroke::NONE,
                    ));
                    painter.add(egui::Shape::closed_line(points, mask_stroke));
                } else if self.mask.points.len() == 2 {
                    let p1 = self.mask.points[0].to_pos(canvas_rect);
                    let p2 = self.mask.points[1].to_pos(canvas_rect);
                    painter.line_segment([p1, p2], mask_stroke);
                }
            }
        }

        // Draw feather indicator
        if self.mask.feather > 0.01 {
            let feather_text = format!("Feather: {:.0}%", self.mask.feather * 100.0);
            painter.text(
                Pos2::new(
                    canvas_rect.min.x + theme.spacing_xs,
                    canvas_rect.min.y + theme.spacing_xs,
                ),
                egui::Align2::LEFT_TOP,
                &feather_text,
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );
        }

        // Draw point handles
        for info in point_infos.iter() {
            let is_selected = self.selected_point == Some(info.index);
            let is_active = info.hovered || info.dragged || is_selected;

            let fill = if is_selected {
                theme.primary
            } else if is_active {
                theme.primary.gamma_multiply(0.8)
            } else {
                theme.bg_tertiary
            };

            painter.circle_filled(info.screen_pos, handle_size, fill);
            painter.circle_stroke(
                info.screen_pos,
                handle_size,
                Stroke::new(if is_selected { 2.0 } else { 1.0 }, theme.primary),
            );

            // Point index label
            painter.text(
                info.screen_pos + Vec2::new(handle_size + 2.0, -handle_size),
                egui::Align2::LEFT_BOTTOM,
                format!("{}", info.index + 1),
                egui::FontId::proportional(theme.font_size_xs * 0.8),
                theme.text_muted,
            );
        }

        // Border
        painter.rect_stroke(
            canvas_rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        // Draw controls
        if self.show_controls {
            let controls_y = canvas_rect.max.y + theme.spacing_xs;
            let button_width = 45.0;
            let mut x = rect.min.x;

            for shape in MaskShape::all() {
                let btn_rect = Rect::from_min_size(
                    Pos2::new(x, controls_y),
                    Vec2::new(button_width, theme.spacing_lg),
                );
                let is_active = self.mask.shape == *shape;
                let bg = if is_active {
                    theme.primary
                } else {
                    theme.bg_tertiary
                };
                let text_color = if is_active {
                    theme.primary_text
                } else {
                    theme.text_secondary
                };

                painter.rect_filled(btn_rect, theme.radius_sm, bg);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    shape.label(),
                    egui::FontId::proportional(theme.font_size_xs),
                    text_color,
                );
                x += button_width + theme.spacing_xs;
            }

            // Invert toggle
            let invert_rect = Rect::from_min_size(
                Pos2::new(x + theme.spacing_sm, controls_y),
                Vec2::new(40.0, theme.spacing_lg),
            );
            let invert_bg = if self.mask.inverted {
                theme.state_warning
            } else {
                theme.bg_tertiary
            };
            painter.rect_filled(invert_rect, theme.radius_sm, invert_bg);
            painter.text(
                invert_rect.center(),
                egui::Align2::CENTER_CENTER,
                "INV",
                egui::FontId::proportional(theme.font_size_xs),
                if self.mask.inverted {
                    theme.primary_text
                } else {
                    theme.text_secondary
                },
            );

            // Feather slider
            let feather_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 80.0, controls_y),
                Vec2::new(70.0, theme.spacing_lg),
            );
            painter.text(
                Pos2::new(
                    feather_rect.min.x - theme.spacing_xs,
                    feather_rect.center().y,
                ),
                egui::Align2::RIGHT_CENTER,
                "F",
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );

            let bar_rect = Rect::from_min_size(
                Pos2::new(feather_rect.min.x, feather_rect.center().y - 3.0),
                Vec2::new(feather_rect.width(), 6.0),
            );
            painter.rect_filled(bar_rect, 3.0, theme.bg_tertiary);
            let fill_rect = Rect::from_min_size(
                bar_rect.min,
                Vec2::new(bar_rect.width() * self.mask.feather, bar_rect.height()),
            );
            painter.rect_filled(fill_rect, 3.0, theme.primary);

            let handle_x = bar_rect.min.x + self.mask.feather * bar_rect.width();
            painter.circle_filled(
                Pos2::new(handle_x, bar_rect.center().y),
                5.0,
                Color32::WHITE,
            );
        }

        // Process events
        if event.is_none() {
            // Point interactions
            for info in point_infos.iter() {
                if info.clicked {
                    event = Some(MaskEvent::SelectPoint(Some(info.index)));
                    break;
                }
                if info.dragged {
                    if let Some(pos) = info.drag_pos {
                        let new_point = MaskPoint::from_pos(pos, canvas_rect);
                        event = Some(MaskEvent::MovePoint {
                            index: info.index,
                            position: new_point,
                        });
                        break;
                    }
                }
                if info.secondary_clicked && self.mask.points.len() > 3 {
                    event = Some(MaskEvent::DeletePoint(info.index));
                    break;
                }
            }
        }

        // Add point on double-click
        if event.is_none() && canvas_double_clicked {
            if let Some(pos) = canvas_click_pos {
                let new_point = MaskPoint::from_pos(pos, canvas_rect);
                event = Some(MaskEvent::AddPoint(new_point));
            }
        }

        // Control events
        if event.is_none() {
            if let Some(shape) = shape_clicked {
                event = Some(MaskEvent::SetShape(shape));
            } else if invert_clicked {
                event = Some(MaskEvent::ToggleInvert);
            } else if let Some(feather) = feather_drag {
                event = Some(MaskEvent::SetFeather(feather));
            }
        }

        event
    }
}
