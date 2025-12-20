//! AutomationLane - Parameter automation over time
//!
//! A lane for drawing and editing parameter automation curves.
//!
//! # Example
//! ```ignore
//! AutomationLane::new("Filter Cutoff")
//!     .range(20.0..=20000.0)
//!     .points(&model.automation_points)
//!     .position(model.playhead)
//!     .show_with(ctx, |event| match event {
//!         AutomationEvent::PointAdd(idx, time, value) => Msg::AddPoint(idx, time, value),
//!         AutomationEvent::PointMove(idx, time, value) => Msg::MovePoint(idx, time, value),
//!         AutomationEvent::PointDelete(idx) => Msg::DeletePoint(idx),
//!         AutomationEvent::CurveChange(idx, curve) => Msg::SetCurve(idx, curve),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;
use std::ops::RangeInclusive;

/// Automation events
#[derive(Clone, Debug, PartialEq)]
pub enum AutomationEvent {
    /// Point added (index, normalized time 0-1, normalized value 0-1)
    PointAdd(usize, f32, f32),
    /// Point moved (index, new time, new value)
    PointMove(usize, f32, f32),
    /// Point deleted
    PointDelete(usize),
    /// Curve type changed
    CurveChange(usize, AutomationCurve),
    /// Seek to position
    Seek(f32),
}

/// Automation curve type between points
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AutomationCurve {
    /// Linear interpolation
    #[default]
    Linear,
    /// Stepped (hold previous value)
    Step,
    /// Smooth (S-curve)
    Smooth,
    /// Exponential curve
    Exponential,
    /// Logarithmic curve
    Logarithmic,
}

/// An automation point
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutomationPoint {
    /// Time position (0.0-1.0 normalized)
    pub time: f32,
    /// Value (0.0-1.0 normalized)
    pub value: f32,
    /// Curve to next point
    pub curve: AutomationCurve,
}

impl AutomationPoint {
    /// Create a new automation point
    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time: time.clamp(0.0, 1.0),
            value: value.clamp(0.0, 1.0),
            curve: AutomationCurve::Linear,
        }
    }

    /// Set curve type
    pub fn with_curve(mut self, curve: AutomationCurve) -> Self {
        self.curve = curve;
        self
    }
}

/// Automation lane component
pub struct AutomationLane<'a> {
    label: &'a str,
    points: &'a [AutomationPoint],
    range: RangeInclusive<f32>,
    position: f32,
    height: f32,
    show_grid: bool,
    show_playhead: bool,
    show_value_at_position: bool,
    color: Option<Color32>,
    editable: bool,
    snap_to_grid: bool,
    grid_divisions: u32,
}

impl<'a> AutomationLane<'a> {
    /// Create a new automation lane
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            points: &[],
            range: 0.0..=1.0,
            position: 0.0,
            height: 80.0,
            show_grid: true,
            show_playhead: true,
            show_value_at_position: true,
            color: None,
            editable: true,
            snap_to_grid: false,
            grid_divisions: 16,
        }
    }

    /// Set automation points
    pub fn points(mut self, points: &'a [AutomationPoint]) -> Self {
        self.points = points;
        self
    }

    /// Set value range (for display/formatting)
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    /// Set current playhead position (0.0-1.0)
    pub fn position(mut self, pos: f32) -> Self {
        self.position = pos.clamp(0.0, 1.0);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Show/hide grid
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Show/hide playhead
    pub fn show_playhead(mut self, show: bool) -> Self {
        self.show_playhead = show;
        self
    }

    /// Show value at current position
    pub fn show_value_at_position(mut self, show: bool) -> Self {
        self.show_value_at_position = show;
        self
    }

    /// Set curve color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Enable/disable editing
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Enable snap to grid
    pub fn snap_to_grid(mut self, snap: bool) -> Self {
        self.snap_to_grid = snap;
        self
    }

    /// Set grid divisions
    pub fn grid_divisions(mut self, divisions: u32) -> Self {
        self.grid_divisions = divisions.max(1);
        self
    }

    /// TEA-style: Show lane and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(AutomationEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show lane, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<AutomationEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<AutomationEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        let label_height = 20.0;
        let graph_height = self.height - label_height;
        let width = ui.available_width();

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, self.height),
            if self.editable {
                Sense::click_and_drag()
            } else {
                Sense::click()
            },
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let graph_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y + label_height),
            Vec2::new(width, graph_height),
        );

        let curve_color = self.color.unwrap_or(theme.primary);

        // First pass: collect handle interactions (must happen before painter borrow)
        struct HandleInfo {
            center: egui::Pos2,
            is_hovered: bool,
            handle_color: Color32,
            curve: AutomationCurve,
        }

        let mut handle_infos = Vec::new();

        if !self.points.is_empty() {
            let mut sorted_points: Vec<_> = self.points.iter().enumerate().collect();
            sorted_points.sort_by(|a, b| a.1.time.partial_cmp(&b.1.time).unwrap());

            let handle_size = 6.0;

            for (idx, point) in &sorted_points {
                let x = graph_rect.min.x + point.time * graph_rect.width();
                let y = graph_rect.max.y - point.value * graph_rect.height();
                let center = egui::pos2(x, y);

                let handle_rect = Rect::from_center_size(center, Vec2::splat(handle_size * 2.0));
                let handle_response = ui.allocate_rect(handle_rect, Sense::click_and_drag());

                let is_hovered = handle_response.hovered() || handle_response.dragged();
                let handle_color = if is_hovered {
                    Color32::WHITE
                } else {
                    curve_color
                };

                // Handle drag
                if handle_response.dragged() && self.editable {
                    let delta = handle_response.drag_delta();
                    let new_time = (point.time + delta.x / graph_rect.width()).clamp(0.0, 1.0);
                    let new_value = (point.value - delta.y / graph_rect.height()).clamp(0.0, 1.0);

                    let snapped_time = if self.snap_to_grid {
                        let grid_size = 1.0 / self.grid_divisions as f32;
                        (new_time / grid_size).round() * grid_size
                    } else {
                        new_time
                    };

                    event = Some(AutomationEvent::PointMove(*idx, snapped_time, new_value));
                }

                // Right-click to delete
                if handle_response.secondary_clicked() && self.editable {
                    event = Some(AutomationEvent::PointDelete(*idx));
                }

                handle_infos.push(HandleInfo {
                    center,
                    is_hovered,
                    handle_color,
                    curve: point.curve,
                });
            }
        }

        // Handle double-click to add point
        if response.double_clicked() && self.editable {
            if let Some(pos) = response.interact_pointer_pos() {
                if graph_rect.contains(pos) {
                    let time = (pos.x - graph_rect.min.x) / graph_rect.width();
                    let value = 1.0 - (pos.y - graph_rect.min.y) / graph_rect.height();

                    let snapped_time = if self.snap_to_grid {
                        let grid_size = 1.0 / self.grid_divisions as f32;
                        (time / grid_size).round() * grid_size
                    } else {
                        time
                    };

                    // Find insertion index
                    let insert_idx = self
                        .points
                        .iter()
                        .position(|p| p.time > snapped_time)
                        .unwrap_or(self.points.len());

                    event = Some(AutomationEvent::PointAdd(
                        insert_idx,
                        snapped_time.clamp(0.0, 1.0),
                        value.clamp(0.0, 1.0),
                    ));
                }
            }
        }

        // Handle click to seek
        if response.clicked() && !response.double_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if graph_rect.contains(pos) {
                    let time = (pos.x - graph_rect.min.x) / graph_rect.width();
                    event = Some(AutomationEvent::Seek(time.clamp(0.0, 1.0)));
                }
            }
        }

        // Second pass: all painting
        let painter = ui.painter();

        // Background
        painter.rect_filled(graph_rect, theme.radius_sm, theme.bg_secondary);

        // Grid
        if self.show_grid {
            let grid_color = Color32::from_rgba_unmultiplied(
                theme.border.r(),
                theme.border.g(),
                theme.border.b(),
                40,
            );

            // Vertical grid lines
            for i in 0..=self.grid_divisions {
                let x = graph_rect.min.x + graph_rect.width() * i as f32 / self.grid_divisions as f32;
                let is_major = i % 4 == 0;
                let stroke = if is_major {
                    Stroke::new(1.0, grid_color)
                } else {
                    Stroke::new(0.5, Color32::from_rgba_unmultiplied(
                        grid_color.r(),
                        grid_color.g(),
                        grid_color.b(),
                        grid_color.a() / 2,
                    ))
                };
                painter.line_segment(
                    [egui::pos2(x, graph_rect.min.y), egui::pos2(x, graph_rect.max.y)],
                    stroke,
                );
            }

            // Horizontal grid lines
            for i in 1..4 {
                let y = graph_rect.min.y + graph_rect.height() * i as f32 / 4.0;
                painter.line_segment(
                    [egui::pos2(graph_rect.min.x, y), egui::pos2(graph_rect.max.x, y)],
                    Stroke::new(0.5, grid_color),
                );
            }
        }

        // Draw automation curve
        if !self.points.is_empty() {
            let mut sorted_points: Vec<_> = self.points.iter().collect();
            sorted_points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

            // Draw curve segments
            let segments = 64;
            let mut prev_point: Option<&AutomationPoint> = None;

            for point in &sorted_points {
                if let Some(prev) = prev_point {
                    self.draw_curve_segment(
                        painter,
                        graph_rect,
                        prev,
                        point,
                        segments,
                        curve_color,
                        &theme,
                    );
                }
                prev_point = Some(point);
            }

            // Extend to edges if needed
            if let Some(first) = sorted_points.first() {
                if first.time > 0.0 {
                    let start_point = AutomationPoint::new(0.0, first.value);
                    self.draw_curve_segment(
                        painter,
                        graph_rect,
                        &start_point,
                        first,
                        4,
                        curve_color,
                        &theme,
                    );
                }
            }

            if let Some(last) = sorted_points.last() {
                if last.time < 1.0 {
                    let end_point = AutomationPoint::new(1.0, last.value);
                    self.draw_curve_segment(
                        painter,
                        graph_rect,
                        last,
                        &end_point,
                        4,
                        curve_color,
                        &theme,
                    );
                }
            }

            // Fill under curve
            let fill_color = Color32::from_rgba_unmultiplied(
                curve_color.r(),
                curve_color.g(),
                curve_color.b(),
                30,
            );

            let mut fill_points = Vec::new();
            fill_points.push(egui::pos2(graph_rect.min.x, graph_rect.max.y));

            // Add all curve points
            for point in &sorted_points {
                let x = graph_rect.min.x + point.time * graph_rect.width();
                let y = graph_rect.max.y - point.value * graph_rect.height();
                fill_points.push(egui::pos2(x, y));
            }

            fill_points.push(egui::pos2(graph_rect.max.x, graph_rect.max.y));

            if fill_points.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(
                    fill_points,
                    fill_color,
                    Stroke::NONE,
                ));
            }

            // Draw point handles using collected info
            let handle_size = 6.0;
            for info in &handle_infos {
                painter.circle_filled(info.center, handle_size / 2.0, info.handle_color);
                painter.circle_stroke(
                    info.center,
                    handle_size / 2.0,
                    Stroke::new(1.0, theme.border),
                );

                // Curve type indicator
                let curve_icon = match info.curve {
                    AutomationCurve::Linear => "─",
                    AutomationCurve::Step => "⌐",
                    AutomationCurve::Smooth => "∿",
                    AutomationCurve::Exponential => "↗",
                    AutomationCurve::Logarithmic => "↘",
                };

                if info.is_hovered {
                    painter.text(
                        egui::pos2(info.center.x, info.center.y - 12.0),
                        egui::Align2::CENTER_CENTER,
                        curve_icon,
                        egui::FontId::proportional(10.0),
                        theme.text_muted,
                    );
                }
            }
        }

        // Draw playhead
        if self.show_playhead {
            let playhead_x = graph_rect.min.x + self.position * graph_rect.width();
            painter.line_segment(
                [
                    egui::pos2(playhead_x, graph_rect.min.y),
                    egui::pos2(playhead_x, graph_rect.max.y),
                ],
                Stroke::new(2.0, theme.state_success),
            );

            // Playhead triangle
            let tri_size = 6.0;
            let tri_points = vec![
                egui::pos2(playhead_x - tri_size, graph_rect.min.y),
                egui::pos2(playhead_x + tri_size, graph_rect.min.y),
                egui::pos2(playhead_x, graph_rect.min.y + tri_size),
            ];
            painter.add(egui::Shape::convex_polygon(
                tri_points,
                theme.state_success,
                Stroke::NONE,
            ));
        }

        // Label and current value
        let label_rect = Rect::from_min_size(rect.min, Vec2::new(width, label_height));
        painter.text(
            egui::pos2(label_rect.min.x + 4.0, label_rect.center().y),
            egui::Align2::LEFT_CENTER,
            self.label,
            egui::FontId::proportional(theme.font_size_sm),
            theme.text_secondary,
        );

        // Value at current position
        if self.show_value_at_position && !self.points.is_empty() {
            let current_value = self.interpolate_value(self.position);
            let display_value = *self.range.start()
                + current_value * (*self.range.end() - *self.range.start());
            let value_str = format!("{:.2}", display_value);

            painter.text(
                egui::pos2(label_rect.max.x - 4.0, label_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                value_str,
                egui::FontId::monospace(theme.font_size_sm),
                curve_color,
            );
        }

        // Border
        painter.rect_stroke(
            graph_rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        event
    }

    fn draw_curve_segment(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        from: &AutomationPoint,
        to: &AutomationPoint,
        _segments: usize,
        color: Color32,
        theme: &Theme,
    ) {
        let from_pos = egui::pos2(
            rect.min.x + from.time * rect.width(),
            rect.max.y - from.value * rect.height(),
        );
        let to_pos = egui::pos2(
            rect.min.x + to.time * rect.width(),
            rect.max.y - to.value * rect.height(),
        );

        match from.curve {
            AutomationCurve::Step => {
                // Horizontal then vertical
                let mid_pos = egui::pos2(to_pos.x, from_pos.y);
                painter.line_segment([from_pos, mid_pos], Stroke::new(2.0, color));
                painter.line_segment([mid_pos, to_pos], Stroke::new(2.0, color));
            }
            AutomationCurve::Linear => {
                painter.line_segment([from_pos, to_pos], Stroke::new(2.0, color));
            }
            AutomationCurve::Smooth | AutomationCurve::Exponential | AutomationCurve::Logarithmic => {
                // Use bezier curve for smooth transitions
                let segments = 16;
                let mut points = Vec::with_capacity(segments + 1);

                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let value = match from.curve {
                        AutomationCurve::Smooth => {
                            // S-curve (smoothstep)
                            let t2 = t * t * (3.0 - 2.0 * t);
                            from.value + (to.value - from.value) * t2
                        }
                        AutomationCurve::Exponential => {
                            // Exponential ease-in
                            let t2 = t * t;
                            from.value + (to.value - from.value) * t2
                        }
                        AutomationCurve::Logarithmic => {
                            // Logarithmic ease-out
                            let t2 = 1.0 - (1.0 - t) * (1.0 - t);
                            from.value + (to.value - from.value) * t2
                        }
                        _ => unreachable!(),
                    };
                    let time = from.time + (to.time - from.time) * t;
                    let x = rect.min.x + time * rect.width();
                    let y = rect.max.y - value * rect.height();
                    points.push(egui::pos2(x, y));
                }

                for window in points.windows(2) {
                    painter.line_segment(
                        [window[0], window[1]],
                        Stroke::new(theme.stroke_width * 1.5, color),
                    );
                }
            }
        }
    }

    fn interpolate_value(&self, time: f32) -> f32 {
        if self.points.is_empty() {
            return 0.5;
        }

        let mut sorted_points: Vec<_> = self.points.iter().collect();
        sorted_points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        // Before first point
        if time <= sorted_points[0].time {
            return sorted_points[0].value;
        }

        // After last point
        if time >= sorted_points.last().unwrap().time {
            return sorted_points.last().unwrap().value;
        }

        // Find surrounding points
        for i in 0..sorted_points.len() - 1 {
            let p1 = sorted_points[i];
            let p2 = sorted_points[i + 1];

            if time >= p1.time && time <= p2.time {
                let t = (time - p1.time) / (p2.time - p1.time);
                return match p1.curve {
                    AutomationCurve::Step => p1.value,
                    AutomationCurve::Linear => p1.value + (p2.value - p1.value) * t,
                    AutomationCurve::Smooth => {
                        let t2 = t * t * (3.0 - 2.0 * t);
                        p1.value + (p2.value - p1.value) * t2
                    }
                    AutomationCurve::Exponential => {
                        let t2 = t * t;
                        p1.value + (p2.value - p1.value) * t2
                    }
                    AutomationCurve::Logarithmic => {
                        let t2 = 1.0 - (1.0 - t) * (1.0 - t);
                        p1.value + (p2.value - p1.value) * t2
                    }
                };
            }
        }

        0.5
    }
}
