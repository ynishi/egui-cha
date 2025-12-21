//! OutputRouter atom - Multi-output routing display for VJ applications
//!
//! A component for visualizing and controlling routing between sources and outputs.
//! Supports multiple displays, NDI outputs, recording, and streaming destinations.

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Output type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputType {
    #[default]
    Display,
    NDI,
    Record,
    Stream,
    Syphon,
    Spout,
    Virtual,
}

impl OutputType {
    pub fn icon(&self) -> &'static str {
        match self {
            OutputType::Display => "🖥",
            OutputType::NDI => "📡",
            OutputType::Record => "⏺",
            OutputType::Stream => "📺",
            OutputType::Syphon => "🔗",
            OutputType::Spout => "🔗",
            OutputType::Virtual => "📦",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            OutputType::Display => "Display",
            OutputType::NDI => "NDI",
            OutputType::Record => "Record",
            OutputType::Stream => "Stream",
            OutputType::Syphon => "Syphon",
            OutputType::Spout => "Spout",
            OutputType::Virtual => "Virtual",
        }
    }
}

/// Source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceType {
    #[default]
    Main,
    Preview,
    Layer,
    Aux,
}

impl SourceType {
    pub fn label(&self) -> &'static str {
        match self {
            SourceType::Main => "Main",
            SourceType::Preview => "Preview",
            SourceType::Layer => "Layer",
            SourceType::Aux => "Aux",
        }
    }
}

/// A routing source
#[derive(Debug, Clone)]
pub struct RouteSource {
    pub id: String,
    pub name: String,
    pub source_type: SourceType,
    pub color: Option<Color32>,
}

impl RouteSource {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source_type: SourceType::Main,
            color: None,
        }
    }

    pub fn with_type(mut self, source_type: SourceType) -> Self {
        self.source_type = source_type;
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

/// A routing output destination
#[derive(Debug, Clone)]
pub struct RouteOutput {
    pub id: String,
    pub name: String,
    pub output_type: OutputType,
    pub enabled: bool,
    pub resolution: Option<(u32, u32)>,
}

impl RouteOutput {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            output_type: OutputType::Display,
            enabled: true,
            resolution: None,
        }
    }

    pub fn with_type(mut self, output_type: OutputType) -> Self {
        self.output_type = output_type;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = Some((width, height));
        self
    }
}

/// A routing connection
#[derive(Debug, Clone, PartialEq)]
pub struct RouteConnection {
    pub source_id: String,
    pub output_id: String,
}

impl RouteConnection {
    pub fn new(source_id: impl Into<String>, output_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            output_id: output_id.into(),
        }
    }
}

/// Events emitted by OutputRouter
#[derive(Debug, Clone)]
pub enum RouterEvent {
    Connect {
        source_id: String,
        output_id: String,
    },
    Disconnect {
        source_id: String,
        output_id: String,
    },
    ToggleOutput(String),
    SelectSource(String),
    SelectOutput(String),
}

/// Output router widget
pub struct OutputRouter<'a> {
    sources: &'a [RouteSource],
    outputs: &'a [RouteOutput],
    connections: &'a [RouteConnection],
    selected_source: Option<&'a str>,
    selected_output: Option<&'a str>,
    size: Vec2,
    show_labels: bool,
    show_resolution: bool,
    compact: bool,
}

impl<'a> OutputRouter<'a> {
    pub fn new(
        sources: &'a [RouteSource],
        outputs: &'a [RouteOutput],
        connections: &'a [RouteConnection],
    ) -> Self {
        Self {
            sources,
            outputs,
            connections,
            selected_source: None,
            selected_output: None,
            size: Vec2::new(400.0, 200.0),
            show_labels: true,
            show_resolution: true,
            compact: false,
        }
    }

    pub fn selected_source(mut self, id: Option<&'a str>) -> Self {
        self.selected_source = id;
        self
    }

    pub fn selected_output(mut self, id: Option<&'a str>) -> Self {
        self.selected_output = id;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn show_resolution(mut self, show: bool) -> Self {
        self.show_resolution = show;
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn show_with<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, on_event: impl Fn(RouterEvent) -> Msg) {
        if let Some(e) = self.show_internal(ctx.ui) {
            ctx.emit(on_event(e));
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<RouterEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<RouterEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<RouterEvent> = None;

        let (rect, _response) = ui.allocate_exact_size(self.size, Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let padding = theme.spacing_sm;
        let inner_rect = rect.shrink(padding);

        // Layout: Sources on left, matrix in center, outputs on right
        let source_width = if self.compact { 60.0 } else { 80.0 };
        let output_height = if self.compact { 50.0 } else { 70.0 };
        let node_size = if self.compact { 16.0 } else { 20.0 };

        let matrix_rect = Rect::from_min_max(
            Pos2::new(inner_rect.min.x + source_width + padding, inner_rect.min.y),
            Pos2::new(inner_rect.max.x, inner_rect.max.y - output_height - padding),
        );

        // Collect interactions
        struct SourceInfo {
            id: String,
            rect: Rect,
            node_pos: Pos2,
            clicked: bool,
            hovered: bool,
        }

        struct OutputInfo {
            id: String,
            rect: Rect,
            node_pos: Pos2,
            clicked: bool,
            hovered: bool,
            toggle_clicked: bool,
        }

        struct MatrixNodeInfo {
            source_id: String,
            output_id: String,
            pos: Pos2,
            clicked: bool,
            hovered: bool,
            connected: bool,
        }

        let mut source_infos: Vec<SourceInfo> = Vec::new();
        let mut output_infos: Vec<OutputInfo> = Vec::new();
        let mut matrix_infos: Vec<MatrixNodeInfo> = Vec::new();

        // Source interactions
        let source_spacing = if self.sources.is_empty() {
            0.0
        } else {
            (matrix_rect.height() - node_size) / self.sources.len().max(1) as f32
        };

        for (i, source) in self.sources.iter().enumerate() {
            let y = matrix_rect.min.y + node_size / 2.0 + i as f32 * source_spacing;
            let source_rect = Rect::from_min_size(
                Pos2::new(inner_rect.min.x, y - node_size),
                Vec2::new(source_width, node_size * 2.0),
            );
            let node_pos = Pos2::new(matrix_rect.min.x, y);

            let resp = ui.allocate_rect(source_rect, Sense::click());
            source_infos.push(SourceInfo {
                id: source.id.clone(),
                rect: source_rect,
                node_pos,
                clicked: resp.clicked(),
                hovered: resp.hovered(),
            });
        }

        // Output interactions
        let output_spacing = if self.outputs.is_empty() {
            0.0
        } else {
            (matrix_rect.width() - node_size) / self.outputs.len().max(1) as f32
        };

        for (i, output) in self.outputs.iter().enumerate() {
            let x = matrix_rect.min.x + node_size / 2.0 + i as f32 * output_spacing;
            let output_rect = Rect::from_min_size(
                Pos2::new(x - node_size, inner_rect.max.y - output_height),
                Vec2::new(node_size * 2.0 + 20.0, output_height),
            );
            let node_pos = Pos2::new(x, matrix_rect.max.y);

            let resp = ui.allocate_rect(output_rect, Sense::click());

            // Toggle button area
            let toggle_rect = Rect::from_center_size(
                Pos2::new(x, inner_rect.max.y - theme.spacing_sm - 8.0),
                Vec2::splat(16.0),
            );
            let toggle_resp = ui.allocate_rect(toggle_rect, Sense::click());

            output_infos.push(OutputInfo {
                id: output.id.clone(),
                rect: output_rect,
                node_pos,
                clicked: resp.clicked() && !toggle_resp.hovered(),
                hovered: resp.hovered(),
                toggle_clicked: toggle_resp.clicked(),
            });
        }

        // Matrix node interactions
        for (si, source) in self.sources.iter().enumerate() {
            for (oi, output) in self.outputs.iter().enumerate() {
                let y = matrix_rect.min.y + node_size / 2.0 + si as f32 * source_spacing;
                let x = matrix_rect.min.x + node_size / 2.0 + oi as f32 * output_spacing;
                let pos = Pos2::new(x, y);

                let node_rect = Rect::from_center_size(pos, Vec2::splat(node_size + 4.0));
                let resp = ui.allocate_rect(node_rect, Sense::click());

                let connected = self
                    .connections
                    .iter()
                    .any(|c| c.source_id == source.id && c.output_id == output.id);

                matrix_infos.push(MatrixNodeInfo {
                    source_id: source.id.clone(),
                    output_id: output.id.clone(),
                    pos,
                    clicked: resp.clicked(),
                    hovered: resp.hovered(),
                    connected,
                });
            }
        }

        // Drawing
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_md, theme.bg_secondary);

        // Matrix background
        painter.rect_filled(matrix_rect, theme.radius_sm, theme.bg_primary);

        // Grid lines
        let grid_stroke = Stroke::new(0.5, theme.border.gamma_multiply(0.3));
        for (i, _) in self.sources.iter().enumerate() {
            let y = matrix_rect.min.y + node_size / 2.0 + i as f32 * source_spacing;
            painter.line_segment(
                [
                    Pos2::new(matrix_rect.min.x, y),
                    Pos2::new(matrix_rect.max.x, y),
                ],
                grid_stroke,
            );
        }
        for (i, _) in self.outputs.iter().enumerate() {
            let x = matrix_rect.min.x + node_size / 2.0 + i as f32 * output_spacing;
            painter.line_segment(
                [
                    Pos2::new(x, matrix_rect.min.y),
                    Pos2::new(x, matrix_rect.max.y),
                ],
                grid_stroke,
            );
        }

        // Draw connection lines
        for info in matrix_infos.iter().filter(|i| i.connected) {
            let source_info = source_infos.iter().find(|s| s.id == info.source_id);
            let output_info = output_infos.iter().find(|o| o.id == info.output_id);

            if let (Some(src), Some(out)) = (source_info, output_info) {
                // Horizontal line from source to node
                painter.line_segment(
                    [src.node_pos, Pos2::new(info.pos.x, src.node_pos.y)],
                    Stroke::new(2.0, theme.primary.gamma_multiply(0.5)),
                );
                // Vertical line from node to output
                painter.line_segment(
                    [info.pos, out.node_pos],
                    Stroke::new(2.0, theme.primary.gamma_multiply(0.5)),
                );
            }
        }

        // Draw sources
        for (info, source) in source_infos.iter().zip(self.sources.iter()) {
            let is_selected = self.selected_source == Some(&source.id);
            let is_hovered = info.hovered;

            let color = source.color.unwrap_or(theme.primary);
            let bg = if is_selected {
                color.gamma_multiply(0.3)
            } else if is_hovered {
                theme.bg_tertiary
            } else {
                Color32::TRANSPARENT
            };

            painter.rect_filled(info.rect, theme.radius_sm, bg);

            // Source label
            if self.show_labels {
                painter.text(
                    Pos2::new(info.rect.min.x + theme.spacing_xs, info.node_pos.y),
                    egui::Align2::LEFT_CENTER,
                    &source.name,
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_primary,
                );
            }

            // Source type indicator
            painter.text(
                Pos2::new(
                    info.rect.min.x + theme.spacing_xs,
                    info.rect.max.y - theme.spacing_xs,
                ),
                egui::Align2::LEFT_BOTTOM,
                source.source_type.label(),
                egui::FontId::proportional(theme.font_size_xs * 0.8),
                theme.text_muted,
            );

            // Connection node
            painter.circle_filled(info.node_pos, node_size / 2.0 - 2.0, color);
            painter.circle_stroke(
                info.node_pos,
                node_size / 2.0,
                Stroke::new(1.0, theme.border),
            );
        }

        // Draw outputs
        for (info, output) in output_infos.iter().zip(self.outputs.iter()) {
            let is_selected = self.selected_output == Some(&output.id);
            let is_hovered = info.hovered;

            let enabled_color = if output.enabled {
                theme.primary
            } else {
                theme.text_muted
            };
            let bg = if is_selected {
                enabled_color.gamma_multiply(0.3)
            } else if is_hovered {
                theme.bg_tertiary
            } else {
                Color32::TRANSPARENT
            };

            painter.rect_filled(info.rect, theme.radius_sm, bg);

            // Output icon
            painter.text(
                Pos2::new(info.node_pos.x, info.rect.min.y + output_height * 0.3),
                egui::Align2::CENTER_CENTER,
                output.output_type.icon(),
                egui::FontId::proportional(theme.font_size_md),
                if output.enabled {
                    theme.text_primary
                } else {
                    theme.text_muted
                },
            );

            // Output label
            if self.show_labels {
                painter.text(
                    Pos2::new(info.node_pos.x, info.rect.min.y + output_height * 0.55),
                    egui::Align2::CENTER_CENTER,
                    &output.name,
                    egui::FontId::proportional(theme.font_size_xs),
                    theme.text_secondary,
                );
            }

            // Resolution
            if self.show_resolution {
                if let Some((w, h)) = output.resolution {
                    let res_text = format!("{}x{}", w, h);
                    painter.text(
                        Pos2::new(info.node_pos.x, info.rect.min.y + output_height * 0.75),
                        egui::Align2::CENTER_CENTER,
                        &res_text,
                        egui::FontId::proportional(theme.font_size_xs * 0.8),
                        theme.text_muted,
                    );
                }
            }

            // Enable/disable toggle
            let toggle_rect = Rect::from_center_size(
                Pos2::new(info.node_pos.x, inner_rect.max.y - theme.spacing_sm - 8.0),
                Vec2::splat(14.0),
            );
            let toggle_bg = if output.enabled {
                theme.state_success
            } else {
                theme.bg_tertiary
            };
            painter.rect_filled(toggle_rect, 2.0, toggle_bg);

            // Connection node
            painter.circle_filled(info.node_pos, node_size / 2.0 - 2.0, enabled_color);
            painter.circle_stroke(
                info.node_pos,
                node_size / 2.0,
                Stroke::new(1.0, theme.border),
            );
        }

        // Draw matrix nodes
        for info in matrix_infos.iter() {
            let fill = if info.connected {
                theme.primary
            } else if info.hovered {
                theme.primary.gamma_multiply(0.5)
            } else {
                theme.bg_tertiary
            };

            painter.circle_filled(info.pos, node_size / 2.0 - 3.0, fill);
            if info.connected || info.hovered {
                painter.circle_stroke(
                    info.pos,
                    node_size / 2.0 - 1.0,
                    Stroke::new(1.5, theme.primary),
                );
            }
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_md,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        // Process events
        for info in matrix_infos.iter() {
            if info.clicked {
                if info.connected {
                    event = Some(RouterEvent::Disconnect {
                        source_id: info.source_id.clone(),
                        output_id: info.output_id.clone(),
                    });
                } else {
                    event = Some(RouterEvent::Connect {
                        source_id: info.source_id.clone(),
                        output_id: info.output_id.clone(),
                    });
                }
                break;
            }
        }

        if event.is_none() {
            for info in output_infos.iter() {
                if info.toggle_clicked {
                    event = Some(RouterEvent::ToggleOutput(info.id.clone()));
                    break;
                }
                if info.clicked {
                    event = Some(RouterEvent::SelectOutput(info.id.clone()));
                    break;
                }
            }
        }

        if event.is_none() {
            for info in source_infos.iter() {
                if info.clicked {
                    event = Some(RouterEvent::SelectSource(info.id.clone()));
                    break;
                }
            }
        }

        event
    }
}
