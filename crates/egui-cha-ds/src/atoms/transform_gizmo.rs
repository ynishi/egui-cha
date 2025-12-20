//! TransformGizmo atom - 2D transform control for VJ applications
//!
//! A visual gizmo for manipulating position, rotation, and scale of 2D objects.
//! Provides handles for interactive transformation editing.
//!
//! # Features
//! - Position drag (center)
//! - Rotation handle (circular arc)
//! - Scale handles (corners and edges)
//! - Uniform/non-uniform scale modes
//! - Pivot point display
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! TransformGizmo::new()
//!     .show_with(ctx, &transform, |event| match event {
//!         TransformEvent::Translate(delta) => Msg::Move(delta),
//!         TransformEvent::Rotate(angle) => Msg::Rotate(angle),
//!         TransformEvent::Scale(scale) => Msg::Scale(scale),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;
use std::f32::consts::PI;

/// 2D Transform data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    /// Position (center point)
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale (x, y)
    pub scale: Vec2,
    /// Pivot point offset from center
    pub pivot: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(1.0),
            pivot: Vec2::ZERO,
        }
    }
}

impl Transform2D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_position(mut self, pos: Vec2) -> Self {
        self.position = pos;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }
}

/// Which part of the gizmo is being manipulated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandle {
    None,
    Center,
    Rotation,
    ScaleTopLeft,
    ScaleTopRight,
    ScaleBottomLeft,
    ScaleBottomRight,
    ScaleTop,
    ScaleBottom,
    ScaleLeft,
    ScaleRight,
}

/// Transform events emitted by the gizmo
#[derive(Debug, Clone, Copy)]
pub enum TransformEvent {
    Translate(Vec2),
    Rotate(f32),
    Scale(Vec2),
    DragStart(GizmoHandle),
    DragEnd,
}

/// Transform mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformMode {
    #[default]
    All,
    TranslateOnly,
    RotateOnly,
    ScaleOnly,
}

/// 2D Transform gizmo for VJ/visual applications
pub struct TransformGizmo {
    size: Vec2,
    handle_size: f32,
    show_rotation: bool,
    show_scale: bool,
    uniform_scale: bool,
    mode: TransformMode,
    show_pivot: bool,
}

impl TransformGizmo {
    pub fn new() -> Self {
        Self {
            size: Vec2::new(200.0, 150.0),
            handle_size: 8.0,
            show_rotation: true,
            show_scale: true,
            uniform_scale: false,
            mode: TransformMode::All,
            show_pivot: true,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn handle_size(mut self, size: f32) -> Self {
        self.handle_size = size;
        self
    }

    pub fn show_rotation(mut self, show: bool) -> Self {
        self.show_rotation = show;
        self
    }

    pub fn show_scale(mut self, show: bool) -> Self {
        self.show_scale = show;
        self
    }

    pub fn uniform_scale(mut self, uniform: bool) -> Self {
        self.uniform_scale = uniform;
        self
    }

    pub fn mode(mut self, mode: TransformMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn show_pivot(mut self, show: bool) -> Self {
        self.show_pivot = show;
        self
    }

    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        transform: &Transform2D,
        on_event: impl Fn(TransformEvent) -> Msg,
    ) {
        let events = self.show_internal(ctx.ui, transform);
        for event in events {
            ctx.emit(on_event(event));
        }
    }

    pub fn show(self, ui: &mut Ui, transform: &Transform2D) -> Vec<TransformEvent> {
        self.show_internal(ui, transform)
    }

    fn show_internal(self, ui: &mut Ui, transform: &Transform2D) -> Vec<TransformEvent> {
        let theme = Theme::current(ui.ctx());
        let mut events = Vec::new();

        let scaled_size = self.size * transform.scale;

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(self.size.x + self.handle_size * 4.0, self.size.y + self.handle_size * 4.0 + 30.0),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return events;
        }

        let center = rect.center() + transform.position;
        let half_w = scaled_size.x / 2.0;
        let half_h = scaled_size.y / 2.0;
        let cos_r = transform.rotation.cos();
        let sin_r = transform.rotation.sin();

        let rotate_point = |offset: Vec2| -> Pos2 {
            let rotated = Vec2::new(
                offset.x * cos_r - offset.y * sin_r,
                offset.x * sin_r + offset.y * cos_r,
            );
            center + rotated
        };

        let corners = [
            rotate_point(Vec2::new(-half_w, -half_h)),
            rotate_point(Vec2::new(half_w, -half_h)),
            rotate_point(Vec2::new(half_w, half_h)),
            rotate_point(Vec2::new(-half_w, half_h)),
        ];

        let edges = [
            rotate_point(Vec2::new(0.0, -half_h)),
            rotate_point(Vec2::new(half_w, 0.0)),
            rotate_point(Vec2::new(0.0, half_h)),
            rotate_point(Vec2::new(-half_w, 0.0)),
        ];

        let can_rotate = self.show_rotation && matches!(self.mode, TransformMode::All | TransformMode::RotateOnly);
        let can_scale = self.show_scale && matches!(self.mode, TransformMode::All | TransformMode::ScaleOnly);
        let can_translate = matches!(self.mode, TransformMode::All | TransformMode::TranslateOnly);

        // First pass: collect interactions
        struct HandleInfo {
            handle: GizmoHandle,
            pos: Pos2,
            hovered: bool,
            dragged: bool,
            drag_started: bool,
            drag_stopped: bool,
            drag_delta: Vec2,
            interact_pos: Option<Pos2>,
        }

        let mut handles: Vec<HandleInfo> = Vec::new();

        // Rotation handle
        if can_rotate {
            let rotation_handle_pos = rotate_point(Vec2::new(0.0, -half_h - 25.0));
            let rotation_rect = Rect::from_center_size(rotation_handle_pos, Vec2::splat(self.handle_size * 1.5));
            let response = ui.allocate_rect(rotation_rect, Sense::click_and_drag());
            handles.push(HandleInfo {
                handle: GizmoHandle::Rotation,
                pos: rotation_handle_pos,
                hovered: response.hovered(),
                dragged: response.dragged(),
                drag_started: response.drag_started(),
                drag_stopped: response.drag_stopped(),
                drag_delta: response.drag_delta(),
                interact_pos: response.interact_pointer_pos(),
            });
        }

        // Scale corner handles
        if can_scale {
            let corner_handles = [
                (corners[0], GizmoHandle::ScaleTopLeft),
                (corners[1], GizmoHandle::ScaleTopRight),
                (corners[2], GizmoHandle::ScaleBottomRight),
                (corners[3], GizmoHandle::ScaleBottomLeft),
            ];

            for (pos, handle) in corner_handles {
                let handle_rect = Rect::from_center_size(pos, Vec2::splat(self.handle_size));
                let response = ui.allocate_rect(handle_rect, Sense::click_and_drag());
                handles.push(HandleInfo {
                    handle,
                    pos,
                    hovered: response.hovered(),
                    dragged: response.dragged(),
                    drag_started: response.drag_started(),
                    drag_stopped: response.drag_stopped(),
                    drag_delta: response.drag_delta(),
                    interact_pos: response.interact_pointer_pos(),
                });
            }

            // Edge handles (for non-uniform scale)
            if !self.uniform_scale {
                let edge_handles = [
                    (edges[0], GizmoHandle::ScaleTop),
                    (edges[1], GizmoHandle::ScaleRight),
                    (edges[2], GizmoHandle::ScaleBottom),
                    (edges[3], GizmoHandle::ScaleLeft),
                ];

                for (pos, handle) in edge_handles {
                    let is_horizontal = matches!(handle, GizmoHandle::ScaleTop | GizmoHandle::ScaleBottom);
                    let handle_size = if is_horizontal {
                        Vec2::new(self.handle_size * 2.0, self.handle_size)
                    } else {
                        Vec2::new(self.handle_size, self.handle_size * 2.0)
                    };
                    let handle_rect = Rect::from_center_size(pos, handle_size);
                    let response = ui.allocate_rect(handle_rect, Sense::click_and_drag());
                    handles.push(HandleInfo {
                        handle,
                        pos,
                        hovered: response.hovered(),
                        dragged: response.dragged(),
                        drag_started: response.drag_started(),
                        drag_stopped: response.drag_stopped(),
                        drag_delta: response.drag_delta(),
                        interact_pos: response.interact_pointer_pos(),
                    });
                }
            }
        }

        // Center handle
        let mut center_info = None;
        if can_translate {
            let center_rect = Rect::from_center_size(center, Vec2::splat(self.handle_size * 2.0));
            let response = ui.allocate_rect(center_rect, Sense::click_and_drag());
            center_info = Some(HandleInfo {
                handle: GizmoHandle::Center,
                pos: center,
                hovered: response.hovered(),
                dragged: response.dragged(),
                drag_started: response.drag_started(),
                drag_stopped: response.drag_stopped(),
                drag_delta: response.drag_delta(),
                interact_pos: response.interact_pointer_pos(),
            });
        }

        // Second pass: draw everything
        let painter = ui.painter();
        let primary_color = theme.primary;
        let secondary_color = theme.text_secondary;
        let handle_fill = theme.bg_secondary;
        let rotation_color = Color32::from_rgb(100, 200, 100);

        // Draw bounding box
        let stroke = Stroke::new(theme.stroke_width, primary_color);
        for i in 0..4 {
            painter.line_segment([corners[i], corners[(i + 1) % 4]], stroke);
        }

        // Draw diagonal guides
        let guide_stroke = Stroke::new(0.5, secondary_color.gamma_multiply(0.3));
        painter.line_segment([corners[0], corners[2]], guide_stroke);
        painter.line_segment([corners[1], corners[3]], guide_stroke);

        // Draw rotation handle
        if can_rotate {
            if let Some(info) = handles.iter().find(|h| h.handle == GizmoHandle::Rotation) {
                let rotation_line_start = edges[0];
                painter.line_segment(
                    [rotation_line_start, info.pos],
                    Stroke::new(theme.stroke_width, rotation_color),
                );

                let rotation_handle_color = if info.hovered || info.dragged {
                    rotation_color
                } else {
                    rotation_color.gamma_multiply(0.7)
                };

                painter.circle_filled(info.pos, self.handle_size * 0.75, rotation_handle_color);
                painter.circle_stroke(info.pos, self.handle_size * 0.75, Stroke::new(1.0, theme.bg_primary));

                // Rotation arc
                let arc_radius = 20.0;
                let arc_segments = 16;
                for i in 0..arc_segments {
                    let angle1 = -PI / 4.0 + (i as f32 / arc_segments as f32) * PI / 2.0;
                    let angle2 = -PI / 4.0 + ((i + 1) as f32 / arc_segments as f32) * PI / 2.0;
                    let p1 = info.pos + Vec2::new(angle1.cos() * arc_radius, angle1.sin() * arc_radius);
                    let p2 = info.pos + Vec2::new(angle2.cos() * arc_radius, angle2.sin() * arc_radius);
                    painter.line_segment([p1, p2], Stroke::new(0.5, rotation_color.gamma_multiply(0.5)));
                }
            }
        }

        // Draw scale handles
        for info in handles.iter() {
            match info.handle {
                GizmoHandle::ScaleTopLeft | GizmoHandle::ScaleTopRight |
                GizmoHandle::ScaleBottomLeft | GizmoHandle::ScaleBottomRight => {
                    let handle_rect = Rect::from_center_size(info.pos, Vec2::splat(self.handle_size));
                    let fill = if info.hovered || info.dragged { primary_color } else { handle_fill };
                    painter.rect_filled(handle_rect, 1.0, fill);
                    painter.rect_stroke(handle_rect, 1.0, Stroke::new(theme.stroke_width, primary_color), egui::StrokeKind::Outside);
                }
                GizmoHandle::ScaleTop | GizmoHandle::ScaleBottom |
                GizmoHandle::ScaleLeft | GizmoHandle::ScaleRight => {
                    let is_horizontal = matches!(info.handle, GizmoHandle::ScaleTop | GizmoHandle::ScaleBottom);
                    let handle_size = if is_horizontal {
                        Vec2::new(self.handle_size * 2.0, self.handle_size)
                    } else {
                        Vec2::new(self.handle_size, self.handle_size * 2.0)
                    };
                    let handle_rect = Rect::from_center_size(info.pos, handle_size);
                    let fill = if info.hovered || info.dragged { primary_color.gamma_multiply(0.8) } else { handle_fill };
                    painter.rect_filled(handle_rect, 1.0, fill);
                    painter.rect_stroke(handle_rect, 1.0, Stroke::new(theme.stroke_width * 0.5, primary_color), egui::StrokeKind::Outside);
                }
                _ => {}
            }
        }

        // Draw center handle
        if let Some(info) = &center_info {
            let center_color = if info.hovered || info.dragged { primary_color } else { secondary_color };
            let cross_size = self.handle_size;
            painter.line_segment(
                [center - Vec2::new(cross_size, 0.0), center + Vec2::new(cross_size, 0.0)],
                Stroke::new(theme.stroke_width, center_color),
            );
            painter.line_segment(
                [center - Vec2::new(0.0, cross_size), center + Vec2::new(0.0, cross_size)],
                Stroke::new(theme.stroke_width, center_color),
            );
            painter.circle_stroke(center, cross_size * 0.6, Stroke::new(theme.stroke_width, center_color));
        }

        // Draw pivot point
        if self.show_pivot && transform.pivot != Vec2::ZERO {
            let pivot_pos = center + transform.pivot;
            let pivot_size = self.handle_size * 0.5;
            painter.circle_stroke(pivot_pos, pivot_size, Stroke::new(1.0, theme.state_warning));
            painter.line_segment(
                [pivot_pos - Vec2::new(pivot_size, 0.0), pivot_pos + Vec2::new(pivot_size, 0.0)],
                Stroke::new(1.0, theme.state_warning),
            );
            painter.line_segment(
                [pivot_pos - Vec2::new(0.0, pivot_size), pivot_pos + Vec2::new(0.0, pivot_size)],
                Stroke::new(1.0, theme.state_warning),
            );
        }

        // Info display
        let info_text = format!(
            "P({:.0},{:.0}) R:{:.1}° S({:.2},{:.2})",
            transform.position.x,
            transform.position.y,
            transform.rotation.to_degrees(),
            transform.scale.x,
            transform.scale.y,
        );
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - theme.font_size_xs),
            egui::Align2::CENTER_BOTTOM,
            &info_text,
            egui::FontId::proportional(theme.font_size_xs),
            theme.text_muted,
        );

        // Process handle events
        for info in handles.iter() {
            if info.drag_started {
                events.push(TransformEvent::DragStart(info.handle));
            }
            if info.drag_stopped {
                events.push(TransformEvent::DragEnd);
            }
            if info.dragged {
                match info.handle {
                    GizmoHandle::Rotation => {
                        if let Some(pos) = info.interact_pos {
                            let to_cursor = pos - center;
                            let angle = to_cursor.y.atan2(to_cursor.x) + PI / 2.0;
                            let delta = angle - transform.rotation;
                            events.push(TransformEvent::Rotate(delta));
                        }
                    }
                    GizmoHandle::ScaleTopLeft | GizmoHandle::ScaleTopRight |
                    GizmoHandle::ScaleBottomLeft | GizmoHandle::ScaleBottomRight => {
                        if let Some(pos) = info.interact_pos {
                            let to_cursor = pos - center;
                            let unrotated = Vec2::new(
                                to_cursor.x * cos_r + to_cursor.y * sin_r,
                                -to_cursor.x * sin_r + to_cursor.y * cos_r,
                            );
                            let new_scale = if self.uniform_scale {
                                let avg = (unrotated.x.abs() / (self.size.x / 2.0)
                                    + unrotated.y.abs() / (self.size.y / 2.0)) / 2.0;
                                Vec2::splat(avg.max(0.1))
                            } else {
                                Vec2::new(
                                    (unrotated.x.abs() / (self.size.x / 2.0)).max(0.1),
                                    (unrotated.y.abs() / (self.size.y / 2.0)).max(0.1),
                                )
                            };
                            events.push(TransformEvent::Scale(new_scale));
                        }
                    }
                    GizmoHandle::ScaleTop | GizmoHandle::ScaleBottom => {
                        if let Some(pos) = info.interact_pos {
                            let to_cursor = pos - center;
                            let unrotated = Vec2::new(
                                to_cursor.x * cos_r + to_cursor.y * sin_r,
                                -to_cursor.x * sin_r + to_cursor.y * cos_r,
                            );
                            let new_scale = Vec2::new(
                                transform.scale.x,
                                (unrotated.y.abs() / (self.size.y / 2.0)).max(0.1),
                            );
                            events.push(TransformEvent::Scale(new_scale));
                        }
                    }
                    GizmoHandle::ScaleLeft | GizmoHandle::ScaleRight => {
                        if let Some(pos) = info.interact_pos {
                            let to_cursor = pos - center;
                            let unrotated = Vec2::new(
                                to_cursor.x * cos_r + to_cursor.y * sin_r,
                                -to_cursor.x * sin_r + to_cursor.y * cos_r,
                            );
                            let new_scale = Vec2::new(
                                (unrotated.x.abs() / (self.size.x / 2.0)).max(0.1),
                                transform.scale.y,
                            );
                            events.push(TransformEvent::Scale(new_scale));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Center handle events
        if let Some(info) = center_info {
            if info.drag_started {
                events.push(TransformEvent::DragStart(GizmoHandle::Center));
            }
            if info.drag_stopped {
                events.push(TransformEvent::DragEnd);
            }
            if info.dragged {
                events.push(TransformEvent::Translate(info.drag_delta));
            }
        }

        events
    }
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self::new()
    }
}
