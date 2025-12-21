//! EnvelopeEditor - ADSR and custom envelope editing
//!
//! An interactive envelope editor for ADSR, multi-point, and custom envelopes.
//!
//! # Example
//! ```ignore
//! EnvelopeEditor::adsr()
//!     .attack(model.attack)
//!     .decay(model.decay)
//!     .sustain(model.sustain)
//!     .release(model.release)
//!     .show_with(ctx, |event| match event {
//!         EnvelopeEvent::AttackChange(v) => Msg::SetAttack(v),
//!         EnvelopeEvent::DecayChange(v) => Msg::SetDecay(v),
//!         EnvelopeEvent::SustainChange(v) => Msg::SetSustain(v),
//!         EnvelopeEvent::ReleaseChange(v) => Msg::SetRelease(v),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Envelope events
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnvelopeEvent {
    /// Attack time changed (0.0-1.0)
    AttackChange(f32),
    /// Decay time changed (0.0-1.0)
    DecayChange(f32),
    /// Sustain level changed (0.0-1.0)
    SustainChange(f32),
    /// Release time changed (0.0-1.0)
    ReleaseChange(f32),
    /// Custom point moved (index, x, y)
    PointMove(usize, f32, f32),
}

/// Envelope point for custom envelopes
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvelopePoint {
    /// X position (0.0-1.0, time)
    pub x: f32,
    /// Y position (0.0-1.0, level)
    pub y: f32,
    /// Curve type to next point
    pub curve: CurveType,
}

impl EnvelopePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            curve: CurveType::Linear,
        }
    }

    pub fn with_curve(mut self, curve: CurveType) -> Self {
        self.curve = curve;
        self
    }
}

/// Curve type between points
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CurveType {
    #[default]
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

/// Envelope editor component
pub struct EnvelopeEditor<'a> {
    // ADSR values
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    // Custom points (if not using ADSR)
    custom_points: Option<&'a [EnvelopePoint]>,
    // Display settings
    width: f32,
    height: f32,
    show_grid: bool,
    show_labels: bool,
    show_values: bool,
    color: Option<Color32>,
    fill: bool,
}

impl<'a> EnvelopeEditor<'a> {
    /// Create ADSR envelope editor
    pub fn adsr() -> Self {
        Self {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
            custom_points: None,
            width: 200.0,
            height: 100.0,
            show_grid: true,
            show_labels: true,
            show_values: true,
            color: None,
            fill: true,
        }
    }

    /// Create custom envelope editor
    pub fn custom(points: &'a [EnvelopePoint]) -> Self {
        Self {
            attack: 0.0,
            decay: 0.0,
            sustain: 0.0,
            release: 0.0,
            custom_points: Some(points),
            width: 200.0,
            height: 100.0,
            show_grid: true,
            show_labels: false,
            show_values: false,
            color: None,
            fill: true,
        }
    }

    /// Set attack time (0.0-1.0)
    pub fn attack(mut self, attack: f32) -> Self {
        self.attack = attack.clamp(0.0, 1.0);
        self
    }

    /// Set decay time (0.0-1.0)
    pub fn decay(mut self, decay: f32) -> Self {
        self.decay = decay.clamp(0.0, 1.0);
        self
    }

    /// Set sustain level (0.0-1.0)
    pub fn sustain(mut self, sustain: f32) -> Self {
        self.sustain = sustain.clamp(0.0, 1.0);
        self
    }

    /// Set release time (0.0-1.0)
    pub fn release(mut self, release: f32) -> Self {
        self.release = release.clamp(0.0, 1.0);
        self
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Show/hide grid
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Show/hide labels (A, D, S, R)
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Show/hide value display
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Set envelope color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Enable/disable fill under curve
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// TEA-style: Show editor and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(EnvelopeEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show editor, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<EnvelopeEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<EnvelopeEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        let label_height = if self.show_labels { 16.0 } else { 0.0 };
        let value_height = if self.show_values { 14.0 } else { 0.0 };
        let graph_height = self.height - label_height - value_height;

        let (rect, _) = ui.allocate_exact_size(Vec2::new(self.width, self.height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let graph_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y + label_height),
            Vec2::new(self.width, graph_height),
        );

        let env_color = self.color.unwrap_or(theme.primary);

        // Calculate ADSR points
        let total_time = self.attack + self.decay + 0.2 + self.release; // 0.2 for sustain hold
        let attack_x = self.attack / total_time;
        let decay_x = attack_x + self.decay / total_time;
        let sustain_x = decay_x + 0.2 / total_time;
        let release_x = 1.0;

        // Handle point dragging
        struct HandleInfo {
            rect: Rect,
            point_type: usize, // 0=attack, 1=decay, 2=sustain, 3=release
        }

        let handle_size = 8.0;
        let handles = [
            HandleInfo {
                rect: Rect::from_center_size(
                    egui::pos2(
                        graph_rect.min.x + attack_x * graph_rect.width(),
                        graph_rect.min.y,
                    ),
                    Vec2::splat(handle_size * 2.0),
                ),
                point_type: 0,
            },
            HandleInfo {
                rect: Rect::from_center_size(
                    egui::pos2(
                        graph_rect.min.x + decay_x * graph_rect.width(),
                        graph_rect.min.y + (1.0 - self.sustain) * graph_rect.height(),
                    ),
                    Vec2::splat(handle_size * 2.0),
                ),
                point_type: 1,
            },
            HandleInfo {
                rect: Rect::from_center_size(
                    egui::pos2(
                        graph_rect.min.x + sustain_x * graph_rect.width(),
                        graph_rect.min.y + (1.0 - self.sustain) * graph_rect.height(),
                    ),
                    Vec2::splat(handle_size * 2.0),
                ),
                point_type: 2,
            },
            HandleInfo {
                rect: Rect::from_center_size(
                    egui::pos2(
                        graph_rect.min.x + release_x * graph_rect.width(),
                        graph_rect.max.y,
                    ),
                    Vec2::splat(handle_size * 2.0),
                ),
                point_type: 3,
            },
        ];

        // Collect interaction responses
        let mut responses: Vec<(egui::Response, usize)> = Vec::new();
        for handle in &handles {
            let response = ui.allocate_rect(handle.rect, Sense::drag());
            responses.push((response, handle.point_type));
        }

        // Handle dragging
        for (response, point_type) in &responses {
            if response.dragged() {
                let delta = response.drag_delta();
                match point_type {
                    0 => {
                        // Attack - horizontal only
                        let delta_norm = delta.x / graph_rect.width();
                        let new_attack = (self.attack + delta_norm * total_time).clamp(0.01, 0.99);
                        event = Some(EnvelopeEvent::AttackChange(new_attack));
                    }
                    1 => {
                        // Decay point - x for decay time, y for sustain level
                        let delta_x = delta.x / graph_rect.width();
                        let delta_y = -delta.y / graph_rect.height();
                        let new_decay = (self.decay + delta_x * total_time).clamp(0.01, 0.99);
                        let new_sustain = (self.sustain + delta_y).clamp(0.0, 1.0);
                        if delta_x.abs() > delta_y.abs() {
                            event = Some(EnvelopeEvent::DecayChange(new_decay));
                        } else {
                            event = Some(EnvelopeEvent::SustainChange(new_sustain));
                        }
                    }
                    2 => {
                        // Sustain - vertical only
                        let delta_norm = -delta.y / graph_rect.height();
                        let new_sustain = (self.sustain + delta_norm).clamp(0.0, 1.0);
                        event = Some(EnvelopeEvent::SustainChange(new_sustain));
                    }
                    3 => {
                        // Release - horizontal only (from sustain point)
                        let delta_norm = delta.x / graph_rect.width();
                        let new_release =
                            (self.release - delta_norm * total_time).clamp(0.01, 0.99);
                        event = Some(EnvelopeEvent::ReleaseChange(new_release));
                    }
                    _ => {}
                }
            }
        }

        // Draw
        let painter = ui.painter();

        // Background
        painter.rect_filled(graph_rect, theme.radius_sm, theme.bg_secondary);

        // Grid
        if self.show_grid {
            let grid_color = Color32::from_rgba_unmultiplied(
                theme.border.r(),
                theme.border.g(),
                theme.border.b(),
                50,
            );
            // Horizontal lines
            for i in 1..4 {
                let y = graph_rect.min.y + graph_rect.height() * i as f32 / 4.0;
                painter.line_segment(
                    [
                        egui::pos2(graph_rect.min.x, y),
                        egui::pos2(graph_rect.max.x, y),
                    ],
                    Stroke::new(1.0, grid_color),
                );
            }
            // Vertical lines at ADSR points
            for x_norm in [attack_x, decay_x, sustain_x] {
                let x = graph_rect.min.x + x_norm * graph_rect.width();
                painter.line_segment(
                    [
                        egui::pos2(x, graph_rect.min.y),
                        egui::pos2(x, graph_rect.max.y),
                    ],
                    Stroke::new(1.0, grid_color),
                );
            }
        }

        // Envelope curve points
        let points = [
            egui::pos2(graph_rect.min.x, graph_rect.max.y), // Start at 0
            egui::pos2(
                graph_rect.min.x + attack_x * graph_rect.width(),
                graph_rect.min.y,
            ), // Attack peak
            egui::pos2(
                graph_rect.min.x + decay_x * graph_rect.width(),
                graph_rect.min.y + (1.0 - self.sustain) * graph_rect.height(),
            ), // Decay end / Sustain start
            egui::pos2(
                graph_rect.min.x + sustain_x * graph_rect.width(),
                graph_rect.min.y + (1.0 - self.sustain) * graph_rect.height(),
            ), // Sustain end
            egui::pos2(graph_rect.max.x, graph_rect.max.y), // Release end
        ];

        // Fill under curve
        if self.fill {
            let fill_color =
                Color32::from_rgba_unmultiplied(env_color.r(), env_color.g(), env_color.b(), 40);
            let mut fill_points = points.to_vec();
            fill_points.push(egui::pos2(graph_rect.max.x, graph_rect.max.y));
            fill_points.push(egui::pos2(graph_rect.min.x, graph_rect.max.y));
            painter.add(egui::Shape::convex_polygon(
                fill_points,
                fill_color,
                Stroke::NONE,
            ));
        }

        // Draw envelope line
        for window in points.windows(2) {
            painter.line_segment([window[0], window[1]], Stroke::new(2.0, env_color));
        }

        // Draw handles
        for (i, (response, _)) in responses.iter().enumerate() {
            let handle = &handles[i];
            let center = handle.rect.center();
            let is_hovered = response.hovered() || response.dragged();

            let handle_color = if is_hovered {
                Color32::WHITE
            } else {
                env_color
            };

            painter.circle_filled(center, handle_size / 2.0, handle_color);
            painter.circle_stroke(center, handle_size / 2.0, Stroke::new(1.0, theme.border));
        }

        // Labels
        if self.show_labels {
            let labels = ["A", "D", "S", "R"];
            let label_positions = [
                graph_rect.min.x + attack_x * graph_rect.width() / 2.0,
                graph_rect.min.x + (attack_x + decay_x) * graph_rect.width() / 2.0,
                graph_rect.min.x + (decay_x + sustain_x) * graph_rect.width() / 2.0,
                graph_rect.min.x + (sustain_x + 1.0) * graph_rect.width() / 2.0,
            ];

            for (label, x) in labels.iter().zip(label_positions.iter()) {
                painter.text(
                    egui::pos2(*x, rect.min.y + label_height / 2.0),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_secondary,
                );
            }
        }

        // Values
        if self.show_values {
            let values = [
                format!("{:.0}ms", self.attack * 1000.0),
                format!("{:.0}ms", self.decay * 1000.0),
                format!("{:.0}%", self.sustain * 100.0),
                format!("{:.0}ms", self.release * 1000.0),
            ];
            let value_positions = [
                graph_rect.min.x + attack_x * graph_rect.width() / 2.0,
                graph_rect.min.x + (attack_x + decay_x) * graph_rect.width() / 2.0,
                graph_rect.min.x + (decay_x + sustain_x) * graph_rect.width() / 2.0,
                graph_rect.min.x + (sustain_x + 1.0) * graph_rect.width() / 2.0,
            ];

            for (value, x) in values.iter().zip(value_positions.iter()) {
                painter.text(
                    egui::pos2(*x, graph_rect.max.y + value_height / 2.0 + 2.0),
                    egui::Align2::CENTER_CENTER,
                    value,
                    egui::FontId::monospace(theme.font_size_xs * 0.85),
                    theme.text_muted,
                );
            }
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
}
