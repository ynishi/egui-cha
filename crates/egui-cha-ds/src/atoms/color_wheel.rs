//! ColorWheel atom - HSV color wheel picker for VJ applications
//!
//! A circular color picker with hue ring and saturation/value triangle or square.
//! Commonly used in VJ software for intuitive color selection.
//!
//! # Features
//! - Hue ring for color selection
//! - Saturation/Value area (triangle or square)
//! - Optional alpha slider
//! - Preview swatch
//! - Hex/RGB value display
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! ColorWheel::new()
//!     .show_with(ctx, model.color, |color| Msg::SetColor(color));
//!
//! // With alpha
//! ColorWheel::new()
//!     .show_alpha(true)
//!     .show_with(ctx, model.color, Msg::SetColor);
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;
use std::f32::consts::PI;

/// HSV color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsva {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

impl Hsva {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self {
            h: h.clamp(0.0, 1.0),
            s: s.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
            a: 1.0,
        }
    }

    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a.clamp(0.0, 1.0);
        self
    }

    pub fn from_color32(color: Color32) -> Self {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;
        let a = color.a() as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            ((g - b) / delta).rem_euclid(6.0) / 6.0
        } else if max == g {
            ((b - r) / delta + 2.0) / 6.0
        } else {
            ((r - g) / delta + 4.0) / 6.0
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        Self { h, s, v, a }
    }

    pub fn to_color32(&self) -> Color32 {
        let h = self.h * 6.0;
        let s = self.s;
        let v = self.v;

        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color32::from_rgba_unmultiplied(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }

    pub fn hue_color(&self) -> Color32 {
        Self::new(self.h, 1.0, 1.0).to_color32()
    }
}

impl Default for Hsva {
    fn default() -> Self {
        Self::new(0.0, 1.0, 1.0)
    }
}

impl From<Color32> for Hsva {
    fn from(color: Color32) -> Self {
        Self::from_color32(color)
    }
}

impl From<Hsva> for Color32 {
    fn from(hsva: Hsva) -> Self {
        hsva.to_color32()
    }
}

/// Color wheel style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WheelStyle {
    #[default]
    Triangle,
    Square,
}

/// HSV Color wheel picker
pub struct ColorWheel {
    size: f32,
    ring_width: f32,
    style: WheelStyle,
    show_alpha: bool,
    show_preview: bool,
    show_values: bool,
}

impl ColorWheel {
    pub fn new() -> Self {
        Self {
            size: 200.0,
            ring_width: 20.0,
            style: WheelStyle::Triangle,
            show_alpha: false,
            show_preview: true,
            show_values: true,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn ring_width(mut self, width: f32) -> Self {
        self.ring_width = width;
        self
    }

    pub fn style(mut self, style: WheelStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show_alpha(mut self, show: bool) -> Self {
        self.show_alpha = show;
        self
    }

    pub fn show_preview(mut self, show: bool) -> Self {
        self.show_preview = show;
        self
    }

    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        color: Color32,
        on_change: impl FnOnce(Color32) -> Msg,
    ) {
        let mut hsva = Hsva::from_color32(color);
        let response = self.show_internal(ctx.ui, &mut hsva);

        if response.changed() {
            ctx.emit(on_change(hsva.to_color32()));
        }
    }

    pub fn show_hsva(self, ui: &mut Ui, hsva: &mut Hsva) -> Response {
        self.show_internal(ui, hsva)
    }

    pub fn show(self, ui: &mut Ui, color: &mut Color32) -> Response {
        let mut hsva = Hsva::from_color32(*color);
        let response = self.show_internal(ui, &mut hsva);
        if response.changed() {
            *color = hsva.to_color32();
        }
        response
    }

    fn show_internal(self, ui: &mut Ui, hsva: &mut Hsva) -> Response {
        let theme = Theme::current(ui.ctx());

        let wheel_size = self.size;
        let alpha_height = if self.show_alpha { theme.spacing_lg } else { 0.0 };
        let preview_height = if self.show_preview { theme.spacing_xl } else { 0.0 };
        let values_height = if self.show_values { theme.font_size_sm * 2.0 + theme.spacing_xs } else { 0.0 };
        let total_height = wheel_size + alpha_height + preview_height + values_height + theme.spacing_sm * 3.0;

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(wheel_size, total_height),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let center = Pos2::new(rect.center().x, rect.min.y + wheel_size / 2.0);
        let outer_radius = wheel_size / 2.0;
        let inner_radius = outer_radius - self.ring_width;

        // First pass: collect interactions
        let ring_rect = Rect::from_center_size(center, Vec2::splat(wheel_size));
        let ring_response = ui.allocate_rect(ring_rect, Sense::click_and_drag());
        let ring_interact_pos = ring_response.interact_pointer_pos();
        let ring_active = ring_response.clicked() || ring_response.dragged();

        // SV area interaction
        let (sv_response, sv_interact_pos) = match self.style {
            WheelStyle::Triangle => {
                let hue_angle = hsva.h * 2.0 * PI - PI / 2.0;
                let v0 = center + Vec2::new(hue_angle.cos() * inner_radius * 0.9, hue_angle.sin() * inner_radius * 0.9);
                let v1 = center + Vec2::new((hue_angle + 2.0 * PI / 3.0).cos() * inner_radius * 0.9, (hue_angle + 2.0 * PI / 3.0).sin() * inner_radius * 0.9);
                let v2 = center + Vec2::new((hue_angle - 2.0 * PI / 3.0).cos() * inner_radius * 0.9, (hue_angle - 2.0 * PI / 3.0).sin() * inner_radius * 0.9);
                let tri_rect = Rect::from_points(&[v0, v1, v2]);
                let resp = ui.allocate_rect(tri_rect, Sense::click_and_drag());
                (resp.clicked() || resp.dragged(), resp.interact_pointer_pos())
            }
            WheelStyle::Square => {
                let half_size = inner_radius * 0.7;
                let sq_rect = Rect::from_center_size(center, Vec2::splat(half_size * 2.0));
                let resp = ui.allocate_rect(sq_rect, Sense::click_and_drag());
                (resp.clicked() || resp.dragged(), resp.interact_pointer_pos())
            }
        };

        // Alpha interaction
        let (alpha_active, alpha_pos) = if self.show_alpha {
            let y_offset = rect.min.y + wheel_size + theme.spacing_sm;
            let alpha_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, y_offset),
                Vec2::new(wheel_size, theme.spacing_md),
            );
            let resp = ui.allocate_rect(alpha_rect, Sense::click_and_drag());
            (resp.clicked() || resp.dragged(), resp.interact_pointer_pos())
        } else {
            (false, None)
        };

        // Second pass: draw everything
        let painter = ui.painter();

        // Draw hue ring
        let segments = 64;
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
            let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

            let hue1 = i as f32 / segments as f32;
            let hue2 = (i + 1) as f32 / segments as f32;

            let color1 = Hsva::new(hue1, 1.0, 1.0).to_color32();
            let color2 = Hsva::new(hue2, 1.0, 1.0).to_color32();

            let outer1 = center + Vec2::new(angle1.cos() * outer_radius, angle1.sin() * outer_radius);
            let outer2 = center + Vec2::new(angle2.cos() * outer_radius, angle2.sin() * outer_radius);
            let inner1 = center + Vec2::new(angle1.cos() * inner_radius, angle1.sin() * inner_radius);
            let inner2 = center + Vec2::new(angle2.cos() * inner_radius, angle2.sin() * inner_radius);

            let mut mesh = egui::Mesh::default();
            mesh.vertices.push(egui::epaint::Vertex { pos: outer1, uv: egui::epaint::WHITE_UV, color: color1 });
            mesh.vertices.push(egui::epaint::Vertex { pos: outer2, uv: egui::epaint::WHITE_UV, color: color2 });
            mesh.vertices.push(egui::epaint::Vertex { pos: inner2, uv: egui::epaint::WHITE_UV, color: color2 });
            mesh.vertices.push(egui::epaint::Vertex { pos: inner1, uv: egui::epaint::WHITE_UV, color: color1 });
            mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
            painter.add(egui::Shape::mesh(mesh));
        }

        // Handle ring interaction
        if ring_active {
            if let Some(pos) = ring_interact_pos {
                let to_center = pos - center;
                let dist = to_center.length();
                if dist >= inner_radius && dist <= outer_radius {
                    let angle = to_center.y.atan2(to_center.x);
                    let hue = (angle / (2.0 * PI) + 0.5).rem_euclid(1.0);
                    hsva.h = hue;
                    response.mark_changed();
                }
            }
        }

        // Draw hue indicator
        let hue_angle = hsva.h * 2.0 * PI - PI;
        let indicator_radius = (inner_radius + outer_radius) / 2.0;
        let indicator_pos = center + Vec2::new(hue_angle.cos() * indicator_radius, hue_angle.sin() * indicator_radius);
        painter.circle_filled(indicator_pos, self.ring_width / 2.0 - 2.0, hsva.hue_color());
        painter.circle_stroke(indicator_pos, self.ring_width / 2.0 - 2.0, Stroke::new(2.0, Color32::WHITE));

        // Draw SV area
        match self.style {
            WheelStyle::Triangle => {
                let hue_angle = hsva.h * 2.0 * PI - PI / 2.0;
                let radius = inner_radius * 0.9;
                let v0 = center + Vec2::new(hue_angle.cos() * radius, hue_angle.sin() * radius);
                let v1 = center + Vec2::new((hue_angle + 2.0 * PI / 3.0).cos() * radius, (hue_angle + 2.0 * PI / 3.0).sin() * radius);
                let v2 = center + Vec2::new((hue_angle - 2.0 * PI / 3.0).cos() * radius, (hue_angle - 2.0 * PI / 3.0).sin() * radius);

                let pure_hue = Hsva::new(hsva.h, 1.0, 1.0).to_color32();
                let mut mesh = egui::Mesh::default();
                mesh.vertices.push(egui::epaint::Vertex { pos: v0, uv: egui::epaint::WHITE_UV, color: pure_hue });
                mesh.vertices.push(egui::epaint::Vertex { pos: v1, uv: egui::epaint::WHITE_UV, color: Color32::BLACK });
                mesh.vertices.push(egui::epaint::Vertex { pos: v2, uv: egui::epaint::WHITE_UV, color: Color32::WHITE });
                mesh.indices.extend_from_slice(&[0, 1, 2]);
                painter.add(egui::Shape::mesh(mesh));
                painter.add(egui::Shape::closed_line(vec![v0, v1, v2], Stroke::new(1.0, theme.border)));

                if sv_response {
                    if let Some(pos) = sv_interact_pos {
                        if let Some((s, v)) = point_to_sv_triangle(pos, v0, v1, v2) {
                            hsva.s = s;
                            hsva.v = v;
                            response.mark_changed();
                        }
                    }
                }

                let sv_pos = sv_to_triangle_point(hsva.s, hsva.v, v0, v1, v2);
                painter.circle_filled(sv_pos, 6.0, hsva.to_color32());
                painter.circle_stroke(sv_pos, 6.0, Stroke::new(2.0, Color32::WHITE));
                painter.circle_stroke(sv_pos, 7.0, Stroke::new(1.0, Color32::BLACK));
            }
            WheelStyle::Square => {
                let half_size = inner_radius * 0.7;
                let sq_rect = Rect::from_center_size(center, Vec2::splat(half_size * 2.0));

                let steps = 16;
                for y in 0..steps {
                    for x in 0..steps {
                        let s = x as f32 / steps as f32;
                        let v = 1.0 - (y as f32 / steps as f32);
                        let color = Hsva::new(hsva.h, s, v).to_color32();
                        let cell_rect = Rect::from_min_size(
                            Pos2::new(sq_rect.min.x + x as f32 * sq_rect.width() / steps as f32, sq_rect.min.y + y as f32 * sq_rect.height() / steps as f32),
                            Vec2::splat(sq_rect.width() / steps as f32 + 1.0),
                        );
                        painter.rect_filled(cell_rect, 0.0, color);
                    }
                }
                painter.rect_stroke(sq_rect, 0.0, Stroke::new(1.0, theme.border), egui::StrokeKind::Inside);

                if sv_response {
                    if let Some(pos) = sv_interact_pos {
                        let s = ((pos.x - sq_rect.min.x) / sq_rect.width()).clamp(0.0, 1.0);
                        let v = (1.0 - (pos.y - sq_rect.min.y) / sq_rect.height()).clamp(0.0, 1.0);
                        hsva.s = s;
                        hsva.v = v;
                        response.mark_changed();
                    }
                }

                let sv_pos = Pos2::new(sq_rect.min.x + hsva.s * sq_rect.width(), sq_rect.min.y + (1.0 - hsva.v) * sq_rect.height());
                painter.circle_filled(sv_pos, 6.0, hsva.to_color32());
                painter.circle_stroke(sv_pos, 6.0, Stroke::new(2.0, Color32::WHITE));
                painter.circle_stroke(sv_pos, 7.0, Stroke::new(1.0, Color32::BLACK));
            }
        }

        let mut y_offset = rect.min.y + wheel_size + theme.spacing_sm;

        // Alpha slider
        if self.show_alpha {
            let alpha_rect = Rect::from_min_size(Pos2::new(rect.min.x, y_offset), Vec2::new(wheel_size, theme.spacing_md));

            // Checkerboard
            let checker_size = 6.0;
            let cols = (alpha_rect.width() / checker_size) as usize;
            let rows = (alpha_rect.height() / checker_size) as usize;
            for row in 0..rows {
                for col in 0..cols {
                    let is_dark = (row + col) % 2 == 0;
                    let color = if is_dark { Color32::from_gray(80) } else { Color32::from_gray(120) };
                    let check_rect = Rect::from_min_size(
                        Pos2::new(alpha_rect.min.x + col as f32 * checker_size, alpha_rect.min.y + row as f32 * checker_size),
                        Vec2::splat(checker_size),
                    ).intersect(alpha_rect);
                    painter.rect_filled(check_rect, 0.0, color);
                }
            }

            // Alpha gradient
            let alpha_color = hsva.to_color32();
            for i in 0..32 {
                let t = i as f32 / 32.0;
                let x = alpha_rect.min.x + t * alpha_rect.width();
                let a = (t * 255.0) as u8;
                let color = Color32::from_rgba_unmultiplied(alpha_color.r(), alpha_color.g(), alpha_color.b(), a);
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(x, alpha_rect.min.y), Vec2::new(alpha_rect.width() / 32.0 + 1.0, alpha_rect.height())),
                    0.0, color,
                );
            }

            painter.rect_stroke(alpha_rect, theme.radius_sm, Stroke::new(theme.border_width, theme.border), egui::StrokeKind::Inside);

            let alpha_x = alpha_rect.min.x + hsva.a * alpha_rect.width();
            painter.circle_filled(Pos2::new(alpha_x, alpha_rect.center().y), theme.spacing_sm, Color32::WHITE);
            painter.circle_stroke(Pos2::new(alpha_x, alpha_rect.center().y), theme.spacing_sm, Stroke::new(1.0, theme.border));

            if alpha_active {
                if let Some(pos) = alpha_pos {
                    hsva.a = ((pos.x - alpha_rect.min.x) / alpha_rect.width()).clamp(0.0, 1.0);
                    response.mark_changed();
                }
            }

            y_offset += theme.spacing_lg + theme.spacing_xs;
        }

        // Preview swatch
        if self.show_preview {
            let preview_rect = Rect::from_min_size(Pos2::new(rect.min.x, y_offset), Vec2::new(wheel_size, theme.spacing_xl));

            if hsva.a < 1.0 {
                let checker_size = 8.0;
                let cols = (preview_rect.width() / checker_size) as usize;
                let rows = (preview_rect.height() / checker_size) as usize;
                for row in 0..rows {
                    for col in 0..cols {
                        let is_dark = (row + col) % 2 == 0;
                        let color = if is_dark { Color32::from_gray(60) } else { Color32::from_gray(100) };
                        let check_rect = Rect::from_min_size(
                            Pos2::new(preview_rect.min.x + col as f32 * checker_size, preview_rect.min.y + row as f32 * checker_size),
                            Vec2::splat(checker_size),
                        ).intersect(preview_rect);
                        painter.rect_filled(check_rect, 0.0, color);
                    }
                }
            }

            painter.rect_filled(preview_rect, theme.radius_sm, hsva.to_color32());
            painter.rect_stroke(preview_rect, theme.radius_sm, Stroke::new(theme.border_width, theme.border), egui::StrokeKind::Inside);

            y_offset += theme.spacing_xl + theme.spacing_xs;
        }

        // Values display
        if self.show_values {
            let color = hsva.to_color32();
            let hex = format!("#{:02X}{:02X}{:02X}{:02X}", color.r(), color.g(), color.b(), color.a());
            let hsv_text = format!("H:{:.0}° S:{:.0}% V:{:.0}%", hsva.h * 360.0, hsva.s * 100.0, hsva.v * 100.0);

            painter.text(Pos2::new(rect.center().x, y_offset), egui::Align2::CENTER_TOP, &hex, egui::FontId::monospace(theme.font_size_sm), theme.text_primary);
            painter.text(Pos2::new(rect.center().x, y_offset + theme.font_size_sm + 2.0), egui::Align2::CENTER_TOP, &hsv_text, egui::FontId::proportional(theme.font_size_xs), theme.text_muted);
        }

        response
    }
}

impl Default for ColorWheel {
    fn default() -> Self {
        Self::new()
    }
}

fn point_to_sv_triangle(p: Pos2, v0: Pos2, v1: Pos2, v2: Pos2) -> Option<(f32, f32)> {
    let v0v1 = v1 - v0;
    let v0v2 = v2 - v0;
    let v0p = p - v0;

    let dot00 = v0v1.dot(v0v1);
    let dot01 = v0v1.dot(v0v2);
    let dot02 = v0v1.dot(v0p);
    let dot11 = v0v2.dot(v0v2);
    let dot12 = v0v2.dot(v0p);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let w = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    let t = 1.0 - u - w;

    let u = u.clamp(0.0, 1.0);
    let w = w.clamp(0.0, 1.0);
    let t = t.clamp(0.0, 1.0);
    let sum = u + w + t;
    let _u = u / sum;
    let w = w / sum;
    let t = t / sum;

    let v = t + w;
    let s = if v > 0.001 { t / v } else { 0.0 };

    Some((s.clamp(0.0, 1.0), v.clamp(0.0, 1.0)))
}

fn sv_to_triangle_point(s: f32, v: f32, v0: Pos2, v1: Pos2, v2: Pos2) -> Pos2 {
    let t = s * v;
    let w = v - t;
    let u = 1.0 - v;

    Pos2::new(t * v0.x + u * v1.x + w * v2.x, t * v0.y + u * v1.y + w * v2.y)
}
